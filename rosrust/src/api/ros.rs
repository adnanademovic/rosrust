use msg::{Duration, Time};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, mpsc};
use std::time;
use super::clock::{Clock, Rate, RealClock, SimulatedClock};
use super::clock::rosgraph_msgs::Clock as ClockMsg;
use super::master::{self, Master, Topic};
use super::slave::Slave;
use super::error::{ErrorKind, Result, ResultExt};
use super::super::rosxmlrpc::Response;
use super::naming::{self, Resolver};
use super::raii::{Publisher, Subscriber, Service};
use super::resolve;
use tcpros::{Client, Message, ServicePair, ServiceResult};
use xml_rpc;
use yaml_rust::{Yaml, YamlLoader};

pub struct Ros {
    master: Arc<Master>,
    slave: Arc<Slave>,
    hostname: String,
    resolver: Resolver,
    name: String,
    clock: Arc<Clock>,
    static_subs: Vec<Subscriber>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl Ros {
    pub fn new(name: &str) -> Result<Ros> {
        let namespace = resolve::namespace();
        let master_uri = resolve::master();
        let hostname = resolve::hostname();
        let name = resolve::name(name);
        let mut ros = Ros::new_raw(&master_uri, &hostname, &namespace, &name)?;
        for (src, dest) in resolve::mappings() {
            ros.map(&src, &dest)?;
        }
        for (src, dest) in resolve::params() {
            let data = YamlLoader::load_from_str(&dest)
                .chain_err(|| format!("Failed to load YAML: {}", dest))?
                .into_iter()
                .next()
                .ok_or_else(|| format!("Failed to load YAML: {}", dest))?;
            let param = ros.param(&src).ok_or_else(
                || format!("Failed to resolve name: {}", src),
            )?;
            param.set_raw(yaml_to_xmlrpc(data)?)?;
        }

        if ros.param("/use_sim_time")
            .and_then(|v| v.get().ok())
            .unwrap_or(false)
        {
            let clock = Arc::new(SimulatedClock::default());
            let ros_clock = clock.clone();
            let sub = ros.subscribe::<ClockMsg, _>("/clock", move |v| clock.trigger(v.clock))
                .chain_err(|| "Failed to subscribe to simulated clock")?;
            ros.static_subs.push(sub);
            ros.clock = ros_clock;
        }

        Ok(ros)
    }

    fn new_raw(master_uri: &str, hostname: &str, namespace: &str, name: &str) -> Result<Ros> {
        let namespace = namespace.trim_right_matches("/");

        if name.contains("/") {
            bail!(ErrorKind::Naming(
                naming::error::ErrorKind::IllegalCharacter(name.into()),
            ));
        }

        let name = format!("{}/{}", namespace, name);
        let resolver = Resolver::new(&name)?;

        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let slave = Slave::new(&master_uri, &hostname, 0, &name, shutdown_tx)?;
        let master = Master::new(&master_uri, &name, &slave.uri());

        Ok(Ros {
            master: Arc::new(master),
            slave: Arc::new(slave),
            hostname: String::from(hostname),
            resolver: resolver,
            name: name,
            clock: Arc::new(RealClock::default()),
            static_subs: Vec::new(),
            shutdown_rx,
        })
    }

    fn map(&mut self, source: &str, destination: &str) -> Result<()> {
        self.resolver.map(source, destination).map_err(|v| v.into())
    }

    pub fn uri(&self) -> &str {
        return self.slave.uri();
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn hostname(&self) -> &str {
        return &self.hostname;
    }

    pub fn now(&self) -> Time {
        self.clock.now()
    }

    pub fn sleep(&self, d: Duration) {
        self.clock.sleep(d);
    }

    pub fn rate(&self, rate: f64) -> Rate {
        let nanos = 1_000_000_000.0 / rate;
        Rate::new(
            self.clock.clone(),
            self.now(),
            Duration::from_nanos(nanos as i64),
        )
    }

    pub fn is_ok(&self) -> bool {
        self.shutdown_rx.try_recv() == Err(mpsc::TryRecvError::Empty)
    }

    pub fn spin(&self) {
        self.shutdown_rx.recv().is_ok();
    }

    pub fn param<'a, 'b>(&'a self, name: &'b str) -> Option<Parameter<'a>> {
        self.resolver.translate(name).ok().map(|v| {
            Parameter {
                master: &self.master,
                name: v,
            }
        })
    }

    pub fn parameters(&self) -> Response<Vec<String>> {
        self.master.get_param_names()
    }

    pub fn state(&self) -> Response<master::SystemState> {
        self.master.get_system_state().map(Into::into)
    }

    pub fn topics(&self) -> Response<Vec<Topic>> {
        self.master.get_topic_types().map(|v| {
            v.into_iter().map(Into::into).collect()
        })
    }

    pub fn client<T: ServicePair>(&self, service: &str) -> Result<Client<T>> {
        let name = self.resolver.translate(service)?;
        let uri = self.master.lookup_service(&name)?;
        Ok(Client::new(&self.name, &uri, &name))
    }

    pub fn wait_for_service(&self, service: &str, timeout: Option<time::Duration>) -> Result<()> {
        use rosxmlrpc::ResponseError;
        use std::thread::sleep;

        let name = self.resolver.translate(service)?;
        let now = ::std::time::Instant::now();
        loop {
            let e = match self.master.lookup_service(&name) {
                Ok(_) => return Ok(()),
                Err(e) => e,
            };
            match e {
                ResponseError::Client(ref m) if m == "no provider" => {
                    if let Some(ref timeout) = timeout {
                        if &now.elapsed() > timeout {
                            return Err(ErrorKind::TimeoutError.into());
                        }
                    }
                    sleep(time::Duration::from_millis(100));
                    continue;
                }
                _ => {}
            }
            return Err(e.into());
        }
    }

    pub fn service<T, F>(&mut self, service: &str, handler: F) -> Result<Service>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        let name = self.resolver.translate(service)?;
        Service::new::<T, F>(
            self.master.clone(),
            self.slave.clone(),
            &self.hostname,
            &name,
            handler,
        )
    }

    pub fn subscribe<T, F>(&mut self, topic: &str, callback: F) -> Result<Subscriber>
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        let name = self.resolver.translate(topic)?;
        Subscriber::new::<T, F>(self.master.clone(), self.slave.clone(), &name, callback)
    }

    pub fn publish<T>(&mut self, topic: &str) -> Result<Publisher<T>>
    where
        T: Message,
    {
        let name = self.resolver.translate(topic)?;
        Publisher::new(
            self.master.clone(),
            self.slave.clone(),
            &self.hostname,
            &name,
        )
    }
}

pub struct Parameter<'a> {
    master: &'a Master,
    name: String,
}

impl<'a> Parameter<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get<'b, T: Deserialize<'b>>(&self) -> Response<T> {
        self.master.get_param::<T>(&self.name)
    }

    pub fn get_raw(&self) -> Response<xml_rpc::Value> {
        self.master.get_param_any(&self.name)
    }

    pub fn set<T: Serialize>(&self, value: &T) -> Response<()> {
        self.master.set_param::<T>(&self.name, value).and(Ok(()))
    }

    pub fn set_raw(&self, value: xml_rpc::Value) -> Response<()> {
        self.master.set_param_any(&self.name, value).and(Ok(()))
    }

    pub fn delete(&self) -> Response<()> {
        self.master.delete_param(&self.name).and(Ok(()))
    }

    pub fn exists(&self) -> Response<bool> {
        self.master.has_param(&self.name)
    }

    pub fn search(&self) -> Response<String> {
        self.master.search_param(&self.name)
    }
}

fn yaml_to_xmlrpc(val: Yaml) -> Result<xml_rpc::Value> {
    Ok(match val {
        Yaml::Real(v) => xml_rpc::Value::Double(v.parse().chain_err(|| "Failed to parse float")?),
        Yaml::Integer(v) => xml_rpc::Value::Int(v as i32),
        Yaml::String(v) => xml_rpc::Value::String(v),
        Yaml::Boolean(v) => xml_rpc::Value::Bool(v),
        Yaml::Array(v) => xml_rpc::Value::Array(
            v.into_iter().map(yaml_to_xmlrpc).collect::<Result<_>>()?,
        ),
        Yaml::Hash(v) => xml_rpc::Value::Struct(v.into_iter()
            .map(|(k, v)| Ok((yaml_to_string(k)?, yaml_to_xmlrpc(v)?)))
            .collect::<Result<_>>()?),
        Yaml::Alias(_) => bail!("Alias is not supported"),
        Yaml::Null => bail!("Illegal null value"),
        Yaml::BadValue => bail!("Bad value provided"),
    })
}

fn yaml_to_string(val: Yaml) -> Result<String> {
    Ok(match val {
        Yaml::Real(v) | Yaml::String(v) => v,
        Yaml::Integer(v) => v.to_string(),
        Yaml::Boolean(true) => "true".into(),
        Yaml::Boolean(false) => "false".into(),
        _ => bail!("Hash keys need to be strings"),
    })
}

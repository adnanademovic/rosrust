use msg::rosgraph_msgs::{Clock as ClockMsg, Log};
use msg::std_msgs::Header;
use time::{Duration, Time};
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Arc};
use std::time;
use super::clock::{Clock, Rate, RealClock, SimulatedClock};
use super::master::{self, Master, Topic};
use super::slave::Slave;
use super::error::{ErrorKind, Result, ResultExt};
use super::super::rosxmlrpc::Response;
use super::naming::{self, Resolver};
use super::raii::{Publisher, Service, Subscriber};
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
    logger: Option<Publisher<Log>>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl Ros {
    pub fn new(name: &str) -> Result<Ros> {
        let mut namespace = resolve::namespace();
        if !namespace.starts_with('/') {
            namespace = format!("/{}", namespace);
        }
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
            let param = ros.param(&src)
                .ok_or_else(|| format!("Failed to resolve name: {}", src))?;
            param.set_raw(yaml_to_xmlrpc(data)?)?;
        }

        if ros.param("/use_sim_time")
            .and_then(|v| v.get().ok())
            .unwrap_or(false)
        {
            let clock = Arc::new(SimulatedClock::default());
            let ros_clock = Arc::clone(&clock);
            let sub = ros.subscribe::<ClockMsg, _>("/clock", move |v| clock.trigger(v.clock))
                .chain_err(|| "Failed to subscribe to simulated clock")?;
            ros.static_subs.push(sub);
            ros.clock = ros_clock;
        }

        ros.logger = Some(ros.publish("/rosout")?);

        Ok(ros)
    }

    fn new_raw(master_uri: &str, hostname: &str, namespace: &str, name: &str) -> Result<Ros> {
        let namespace = namespace.trim_right_matches('/');

        if name.contains('/') {
            bail!(ErrorKind::Naming(
                naming::error::ErrorKind::IllegalCharacter(name.into()),
            ));
        }

        let name = format!("{}/{}", namespace, name);
        let resolver = Resolver::new(&name)?;

        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let slave = Slave::new(master_uri, hostname, 0, &name, shutdown_tx.clone())?;
        let master = Master::new(master_uri, &name, slave.uri());

        Ok(Ros {
            master: Arc::new(master),
            slave: Arc::new(slave),
            hostname: String::from(hostname),
            resolver: resolver,
            name: name,
            clock: Arc::new(RealClock::default()),
            static_subs: Vec::new(),
            logger: None,
            shutdown_tx,
            shutdown_rx: Some(shutdown_rx),
        })
    }

    fn map(&mut self, source: &str, destination: &str) -> Result<()> {
        self.resolver.map(source, destination).map_err(|v| v.into())
    }

    #[inline]
    pub fn uri(&self) -> &str {
        self.slave.uri()
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    #[inline]
    pub fn now(&self) -> Time {
        self.clock.now()
    }

    #[inline]
    pub fn sleep(&self, d: Duration) {
        self.clock.await_init();
        self.clock.sleep(d);
    }

    #[inline]
    pub fn shutdown_sender(&self) -> mpsc::Sender<()> {
        self.shutdown_tx.clone()
    }

    pub fn rate(&self, rate: f64) -> Rate {
        self.clock.await_init();
        let nanos = 1_000_000_000.0 / rate;
        Rate::new(Arc::clone(&self.clock), Duration::from_nanos(nanos as i64))
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        if let Some(ref rx) = self.shutdown_rx {
            rx.try_recv() == Err(mpsc::TryRecvError::Empty)
        } else {
            return false;
        }
    }

    #[inline]
    pub fn spin(&mut self) -> Spinner {
        Spinner {
            shutdown_rx: self.shutdown_rx.take(),
        }
    }

    pub fn param(&self, name: &str) -> Option<Parameter> {
        self.resolver.translate(name).ok().map(|v| Parameter {
            master: Arc::clone(&self.master),
            name: v,
        })
    }

    pub fn parameters(&self) -> Response<Vec<String>> {
        self.master.get_param_names()
    }

    pub fn state(&self) -> Response<master::SystemState> {
        self.master.get_system_state().map(Into::into)
    }

    pub fn topics(&self) -> Response<Vec<Topic>> {
        self.master
            .get_topic_types()
            .map(|v| v.into_iter().map(Into::into).collect())
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
            Arc::clone(&self.master),
            Arc::clone(&self.slave),
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
        Subscriber::new::<T, F>(
            Arc::clone(&self.master),
            Arc::clone(&self.slave),
            &name,
            callback,
        )
    }

    pub fn publish<T>(&mut self, topic: &str) -> Result<Publisher<T>>
    where
        T: Message,
    {
        let name = self.resolver.translate(topic)?;
        Publisher::new(
            Arc::clone(&self.master),
            Arc::clone(&self.slave),
            Arc::clone(&self.clock),
            &self.hostname,
            &name,
        )
    }

    pub fn log(&mut self, level: i8, msg: String, file: &str, line: u32) {
        let logger = &mut match self.logger {
            Some(ref mut v) => v,
            None => return,
        };
        let topics = self.slave
            .publications
            .lock()
            .expect("Failed to acquire lock on mutex")
            .keys()
            .cloned()
            .collect();
        let message = Log {
            header: Header::default(),
            level,
            msg,
            name: self.name.clone(),
            line,
            file: file.into(),
            function: String::default(),
            topics,
        };
        if let Err(err) = logger.send(message) {
            error!("Logging error: {}", err);
        }
    }
}

pub struct Parameter {
    master: Arc<Master>,
    name: String,
}

impl Parameter {
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
        Yaml::Array(v) => {
            xml_rpc::Value::Array(v.into_iter().map(yaml_to_xmlrpc).collect::<Result<_>>()?)
        }
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

pub struct Spinner {
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if let Some(ref rx) = self.shutdown_rx {
            rx.recv().is_ok();
        }
    }
}

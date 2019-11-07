use super::super::rosxmlrpc::Response;
use super::clock::{Clock, Rate, RealClock, SimulatedClock};
use super::error::{ErrorKind, Result, ResultExt};
use super::master::{self, Master, Topic};
use super::naming::{self, Resolver};
use super::raii::{Publisher, Service, Subscriber};
use super::resolve;
use super::slave::Slave;
use crate::api::clock::Delay;
use crate::api::ShutdownManager;
use crate::msg::rosgraph_msgs::{Clock as ClockMsg, Log};
use crate::msg::std_msgs::Header;
use crate::tcpros::{Client, Message, ServicePair, ServiceResult};
use crate::time::{Duration, Time};
use error_chain::bail;
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread::sleep;
use xml_rpc;
use yaml_rust::{Yaml, YamlLoader};

pub struct Ros {
    master: Arc<Master>,
    slave: Arc<Slave>,
    hostname: String,
    bind_address: String,
    resolver: Resolver,
    name: String,
    clock: Arc<dyn Clock>,
    static_subs: Vec<Subscriber>,
    logger: Option<Publisher<Log>>,
    shutdown_manager: Arc<ShutdownManager>,
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
                .chain_err(|| ErrorKind::BadYamlData(dest.clone()))?
                .into_iter()
                .next()
                .ok_or_else(|| ErrorKind::BadYamlData(dest.clone()))?;
            let param = ros
                .param(&src)
                .ok_or_else(|| ErrorKind::CannotResolveName(src))?;
            param.set_raw(yaml_to_xmlrpc(data)?)?;
        }

        if ros
            .param("/use_sim_time")
            .and_then(|v| v.get().ok())
            .unwrap_or(false)
        {
            let clock = Arc::new(SimulatedClock::default());
            let ros_clock = Arc::clone(&clock);
            let sub = ros
                .subscribe::<ClockMsg, _>("/clock", 1, move |v| clock.trigger(v.clock))
                .chain_err(|| {
                    ErrorKind::CommunicationIssue("Failed to subscribe to simulated clock".into())
                })?;
            ros.static_subs.push(sub);
            ros.clock = ros_clock;
        }

        ros.logger = Some(ros.publish("/rosout", 100)?);

        Ok(ros)
    }

    fn new_raw(master_uri: &str, hostname: &str, namespace: &str, name: &str) -> Result<Ros> {
        let namespace = namespace.trim_end_matches('/');

        if name.contains('/') {
            bail!(ErrorKind::Naming(
                naming::error::ErrorKind::IllegalCharacter(name.into()),
            ));
        }

        let bind_host = {
            if hostname == "localhost" || hostname.starts_with("127.") {
                hostname
            } else {
                "0.0.0.0"
            }
        };

        let name = format!("{}/{}", namespace, name);
        let resolver = Resolver::new(&name)?;

        let shutdown_manager = Arc::new(ShutdownManager::default());

        let slave = Slave::new(
            master_uri,
            hostname,
            bind_host,
            0,
            &name,
            Arc::clone(&shutdown_manager),
        )?;
        let master = Master::new(master_uri, &name, slave.uri())?;

        Ok(Ros {
            master: Arc::new(master),
            slave: Arc::new(slave),
            hostname: String::from(hostname),
            bind_address: String::from(bind_host),
            resolver,
            name,
            clock: Arc::new(RealClock::default()),
            static_subs: Vec::new(),
            logger: None,
            shutdown_manager,
        })
    }

    fn map(&mut self, source: &str, destination: &str) -> Result<()> {
        self.resolver.map(source, destination).map_err(Into::into)
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
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    #[inline]
    pub fn now(&self) -> Time {
        self.clock.now()
    }

    #[inline]
    pub fn delay(&self, d: Duration) -> Delay {
        self.clock.await_init();
        Delay::new(Arc::clone(&self.clock), d)
    }

    #[inline]
    pub fn shutdown_sender(&self) -> Arc<ShutdownManager> {
        Arc::clone(&self.shutdown_manager)
    }

    pub fn rate(&self, rate: f64) -> Rate {
        self.clock.await_init();
        let nanos = 1_000_000_000.0 / rate;
        Rate::new(Arc::clone(&self.clock), Duration::from_nanos(nanos as i64))
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        !self.shutdown_manager.awaiting_shutdown()
    }

    #[inline]
    pub fn spin(&self) -> Spinner {
        Spinner {
            shutdown_manager: Arc::clone(&self.shutdown_manager),
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

    pub fn wait_for_service(
        &self,
        service: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<()> {
        use crate::rosxmlrpc::ResponseError;

        let name = self.resolver.translate(service)?;
        let now = std::time::Instant::now();
        loop {
            let e = match self.master.lookup_service(&name) {
                Ok(_) => return Ok(()),
                Err(e) => e,
            };
            match e {
                ResponseError::Client(ref m) if m == "no provider" => {
                    if let Some(ref timeout) = timeout {
                        if now.elapsed() > *timeout {
                            return Err(ErrorKind::TimeoutError.into());
                        }
                    }
                    sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                _ => {}
            }
            return Err(e.into());
        }
    }

    pub fn service<T, F>(&self, service: &str, handler: F) -> Result<Service>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        let name = self.resolver.translate(service)?;
        Service::new::<T, F>(
            Arc::clone(&self.master),
            Arc::clone(&self.slave),
            &self.hostname,
            &self.bind_address,
            &name,
            handler,
        )
    }

    #[inline]
    pub fn subscribe<T, F>(&self, topic: &str, queue_size: usize, callback: F) -> Result<Subscriber>
    where
        T: Message,
        F: Fn(T) + Send + 'static,
    {
        self.subscribe_with_ids(topic, queue_size, move |data, _| callback(data))
    }

    pub fn subscribe_with_ids<T, F>(
        &self,
        topic: &str,
        mut queue_size: usize,
        callback: F,
    ) -> Result<Subscriber>
    where
        T: Message,
        F: Fn(T, &str) + Send + 'static,
    {
        if queue_size == 0 {
            queue_size = usize::max_value();
        }
        let name = self.resolver.translate(topic)?;
        Subscriber::new::<T, F>(
            Arc::clone(&self.master),
            Arc::clone(&self.slave),
            &name,
            queue_size,
            callback,
        )
    }

    pub fn publish<T>(&self, topic: &str, mut queue_size: usize) -> Result<Publisher<T>>
    where
        T: Message,
    {
        if queue_size == 0 {
            queue_size = usize::max_value();
        }
        let name = self.resolver.translate(topic)?;
        Publisher::new(
            Arc::clone(&self.master),
            Arc::clone(&self.slave),
            Arc::clone(&self.clock),
            &self.bind_address,
            &name,
            queue_size,
        )
    }

    fn log_to_terminal(&self, level: i8, msg: &str, file: &str, line: u32) {
        use colored::{Color, Colorize};

        let format_string =
            |prefix, color| format!("[{} @ {}:{}]: {}", prefix, file, line, msg).color(color);

        match level {
            Log::DEBUG => println!("{}", format_string("DEBUG", Color::White)),
            Log::INFO => println!("{}", format_string("INFO", Color::White)),
            Log::WARN => eprintln!("{}", format_string("WARN", Color::Yellow)),
            Log::ERROR => eprintln!("{}", format_string("ERROR", Color::Red)),
            Log::FATAL => eprintln!("{}", format_string("FATAL", Color::Red)),
            _ => {}
        }
    }

    pub fn log(&self, level: i8, msg: String, file: &str, line: u32) {
        self.log_to_terminal(level, &msg, file, line);
        let logger = &match self.logger {
            Some(ref v) => v,
            None => return,
        };
        let topics = self.slave.publications.get_topic_names();
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
        Yaml::Real(v) => xml_rpc::Value::Double(
            v.parse()
                .chain_err(|| ErrorKind::BadYamlData("Failed to parse float".into()))?,
        ),
        Yaml::Integer(v) => xml_rpc::Value::Int(v as i32),
        Yaml::String(v) => xml_rpc::Value::String(v),
        Yaml::Boolean(v) => xml_rpc::Value::Bool(v),
        Yaml::Array(v) => {
            xml_rpc::Value::Array(v.into_iter().map(yaml_to_xmlrpc).collect::<Result<_>>()?)
        }
        Yaml::Hash(v) => xml_rpc::Value::Struct(
            v.into_iter()
                .map(|(k, v)| Ok((yaml_to_string(k)?, yaml_to_xmlrpc(v)?)))
                .collect::<Result<_>>()?,
        ),
        Yaml::Alias(_) => bail!(ErrorKind::BadYamlData("Alias is not supported".into())),
        Yaml::Null => bail!(ErrorKind::BadYamlData("Illegal null value".into())),
        Yaml::BadValue => bail!(ErrorKind::BadYamlData("Bad value provided".into())),
    })
}

fn yaml_to_string(val: Yaml) -> Result<String> {
    Ok(match val {
        Yaml::Real(v) | Yaml::String(v) => v,
        Yaml::Integer(v) => v.to_string(),
        Yaml::Boolean(true) => "true".into(),
        Yaml::Boolean(false) => "false".into(),
        _ => bail!(ErrorKind::BadYamlData(
            "Hash keys need to be strings".into()
        )),
    })
}

pub struct Spinner {
    shutdown_manager: Arc<ShutdownManager>,
}

impl Drop for Spinner {
    fn drop(&mut self) {
        while !self.shutdown_manager.awaiting_shutdown() {
            sleep(std::time::Duration::from_millis(100));
        }
    }
}

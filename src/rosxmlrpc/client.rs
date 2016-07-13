extern crate hyper;
extern crate xml;

use std::io::Read;

pub struct Client {
    http_client: hyper::Client,
    server_uri: String,
}

impl Client {
    pub fn new(server_uri: String) -> Client {
        Client {
            http_client: hyper::Client::new(),
            server_uri: server_uri,
        }
    }

    pub fn request(&self, function_name: String, parameters: &Vec<(String, String)>) -> String {
        let mut body = Vec::<u8>::new();
        {
            let mut writer = xml::EventWriter::new(&mut body);
            writer.write(xml::writer::XmlEvent::start_element("methodCall")).unwrap();
            writer.write(xml::writer::XmlEvent::start_element("methodName")).unwrap();
            writer.write(xml::writer::XmlEvent::characters(function_name.as_str())).unwrap();
            writer.write(xml::writer::XmlEvent::end_element()).unwrap();
            writer.write(xml::writer::XmlEvent::start_element("params")).unwrap();
            for param in parameters {
                writer.write(xml::writer::XmlEvent::start_element(param.0.as_str())).unwrap();
                writer.write(xml::writer::XmlEvent::start_element("value")).unwrap();
                writer.write(xml::writer::XmlEvent::start_element("string")).unwrap();
                writer.write(xml::writer::XmlEvent::characters(param.1.as_str())).unwrap();
                writer.write(xml::writer::XmlEvent::end_element()).unwrap();
                writer.write(xml::writer::XmlEvent::end_element()).unwrap();
                writer.write(xml::writer::XmlEvent::end_element()).unwrap();
            }
            writer.write(xml::writer::XmlEvent::end_element()).unwrap();
            writer.write(xml::writer::XmlEvent::end_element()).unwrap();
        }

        let mut res = self.http_client
            .post(self.server_uri.as_str())
            .body(String::from_utf8(body).unwrap().as_str())
            .send()
            .unwrap();

        let mut buf = String::new();
        res.read_to_string(&mut buf).unwrap();
        buf
    }
}

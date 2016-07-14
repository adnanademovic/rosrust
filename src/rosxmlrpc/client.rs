extern crate hyper;
extern crate xml;

use std::io::Read;

pub struct Client<'t> {
    http_client: hyper::Client,
    server_uri: &'t str,
}

impl<'t> Client<'t> {
    pub fn new(server_uri: &'t str) -> Client<'t> {
        Client {
            http_client: hyper::Client::new(),
            server_uri: server_uri,
        }
    }

    pub fn request(&self, function_name: &str, parameters: &Vec<&str>) -> Member {
        let mut body = Vec::<u8>::new();
        {
            let mut writer = xml::EventWriter::new(&mut body);
            writer.write(xml::writer::XmlEvent::start_element("methodCall")).unwrap();
            writer.write(xml::writer::XmlEvent::start_element("methodName")).unwrap();
            writer.write(xml::writer::XmlEvent::characters(function_name)).unwrap();
            writer.write(xml::writer::XmlEvent::end_element()).unwrap();
            writer.write(xml::writer::XmlEvent::start_element("params")).unwrap();
            for param in parameters {
                writer.write(xml::writer::XmlEvent::start_element("value")).unwrap();
                writer.write(xml::writer::XmlEvent::start_element("string")).unwrap();
                writer.write(xml::writer::XmlEvent::characters(param)).unwrap();
                writer.write(xml::writer::XmlEvent::end_element()).unwrap();
                writer.write(xml::writer::XmlEvent::end_element()).unwrap();
            }
            writer.write(xml::writer::XmlEvent::end_element()).unwrap();
            writer.write(xml::writer::XmlEvent::end_element()).unwrap();
        }

        let res = self.http_client
            .post(self.server_uri)
            .body(String::from_utf8(body).unwrap().as_str())
            .send()
            .unwrap();

        let xml_tree = read_xml_tree(&mut xml::EventReader::new(res)).unwrap();
        parse_xml_tree(&xml_tree).unwrap()
    }
}

fn read_xml_tree<T: Read>(mut parser: &mut xml::EventReader<T>) -> Option<XmlTreeNode> {
    match parser.next() {
        Ok(xml::reader::XmlEvent::StartElement { name, .. }) => {
            let mut children = Vec::<XmlTreeNode>::new();
            while let Some(tree) = read_xml_tree(&mut parser) {
                children.push(tree);
            }
            Some(XmlTreeNode::Node(name.local_name, children))
        }
        Ok(xml::reader::XmlEvent::Characters(value)) => Some(XmlTreeNode::Leaf(value)),
        Ok(xml::reader::XmlEvent::EndElement { .. }) => None,
        Err(..) => None,
        _ => read_xml_tree(&mut parser),
    }
}

fn parse_xml_tree(tree: &XmlTreeNode) -> Option<Member> {
    if let Some(tree) = unwrap_xml_layer(&tree, "methodResponse".to_string()) {
        if let Some(tree) = unwrap_xml_layer(&tree, "params".to_string()) {
            if let Some(tree) = unwrap_xml_layer(&tree, "param".to_string()) {
                return parse_xml_tree_helper(&tree);
            }
        }
    }
    None
}

fn parse_xml_tree_helper(tree: &XmlTreeNode) -> Option<Member> {
    if let Some(tree) = unwrap_xml_layer(&tree, "value".to_string()) {
        if let XmlTreeNode::Node(ref name, ref children) = *tree {
            if children.len() == 1 {
                let child = &children[0];
                return match name.as_str() {
                    "array" => parse_xml_array(child),
                    "string" => parse_xml_string(child),
                    "int" | "i4" => parse_xml_int(child),
                    _ => None,
                };
            }
        }
    }
    None
}

fn parse_xml_array(tree: &XmlTreeNode) -> Option<Member> {
    if let XmlTreeNode::Node(ref name, ref children) = *tree {
        if name.eq(&"data".to_string()) {
            return Some(Member::Array(children.iter().filter_map(parse_xml_tree_helper).collect()));
        }
    }
    None
}

fn parse_xml_int(tree: &XmlTreeNode) -> Option<Member> {
    if let Some(Member::String(text)) = parse_xml_string(&tree) {
        if let Ok(value) = text.parse::<i32>() {
            return Some(Member::Int(value));
        }
    }
    None
}

fn parse_xml_string(tree: &XmlTreeNode) -> Option<Member> {
    if let XmlTreeNode::Leaf(ref value) = *tree {
        return Some(Member::String(value.clone()));
    }
    None
}

fn unwrap_xml_layer(tree: &XmlTreeNode, node_name: String) -> Option<&XmlTreeNode> {
    if let XmlTreeNode::Node(ref name, ref children) = *tree {
        if name.eq(&node_name) && children.len() == 1 {
            return Some(&children[0]);
        }
    }
    None
}

enum XmlTreeNode {
    Leaf(String),
    Node(String, Vec<XmlTreeNode>),
}

pub enum Member {
    Array(Vec<Member>),
    String(String),
    Int(i32),
}

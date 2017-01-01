use std;
use xml;
use self::error::{ErrorKind, Result, ResultExt};

#[derive(Clone,Debug,PartialEq)]
pub enum XmlRpcValue {
    Int(i32),
    Bool(bool),
    String(String),
    Double(f64),
    Array(Vec<XmlRpcValue>),
    Struct(Vec<(String, XmlRpcValue)>),
}

#[derive(Debug)]
pub struct XmlRpcRequest {
    pub method: String,
    pub parameters: Vec<XmlRpcValue>,
}

#[derive(Debug)]
pub struct XmlRpcResponse {
    pub parameters: Vec<XmlRpcValue>,
}

impl std::fmt::Display for XmlRpcValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            XmlRpcValue::Int(v) => write!(f, "<value><i4>{}</i4></value>", v),
            XmlRpcValue::Bool(v) => {
                write!(f,
                       "<value><boolean>{}</boolean></value>",
                       if v { 1 } else { 0 })
            }
            XmlRpcValue::String(ref v) => write!(f, "<value><string>{}</string></value>", v),
            XmlRpcValue::Double(v) => write!(f, "<value><double>{}</double></value>", v),
            XmlRpcValue::Array(ref v) => {
                write!(f, "<value><array><data>")?;
                for item in v {
                    item.fmt(f)?;
                }
                write!(f, "</data></array></value>")
            }
            XmlRpcValue::Struct(ref v) => {
                write!(f, "<value><struct>")?;
                for &(ref name, ref item) in v {
                    write!(f, "<member><name>{}</name>", name)?;
                    item.fmt(f)?;
                    write!(f, "</member>")?;
                }
                write!(f, "</struct></value>")
            }
        }
    }
}

impl std::fmt::Display for XmlRpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
               "<?xml version=\"1.0\"?><methodCall><methodName>{}</methodName><params>",
               self.method)?;
        for parameter in &self.parameters {
            write!(f, "<param>{}</param>", parameter)?;
        }
        write!(f, "</params></methodCall>")
    }
}

impl std::fmt::Display for XmlRpcResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<?xml version=\"1.0\"?><methodResponse><params>")?;
        for parameter in &self.parameters {
            write!(f, "<param>{}</param>", parameter)?;
        }
        write!(f, "</params></methodResponse>")
    }
}

impl XmlRpcRequest {
    pub fn new<T: std::io::Read>(body: T) -> Result<XmlRpcRequest> {
        let tree = Tree::new(body).chain_err(|| "Failed to transform XML data into tree")?;
        let (key, mut children) = match tree {
            Tree::Node(key, children) => (key, children),
            Tree::Leaf(_) => {
                bail!("XML-RPC request should contain a node called 'methodCall' with two children")
            }
        };
        if key != "methodCall" || children.len() != 2 {
            bail!("XML-RPC request should contain a node called 'methodCall' with two children")
        }
        // We checked the array length, so it's safe to pop
        let parameters_tree = children.pop().unwrap();
        let method = children.pop().unwrap();
        match method.peel_layer("methodName")
            .chain_err(|| "Bad XML-RPC Request method name value")? {
            Tree::Leaf(method_name) => {
                Ok(XmlRpcRequest {
                    method: method_name,
                    parameters: extract_parameters(parameters_tree)?,
                })
            }
            Tree::Node(_, _) => {
                bail!("Node 'methodName' should just contain a string representing the method name")
            }
        }
    }
}

impl XmlRpcResponse {
    pub fn new<T: std::io::Read>(body: T) -> Result<XmlRpcResponse> {
        extract_parameters(Tree::new(body).chain_err(|| "Failed to transform XML data into tree")?
                .peel_layer("methodResponse")?)
            .map(|parameters| XmlRpcResponse { parameters: parameters })
    }
}

fn extract_parameters(parameters: Tree) -> Result<Vec<XmlRpcValue>> {
    if let Tree::Node(key, parameters) = parameters {
        if key == "params" {
            return parameters.into_iter()
                .map(XmlRpcValue::from_parameter)
                .collect::<Result<_>>()
                .chain_err(|| "Failed to parse parameters");
        }
    }
    bail!("Parameters need to be contained within a node called params")
}

impl XmlRpcValue {
    pub fn new<T: std::io::Read>(body: T) -> Result<XmlRpcValue> {
        XmlRpcValue::from_tree(Tree::new(body)
                               .chain_err(|| "Couldn't generate XML tree to form value")?)
    }

    fn read_member(tree: Tree) -> Result<(String, XmlRpcValue)> {
        let (key, mut children) = match tree {
            Tree::Node(key, children) => (key, children),
            Tree::Leaf(_) => {
                bail!("Structure member node should contain a node called 'member' with two \
                       children")
            }
        };
        if key != "member" || children.len() != 2 {
            bail!("Structure member node should contain a node called 'member' with two children")
        }
        // We tested the vector length already, so it's safe to pop
        let value = children.pop().unwrap();
        let name_node = children.pop().unwrap();
        let name = match name_node.peel_layer("name")
            .chain_err(|| "First struct member field should be a node called 'name'")? {
            Tree::Leaf(name) => name,
            Tree::Node(_, _) => {
                bail!("Struct member's name node should just contain the member's name")
            }
        };

        XmlRpcValue::from_tree(value)
            .chain_err(|| format!("Failed to parse subtree of struct field {}", name))
            .map(|v| (name, v))
    }

    fn from_parameter(tree: Tree) -> Result<XmlRpcValue> {
        XmlRpcValue::from_tree(tree.peel_layer("param")
                .chain_err(|| "Parameters should be contained within node 'param'")?)
            .chain_err(|| "Failed to parse XML RPC parameter")
    }

    fn from_tree(tree: Tree) -> Result<XmlRpcValue> {
        let (key, mut values) = match tree.peel_layer("value")? {
            Tree::Node(key, values) => (key, values),
            Tree::Leaf(_) => bail!("Value node should contain one node representing its data type"),
        };
        if key == "struct" {
            return Ok(XmlRpcValue::Struct(values.into_iter()
                .map(XmlRpcValue::read_member)
                .collect::<Result<Vec<(String, XmlRpcValue)>>>()
                .chain_err(|| "Couldn't parse struct")?));
        }
        if values.len() > 1 {
            bail!("Node '{}' can't have more than one child", key);
        }
        if key == "array" {
            return if let Some(Tree::Node(key, children)) = values.pop() {
                if key != "data" {
                    bail!("Node 'array' must contain 'data' node, but '{}' detected",
                          key);
                }
                Ok(XmlRpcValue::Array(children.into_iter()
                    .map(XmlRpcValue::from_tree)
                    .collect::<Result<_>>()
                    .chain_err(|| "Failed to parse array's children")?))
            } else {
                bail!("Node 'array' must contain 'data' node with child values");
            };
        }
        let value = match values.pop().unwrap_or(Tree::Leaf(String::from(""))) {
            Tree::Leaf(value) => value,
            Tree::Node(_, _) => bail!("Value field for type '{}' must contain just the value", key),
        };
        match key.as_str() {
            "i4" | "int" => {
                Ok(XmlRpcValue::Int(value.parse()
                    .chain_err(|| format!("Failed to parse integer (i32) {}", value))?))
            }
            "boolean" => {
                Ok(XmlRpcValue::Bool(value.parse::<i32>()
                    .chain_err(|| format!("Expected 0 or 1 for boolean, got {}", value))? !=
                                     0))
            }
            "string" => Ok(XmlRpcValue::String(value)),
            "double" => {
                Ok(XmlRpcValue::Double(value.parse()
                    .chain_err(|| format!("Failed to parse double (f64) {}", value))?))
            }
            _ => bail!("Unsupported data type '{}'", key),
        }
    }
}

enum Tree {
    Leaf(String),
    Node(String, Vec<Tree>),
}

impl Tree {
    fn new<T: std::io::Read>(body: T) -> Result<Tree> {
        parse_tree(&mut xml::EventReader::new(body))
            ?
            .ok_or("XML data started with a closing tag".into())
    }

    fn peel_layer(self, name: &str) -> Result<Tree> {
        if let Tree::Node(key, mut children) = self {
            if key == name && children.len() == 1 {
                // Popping element from a vector of length 1 cannot fail
                return Ok(children.pop().unwrap());
            }
        }
        bail!("Expected a node named '{}' with 1 child", name)
    }
}

enum Node {
    Open(String),
    Data(String),
    Close(String),
}

fn parse_tree<T: std::io::Read>(reader: &mut xml::EventReader<T>) -> Result<Option<Tree>> {
    match next_node(reader).chain_err(|| "Unexpected end of XML data")? {
        Node::Close(..) => Ok(None),
        Node::Data(value) => Ok(Some(Tree::Leaf(value))),
        Node::Open(name) => {
            let mut children = Vec::<Tree>::new();
            while let Some(node) =
                parse_tree(reader).chain_err(|| ErrorKind::TreeParsing(name.clone()))? {
                children.push(node);
            }
            Ok(Some(Tree::Node(name, children)))
        }
    }
}

fn next_node<T: std::io::Read>(reader: &mut xml::EventReader<T>) -> Result<Node> {
    match reader.next().chain_err(|| "Couldn't obtain XML token")? {
        xml::reader::XmlEvent::StartElement { name, .. } => Ok(Node::Open(name.local_name)),
        xml::reader::XmlEvent::Characters(value) => Ok(Node::Data(value)),
        xml::reader::XmlEvent::EndElement { name } => Ok(Node::Close(name.local_name)),
        _ => next_node(reader),
    }
}

mod error {
    error_chain!{
        errors {
            TreeParsing(node_name: String) {
                description("Error while building tree out of XML data")
                display("XML tree building error within node {}", node_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    #[test]
    fn reads_string() {
        let data = r#"<?xml version="1.0"?><value><string>First test</string></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::String(String::from("First test")), value);
        let data = r#"<?xml version="1.0"?><value><string /></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::String(String::from("")), value);
        let data = r#"<?xml version="1.0"?><value><string></string></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::String(String::from("")), value);
    }

    #[test]
    fn reads_int() {
        let data = r#"<?xml version="1.0"?><value><i4>41</i4></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Int(41), value);
        let data = r#"<?xml version="1.0"?><value><int>14</int></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Int(14), value);
    }

    #[test]
    fn reads_float() {
        let data = r#"<?xml version="1.0"?><value><double>33.25</double></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Double(33.25), value);
    }

    #[test]
    fn reads_bool() {
        let data = r#"<?xml version="1.0"?><value><boolean>1</boolean></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Bool(true), value);
        let data = r#"<?xml version="1.0"?><value><boolean>0</boolean></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Bool(false), value);
    }

    #[test]
    fn reads_array() {
        let data = r#"<?xml version="1.0"?>
<value><array><data>
  <value><i4>41</i4></value>
  <value><boolean>1</boolean></value>
  <value><array><data>
    <value><string>Hello</string></value>
    <value><double>0.5</double></value>
  </data></array></value>
</data></array></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                           XmlRpcValue::Bool(true),
                                           XmlRpcValue::Array(vec![
                XmlRpcValue::String(String::from("Hello")),
                XmlRpcValue::Double(0.5),
            ])]),
                   value);
    }

    #[test]
    fn reads_struct() {
        let data = r#"<?xml version="1.0"?>
<value><struct>
  <member>
    <name>a</name>
    <value><i4>41</i4></value>
  </member>
  <member>
    <name>b</name>
    <value><boolean>1</boolean></value>
  </member>
  <member>
    <name>c</name>
    <value><struct>
      <member>
        <name>xxx</name>
        <value><string>Hello</string></value>
      </member>
      <member>
        <name>yyy</name>
        <value><double>0.5</double></value>
      </member>
    </struct></value>
  </member>
</struct></value>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcValue::new(&mut cursor).unwrap();
        assert_eq!(XmlRpcValue::Struct(vec![(String::from("a"), XmlRpcValue::Int(41)),
                                            (String::from("b"), XmlRpcValue::Bool(true)),
                                            (String::from("c"),
                                             XmlRpcValue::Struct(vec![
                (String::from("xxx"), XmlRpcValue::String(String::from("Hello"))),
                (String::from("yyy"), XmlRpcValue::Double(0.5)),
            ]))]),
                   value);
    }

    #[test]
    fn reads_request() {
        let data = r#"<?xml version="1.0"?>
<methodCall>
  <methodName>mytype.mymethod</methodName>
  <params>
    <param>
      <value><i4>33</i4></value>
    </param>
    <param>
      <value><array><data>
        <value><i4>41</i4></value>
        <value><boolean>1</boolean></value>
        <value><array><data>
          <value><string>Hello</string></value>
          <value><double>0.5</double></value>
        </data></array></value>
      </data></array></value>
    </param>
  </params>
</methodCall>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcRequest::new(&mut cursor).unwrap();
        assert_eq!("mytype.mymethod", value.method);
        assert_eq!(vec![XmlRpcValue::Int(33),
                        XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                                XmlRpcValue::Bool(true),
                                                XmlRpcValue::Array(vec![
                    XmlRpcValue::String(String::from("Hello")),
                    XmlRpcValue::Double(0.5),
                ])])],
                   value.parameters);
    }

    #[test]
    fn reads_response() {
        let data = r#"<?xml version="1.0"?>
<methodResponse>
  <params>
    <param>
      <value><i4>33</i4></value>
    </param>
    <param>
      <value><array><data>
        <value><i4>41</i4></value>
        <value><boolean>1</boolean></value>
        <value><array><data>
          <value><string>Hello</string></value>
          <value><double>0.5</double></value>
        </data></array></value>
      </data></array></value>
    </param>
  </params>
</methodResponse>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let value = XmlRpcResponse::new(&mut cursor).unwrap();
        assert_eq!(vec![XmlRpcValue::Int(33),
                        XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                                XmlRpcValue::Bool(true),
                                                XmlRpcValue::Array(vec![
                    XmlRpcValue::String(String::from("Hello")),
                    XmlRpcValue::Double(0.5),
                ])])],
                   value.parameters);
    }

    #[test]
    fn writes_string() {
        assert_eq!(r#"<value><string>First test</string></value>"#,
                   format!("{}", XmlRpcValue::String(String::from("First test"))));
        assert_eq!(r#"<value><string></string></value>"#,
                   format!("{}", XmlRpcValue::String(String::from(""))));
    }

    #[test]
    fn writes_int() {
        assert_eq!(r#"<value><i4>41</i4></value>"#,
                   format!("{}", XmlRpcValue::Int(41)));
    }

    #[test]
    fn writes_float() {
        assert_eq!(r#"<value><double>33.25</double></value>"#,
                   format!("{}", XmlRpcValue::Double(33.25)));
    }

    #[test]
    fn writes_bool() {
        assert_eq!(r#"<value><boolean>1</boolean></value>"#,
                   format!("{}", XmlRpcValue::Bool(true)));
        assert_eq!(r#"<value><boolean>0</boolean></value>"#,
                   format!("{}", XmlRpcValue::Bool(false)));
    }

    #[test]
    fn writes_array() {
        assert_eq!(concat!(r#"<value><array><data>"#,
                           r#"<value><i4>41</i4></value>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"<value><array><data>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"</data></array></value>"#,
                           r#"</data></array></value>"#),
                   format!("{}",
                           XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                                   XmlRpcValue::Bool(true),
                                                   XmlRpcValue::Array(vec![
                                   XmlRpcValue::String(String::from("Hello")),
                                   XmlRpcValue::Double(0.5),
                               ])])));
    }

    #[test]
    fn writes_struct() {
        assert_eq!(concat!(r#"<value><struct>"#,
                           r#"<member>"#,
                           r#"<name>a</name>"#,
                           r#"<value><i4>41</i4></value>"#,
                           r#"</member>"#,
                           r#"<member>"#,
                           r#"<name>b</name>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"</member>"#,
                           r#"<member>"#,
                           r#"<name>c</name>"#,
                           r#"<value><struct>"#,
                           r#"<member>"#,
                           r#"<name>xxx</name>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"</member>"#,
                           r#"<member>"#,
                           r#"<name>yyy</name>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"</member>"#,
                           r#"</struct></value>"#,
                           r#"</member>"#,
                           r#"</struct></value>"#),
                   format!("{}",
                           XmlRpcValue::Struct(vec![(String::from("a"), XmlRpcValue::Int(41)),
                                                    (String::from("b"),
                                                     XmlRpcValue::Bool(true)),
                                                    (String::from("c"),
                                                     XmlRpcValue::Struct(vec![
                           (String::from("xxx"), XmlRpcValue::String(String::from("Hello"))),
                           (String::from("yyy"), XmlRpcValue::Double(0.5)),
                       ]))])));
    }

    #[test]
    fn writes_request() {
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodCall>"#,
                           r#"<methodName>mytype.mymethod</methodName>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><i4>33</i4></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>41</i4></value>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"<value><array><data>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"</data></array></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodCall>"#),
                   format!("{}",
                           XmlRpcRequest {
                               method: String::from("mytype.mymethod"),
                               parameters: vec![XmlRpcValue::Int(33),
                                                XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                                                        XmlRpcValue::Bool(true),
                                                                        XmlRpcValue::Array(vec![
                                           XmlRpcValue::String(
                                               String::from("Hello")),
                                           XmlRpcValue::Double(0.5),
                                       ])])],
                           }));
    }

    #[test]
    fn writes_response() {
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><i4>33</i4></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>41</i4></value>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"<value><array><data>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"</data></array></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   format!("{}",
                           XmlRpcResponse {
                               parameters: vec![XmlRpcValue::Int(33),
                                                XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                                                        XmlRpcValue::Bool(true),
                                                                        XmlRpcValue::Array(vec![
                                           XmlRpcValue::String(
                                               String::from("Hello")),
                                           XmlRpcValue::Double(0.5),
                                       ])])],
                           }));
    }
}

use std;
use std::error::Error;
use xml;

#[derive(Clone,Debug,PartialEq)]
pub enum XmlRpcValue {
    Int(i32),
    Bool(bool),
    String(String),
    Double(f64),
    Array(Vec<XmlRpcValue>),
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
    pub fn new<T: std::io::Read>(body: T) -> Result<XmlRpcRequest, DecodeError> {
        if let Tree::Node(key, mut children) = Tree::new(body)? {
            if key != "methodCall" || children.len() != 2 {
                return Err(DecodeError::BadXmlStructure);
            }
            let parameters = children.pop().ok_or(DecodeError::BadXmlStructure)?;
            let method =
                children.pop().ok_or(DecodeError::BadXmlStructure)?.peel_layer("methodName")?;
            if let Tree::Leaf(method) = method {
                if let Tree::Node(key, parameters) = parameters {
                    if key != "params" {
                        return Err(DecodeError::BadXmlStructure);
                    }
                    return Ok(XmlRpcRequest {
                        method: method,
                        parameters: parameters.into_iter()
                            .map(|v| XmlRpcValue::from_tree(v.peel_layer("param")?))
                            .collect::<Result<_, _>>()?,
                    });
                }
            }
        }
        Err(DecodeError::BadXmlStructure)
    }
}

impl XmlRpcResponse {
    pub fn new<T: std::io::Read>(body: T) -> Result<XmlRpcResponse, DecodeError> {
        let parameters = Tree::new(body)?.peel_layer("methodResponse")?;
        if let Tree::Node(key, parameters) = parameters {
            if key != "params" {
                return Err(DecodeError::BadXmlStructure);
            }
            return Ok(XmlRpcResponse {
                parameters: parameters.into_iter()
                    .map(|v| XmlRpcValue::from_tree(v.peel_layer("param")?))
                    .collect::<Result<_, _>>()?,
            });
        }
        return Err(DecodeError::BadXmlStructure);
    }
}

impl XmlRpcValue {
    pub fn new<T: std::io::Read>(body: T) -> Result<XmlRpcValue, DecodeError> {
        XmlRpcValue::from_tree(Tree::new(body)?)
    }

    fn from_tree(tree: Tree) -> Result<XmlRpcValue, DecodeError> {
        if let Ok(Tree::Node(key, mut values)) = tree.peel_layer("value") {
            if values.len() > 1 {
                return Err(DecodeError::BadXmlStructure);
            }
            if key == "array" {
                return if let Some(Tree::Node(key, children)) = values.pop() {
                    if key != "data" {
                        return Err(DecodeError::BadXmlStructure);
                    }
                    Ok(XmlRpcValue::Array(children.into_iter()
                        .map(XmlRpcValue::from_tree)
                        .collect::<Result<_, _>>()?))
                } else {
                    Err(DecodeError::BadXmlStructure)
                };
            }
            let value = values.pop().unwrap_or(Tree::Leaf(String::from("")));
            if let Tree::Leaf(value) = value {
                return match key.as_str() {
                    "i4" | "int" => Ok(XmlRpcValue::Int(value.parse()?)),
                    "boolean" => Ok(XmlRpcValue::Bool(value.parse::<i32>()? != 0)),
                    "string" => Ok(XmlRpcValue::String(value)),
                    "double" => Ok(XmlRpcValue::Double(value.parse()?)),
                    _ => Err(DecodeError::UnsupportedDataFormat),
                };
            }
        }
        return Err(DecodeError::BadXmlStructure);
    }
}

enum Tree {
    Leaf(String),
    Node(String, Vec<Tree>),
}

impl Tree {
    fn new<T: std::io::Read>(body: T) -> Result<Tree, DecodeError> {
        parse_tree(&mut xml::EventReader::new(body))?.ok_or(DecodeError::BadXmlStructure)
    }

    fn peel_layer(self, name: &str) -> Result<Tree, DecodeError> {
        if let Tree::Node(key, mut children) = self {
            if key == name && children.len() == 1 {
                return children.pop().ok_or(DecodeError::BadXmlStructure);
            }
        }
        Err(DecodeError::BadXmlStructure)
    }
}

enum Node {
    Open(String),
    Data(String),
    Close(String),
}

fn parse_tree<T: std::io::Read>(reader: &mut xml::EventReader<T>)
                                -> Result<Option<Tree>, DecodeError> {
    match next_node(reader)? {
        Node::Close(..) => Ok(None),
        Node::Data(value) => Ok(Some(Tree::Leaf(value))),
        Node::Open(name) => {
            Ok(Some(Tree::Node(name, {
                let mut children = Vec::<Tree>::new();
                while let Some(node) = parse_tree(reader)? {
                    children.push(node);
                }
                children
            })))
        }
    }
}

fn next_node<T: std::io::Read>(reader: &mut xml::EventReader<T>) -> Result<Node, DecodeError> {
    match reader.next()? {
        xml::reader::XmlEvent::StartElement { name, .. } => Ok(Node::Open(name.local_name)),
        xml::reader::XmlEvent::Characters(value) => Ok(Node::Data(value)),
        xml::reader::XmlEvent::EndElement { name } => Ok(Node::Close(name.local_name)),
        _ => next_node(reader),
    }
}

#[derive(Debug)]
pub enum DecodeError {
    BadXmlStructure,
    XmlRead(xml::reader::Error),
    UnsupportedDataFormat,
}

impl From<xml::reader::Error> for DecodeError {
    fn from(err: xml::reader::Error) -> DecodeError {
        DecodeError::XmlRead(err)
    }
}

impl From<std::num::ParseIntError> for DecodeError {
    fn from(_: std::num::ParseIntError) -> DecodeError {
        DecodeError::BadXmlStructure
    }
}

impl From<std::num::ParseFloatError> for DecodeError {
    fn from(_: std::num::ParseFloatError) -> DecodeError {
        DecodeError::BadXmlStructure
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DecodeError::XmlRead(ref err) => write!(f, "XML reading error: {}", err),
            DecodeError::BadXmlStructure |
            DecodeError::UnsupportedDataFormat => write!(f, "{}", self.description()),
        }
    }
}

impl std::error::Error for DecodeError {
    fn description(&self) -> &str {
        match *self {
            DecodeError::BadXmlStructure => "XML data provided didn't have XML-RPC format",
            DecodeError::XmlRead(ref err) => err.description(),
            DecodeError::UnsupportedDataFormat => {
                "Data provided within XML-RPC call is not supported"
            }
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            DecodeError::XmlRead(ref err) => Some(err),
            DecodeError::BadXmlStructure |
            DecodeError::UnsupportedDataFormat => None,
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
        assert_eq!(XmlRpcValue::Array(vec![
            XmlRpcValue::Int(41),
            XmlRpcValue::Bool(true),
            XmlRpcValue::Array(vec![
                XmlRpcValue::String(String::from("Hello")),
                XmlRpcValue::Double(0.5),
            ]),
        ]),
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
        assert_eq!(vec![
            XmlRpcValue::Int(33),
            XmlRpcValue::Array(vec![
                XmlRpcValue::Int(41),
                XmlRpcValue::Bool(true),
                XmlRpcValue::Array(vec![
                    XmlRpcValue::String(String::from("Hello")),
                    XmlRpcValue::Double(0.5),
                ]),
            ]),
        ],
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
        assert_eq!(vec![
            XmlRpcValue::Int(33),
            XmlRpcValue::Array(vec![
                XmlRpcValue::Int(41),
                XmlRpcValue::Bool(true),
                XmlRpcValue::Array(vec![
                    XmlRpcValue::String(String::from("Hello")),
                    XmlRpcValue::Double(0.5),
                ]),
            ]),
        ],
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
                           XmlRpcValue::Array(vec![
                               XmlRpcValue::Int(41),
                               XmlRpcValue::Bool(true),
                               XmlRpcValue::Array(vec![
                                   XmlRpcValue::String(String::from("Hello")),
                                   XmlRpcValue::Double(0.5),
                               ]),
                           ])));
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
                               parameters: vec![
                                   XmlRpcValue::Int(33),
                                   XmlRpcValue::Array(vec![
                                       XmlRpcValue::Int(41),
                                       XmlRpcValue::Bool(true),
                                       XmlRpcValue::Array(vec![
                                           XmlRpcValue::String(
                                               String::from("Hello")),
                                           XmlRpcValue::Double(0.5),
                                       ]),
                                   ]),
                               ],
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
                               parameters: vec![
                                   XmlRpcValue::Int(33),
                                   XmlRpcValue::Array(vec![
                                       XmlRpcValue::Int(41),
                                       XmlRpcValue::Bool(true),
                                       XmlRpcValue::Array(vec![
                                           XmlRpcValue::String(
                                               String::from("Hello")),
                                           XmlRpcValue::Double(0.5),
                                       ]),
                                   ]),
                               ],
                           }));
    }
}

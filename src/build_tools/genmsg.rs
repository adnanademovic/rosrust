use regex::Regex;

static IGNORE_WHITESPACE: &'static str = r"\s*";
static ANY_WHITESPACE: &'static str = r"\s+";
static FIELD_TYPE: &'static str = r"([a-zA-Z0-9_/]+)";
static FIELD_NAME: &'static str = r"([a-zA-Z][a-zA-Z0-9_]*)";
static EMPTY_BRACKETS: &'static str = r"\[\s*\]";
static NUMBER_BRACKETS: &'static str = r"\[\s*([0-9]+)\s*\]";

fn match_field(data: &str) -> Option<FieldLine> {
    lazy_static! {
        static ref MATCHER: String = format!("^{}{}{}$", FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some(FieldLine {
        field_type: captures.get(1).unwrap().as_str().into(),
        field_name: captures.get(2).unwrap().as_str().into(),
    })
}

fn match_vector_field(data: &str) -> Option<FieldLine> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}$", FIELD_TYPE, IGNORE_WHITESPACE, EMPTY_BRACKETS, ANY_WHITESPACE,
            FIELD_NAME);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some(FieldLine {
        field_type: captures.get(1).unwrap().as_str().into(),
        field_name: captures.get(2).unwrap().as_str().into(),
    })
}

fn match_array_field(data: &str) -> Option<(FieldLine, usize)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}$", FIELD_TYPE, IGNORE_WHITESPACE, NUMBER_BRACKETS, ANY_WHITESPACE,
            FIELD_NAME);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((FieldLine {
              field_type: captures.get(1).unwrap().as_str().into(),
              field_name: captures.get(3).unwrap().as_str().into(),
          },
          captures.get(2).unwrap().as_str().parse().unwrap()))
}

fn match_const_string(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^(string){}{}{}={}(.*)$", ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE,
            IGNORE_WHITESPACE);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((FieldLine {
              field_type: captures.get(1).unwrap().as_str().into(),
              field_name: captures.get(2).unwrap().as_str().into(),
          },
          captures.get(3).unwrap().as_str().into()))
}

fn match_const_numeric(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^{}{}{}{}={}(-?[0-9]+)$", FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME,
            IGNORE_WHITESPACE, IGNORE_WHITESPACE);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((FieldLine {
              field_type: captures.get(1).unwrap().as_str().into(),
              field_name: captures.get(2).unwrap().as_str().into(),
          },
          captures.get(3).unwrap().as_str().into()))
}

fn match_line(data: &str) -> Result<Option<FieldInfo>, ()> {
    if let Some((info, data)) = match_const_string(data.trim()) {
        return Ok(Some(FieldInfo {
            datatype: info.field_type,
            name: info.field_name,
            case: FieldCase::Const(data),
        }));
    }
    let data = data.splitn(2, '#').next().unwrap().trim();
    if data == "" {
        return Ok(None);
    }
    if let Some(info) = match_field(data) {
        return Ok(Some(FieldInfo {
            datatype: info.field_type,
            name: info.field_name,
            case: FieldCase::Unit,
        }));
    }
    if let Some(info) = match_vector_field(data) {
        return Ok(Some(FieldInfo {
            datatype: info.field_type,
            name: info.field_name,
            case: FieldCase::Vector,
        }));
    }
    if let Some((info, count)) = match_array_field(data) {
        return Ok(Some(FieldInfo {
            datatype: info.field_type,
            name: info.field_name,
            case: FieldCase::Array(count),
        }));
    }
    if let Some((info, data)) = match_const_numeric(data) {
        return Ok(Some(FieldInfo {
            datatype: info.field_type,
            name: info.field_name,
            case: FieldCase::Const(data),
        }));
    }
    Err(())
}

#[inline]
fn match_lines(data: &str) -> Result<Vec<FieldInfo>, ()> {
    let lines: Result<Vec<Option<_>>, ()> = data.split('\n').map(match_line).collect();
    Ok(lines?.into_iter().filter_map(|v| v).collect())
}

#[derive(Debug,PartialEq)]
struct FieldLine {
    field_type: String,
    field_name: String,
}

#[derive(Debug,PartialEq)]
enum FieldCase {
    Unit,
    Vector,
    Array(usize),
    Const(String),
}

#[derive(Clone,Debug,PartialEq)]
enum MemberCase {
    Unit,
    Vector,
    Array(usize),
}

impl<'a> From<&'a FieldCase> for MemberCase {
    fn from(case: &'a FieldCase) -> MemberCase {
        match *case {
            FieldCase::Unit => MemberCase::Unit,
            FieldCase::Vector => MemberCase::Vector,
            FieldCase::Array(v) => MemberCase::Array(v),
            _ => panic!("Conversion not supported"),
        }
    }
}

#[derive(Debug,PartialEq)]
struct FieldInfo {
    datatype: String,
    name: String,
    case: FieldCase,
}

#[derive(Debug,PartialEq)]
enum DataType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Time,
    Duration,
    LocalStruct(String),
    RemoteStruct(String, String),
}

#[derive(Debug,PartialEq)]
struct MemberInfo {
    datatype: DataType,
    name: String,
    case: MemberCase,
}

fn parse_members(fields: &[FieldInfo]) -> Result<Vec<MemberInfo>, ()> {
    fields.iter()
        .map(|v| {
            Ok(MemberInfo {
                datatype: parse_datatype(&v.datatype).ok_or(())?,
                name: v.name.clone(),
                case: MemberCase::from(&v.case),
            })
        })
        .collect()
}

fn parse_datatype(datatype: &str) -> Option<DataType> {
    match datatype {
        "bool" => Some(DataType::Bool),
        "int8" => Some(DataType::I8),
        "int16" => Some(DataType::I16),
        "int32" => Some(DataType::I32),
        "int64" => Some(DataType::I64),
        "uint8" => Some(DataType::U8),
        "uint16" => Some(DataType::U16),
        "uint32" => Some(DataType::U32),
        "uint64" => Some(DataType::U64),
        "float32" => Some(DataType::F32),
        "float64" => Some(DataType::F64),
        "string" => Some(DataType::String),
        "time" => Some(DataType::Time),
        "duration" => Some(DataType::Duration),
        _ => {
            let parts = datatype.split('/').collect::<Vec<_>>();
            if parts.iter().any(|v| v.len() == 0) {
                return None;
            }
            match parts.len() {
                2 => Some(DataType::RemoteStruct(parts[0].to_owned(), parts[1].to_owned())),
                1 => Some(DataType::LocalStruct(parts[0].to_owned())),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_field_matches_legal_field() {
        assert_eq!(FieldLine {
                       field_type: "geom_msgs/Twist".into(),
                       field_name: "myname".into(),
                   },
                   match_field("geom_msgs/Twist   myname").unwrap());
    }

    #[test]
    fn match_vector_field_matches_legal_field() {
        assert_eq!(FieldLine {
                       field_type: "geom_msgs/Twist".into(),
                       field_name: "myname".into(),
                   },
                   match_vector_field("geom_msgs/Twist [  ]   myname").unwrap());
    }

    #[test]
    fn match_array_field_matches_legal_field() {
        assert_eq!((FieldLine {
                        field_type: "geom_msgs/Twist".into(),
                        field_name: "myname".into(),
                    },
                    127),
                   match_array_field("geom_msgs/Twist   [   127 ]   myname").unwrap());
    }

    #[test]
    fn match_const_string_matches_legal_field() {
        assert_eq!((FieldLine {
                        field_type: "string".into(),
                        field_name: "myname".into(),
                    },
                    "this is # data".into()),
                   match_const_string("string   myname  =  this is # data").unwrap());
    }

    #[test]
    fn match_const_numeric_matches_legal_field() {
        assert_eq!((FieldLine {
                        field_type: "mytype".into(),
                        field_name: "myname".into(),
                    },
                    "-444".into()),
                   match_const_numeric("mytype   myname  =  -444").unwrap());
    }

    #[test]
    fn match_line_works_on_legal_data() {
        assert!(match_line("#just a comment").unwrap().is_none());
        assert!(match_line("#  YOLO !   ").unwrap().is_none());
        assert!(match_line("      ").unwrap().is_none());

        assert_eq!(FieldInfo {
                       datatype: "geom_msgs/Twist".into(),
                       name: "myname".into(),
                       case: FieldCase::Unit,
                   },
                   match_line("  geom_msgs/Twist   myname    # this clearly should succeed")
                       .unwrap()
                       .unwrap());

        assert_eq!(FieldInfo {
                       datatype: "geom_msgs/Twist".into(),
                       name: "myname".into(),
                       case: FieldCase::Vector,
                   },
                   match_line("  geom_msgs/Twist [  ]   myname  # ...").unwrap().unwrap());

        assert_eq!(FieldInfo {
                       datatype: "geom_msgs/Twist".into(),
                       name: "myname".into(),
                       case: FieldCase::Array(127),
                   },
                   match_line("  geom_msgs/Twist   [   127 ]   myname# comment").unwrap().unwrap());
        assert_eq!(FieldInfo {
                       datatype: "string".into(),
                       name: "myname".into(),
                       case: FieldCase::Const("this is # data".into()),
                   },
                   match_line("  string  myname =   this is # data  ").unwrap().unwrap());
        assert_eq!(FieldInfo {
                       datatype: "geom_msgs/Twist".into(),
                       name: "myname".into(),
                       case: FieldCase::Const("-444".into()),
                   },
                   match_line("  geom_msgs/Twist  myname =   -444 # data  ").unwrap().unwrap());
    }

    #[test]
    fn match_lines_parses_real_message() {
        let data = match_lines(include_str!("TwistWithCovariance.msg")).unwrap();
        assert_eq!(vec![FieldInfo {
                            datatype: "Twist".into(),
                            name: "twist".into(),
                            case: FieldCase::Unit,
                        },
                        FieldInfo {
                            datatype: "float64".into(),
                            name: "covariance".into(),
                            case: FieldCase::Array(36),
                        }],
                   data);
    }

    #[test]
    fn parse_members_parses_real_message() {
        let data = parse_members(&match_lines(include_str!("TwistWithCovariance.msg")).unwrap())
            .unwrap();
        assert_eq!(vec![MemberInfo {
                            datatype: DataType::LocalStruct("Twist".into()),
                            name: "twist".into(),
                            case: MemberCase::Unit,
                        },
                        MemberInfo {
                            datatype: DataType::F64,
                            name: "covariance".into(),
                            case: MemberCase::Array(36),
                        }],
                   data);
    }
}

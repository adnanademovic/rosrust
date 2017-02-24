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

#[derive(Debug,PartialEq)]
struct FieldInfo {
    datatype: String,
    name: String,
    case: FieldCase,
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
}

use regex::Regex;

static IGNORE_WHITESPACE: &'static str = r"\s*";
static ANY_WHITESPACE: &'static str = r"\s+";
static COMMENT: &'static str = r"(#.*)?";
static FIELD_TYPE: &'static str = r"([a-zA-Z0-9_/]+)";
static FIELD_NAME: &'static str = r"([a-zA-Z][a-zA-Z0-9_]*)";
static EMPTY_BRACKETS: &'static str = r"\[\s*\]";
static NUMBER_BRACKETS: &'static str = r"\[\s*([0-9]+)\s*\]";

fn match_nothing(data: &str) -> Option<String> {
    lazy_static! {
        static ref MATCHER: String = format!("^{}{}$", IGNORE_WHITESPACE, COMMENT);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    RE.captures(data).map(|captures| captures.get(1).map_or("", |v| v.as_str()).into())
}

fn match_field(data: &str) -> Option<FieldLine> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}$", FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE, COMMENT);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some(FieldLine {
        field_type: captures.get(1).unwrap().as_str().into(),
        field_name: captures.get(2).unwrap().as_str().into(),
        comment: captures.get(3).map_or("", |v| v.as_str()).into(),
    })
}

fn match_vector_field(data: &str) -> Option<FieldLine> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}{}{}$", FIELD_TYPE, IGNORE_WHITESPACE, EMPTY_BRACKETS, ANY_WHITESPACE,
            FIELD_NAME, IGNORE_WHITESPACE, COMMENT);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some(FieldLine {
        field_type: captures.get(1).unwrap().as_str().into(),
        field_name: captures.get(2).unwrap().as_str().into(),
        comment: captures.get(3).map_or("", |v| v.as_str()).into(),
    })
}

fn match_array_field(data: &str) -> Option<(FieldLine, usize)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}{}{}$", FIELD_TYPE, IGNORE_WHITESPACE, NUMBER_BRACKETS, ANY_WHITESPACE,
            FIELD_NAME, IGNORE_WHITESPACE, COMMENT);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((FieldLine {
              field_type: captures.get(1).unwrap().as_str().into(),
              field_name: captures.get(3).unwrap().as_str().into(),
              comment: captures.get(4).map_or("", |v| v.as_str()).into(),
          },
          captures.get(2).unwrap().as_str().parse().unwrap()))
}

fn match_const_string(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^{}{}{}{}={}(.*)$", FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE,
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
              comment: "".into(),
          },
          captures.get(3).unwrap().as_str().into()))
}

fn match_const_numeric(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^{}{}{}{}={}(-?[0-9]+){}{}$", FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME,
            IGNORE_WHITESPACE, IGNORE_WHITESPACE, IGNORE_WHITESPACE, COMMENT);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((FieldLine {
              field_type: captures.get(1).unwrap().as_str().into(),
              field_name: captures.get(2).unwrap().as_str().into(),
              comment: captures.get(4).map_or("", |v| v.as_str()).into(),
          },
          captures.get(3).unwrap().as_str().into()))
}

fn match_line(data: &str) -> Result<Option<FieldInfo>, ()> {
    let data = data.trim();
    if let Some(_) = match_nothing(data) {
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
    if let Some((info, data)) = match_const_string(data) {
        return Ok(Some(FieldInfo {
            datatype: info.field_type,
            name: info.field_name,
            case: FieldCase::Const(data),
        }));
    }
    Err(())
}

#[derive(Debug,PartialEq)]
struct FieldLine {
    field_type: String,
    field_name: String,
    comment: String,
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
    fn match_nothing_matches_comment() {
        assert_eq!(String::from("#just a comment"),
                   match_nothing("#just a comment").unwrap());
        assert_eq!(String::from("#  YOLO !"),
                   match_nothing("#  YOLO !").unwrap());
    }

    #[test]
    fn match_nothing_matches_whitespace() {
        assert_eq!(String::from(""), match_nothing("").unwrap());
    }

    #[test]
    fn match_nothing_fails_to_match_lines_with_logic() {
        assert!(match_nothing("geom_msgs/Twist  # this clearly should fail").is_none());
    }

    #[test]
    fn match_field_matches_legal_field() {
        assert_eq!(FieldLine {
                       field_type: "geom_msgs/Twist".into(),
                       field_name: "myname".into(),
                       comment: "# this clearly should succeed".into(),
                   },
                   match_field("geom_msgs/Twist   myname    # this clearly should succeed")
                       .unwrap());
    }

    #[test]
    fn match_vector_field_matches_legal_field() {
        assert_eq!(FieldLine {
                       field_type: "geom_msgs/Twist".into(),
                       field_name: "myname".into(),
                       comment: "# ...".into(),
                   },
                   match_vector_field("geom_msgs/Twist [  ]   myname  # ...").unwrap());
    }

    #[test]
    fn match_array_field_matches_legal_field() {
        assert_eq!((FieldLine {
                        field_type: "geom_msgs/Twist".into(),
                        field_name: "myname".into(),
                        comment: "# comment".into(),
                    },
                    127),
                   match_array_field("geom_msgs/Twist   [   127 ]   myname# comment").unwrap());
    }

    #[test]
    fn match_const_string_matches_legal_field() {
        assert_eq!((FieldLine {
                        field_type: "mytype".into(),
                        field_name: "myname".into(),
                        comment: "".into(),
                    },
                    "this is # data".into()),
                   match_const_string("mytype   myname  =  this is # data").unwrap());
    }

    #[test]
    fn match_const_numeric_matches_legal_field() {
        assert_eq!((FieldLine {
                        field_type: "mytype".into(),
                        field_name: "myname".into(),
                        comment: "# data".into(),
                    },
                    "-444".into()),
                   match_const_numeric("mytype   myname  =  -444 # data").unwrap());
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
                       datatype: "geom_msgs/Twist".into(),
                       name: "myname".into(),
                       case: FieldCase::Const("this is # data".into()),
                   },
                   match_line("  geom_msgs/Twist  myname =   this is # data  ").unwrap().unwrap());
        assert_eq!(FieldInfo {
                       datatype: "geom_msgs/Twist".into(),
                       name: "myname".into(),
                       case: FieldCase::Const("-444".into()),
                   },
                   match_line("  geom_msgs/Twist  myname =   -444 # data  ").unwrap().unwrap());
    }
}

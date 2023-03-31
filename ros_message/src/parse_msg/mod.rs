use crate::{Error, FieldCase, FieldInfo, Result};
use lazy_static::lazy_static;
use regex::Regex;

static IGNORE_WHITESPACE: &str = r"\s*";
static ANY_WHITESPACE: &str = r"\s+";
static FIELD_TYPE: &str = r"([a-zA-Z0-9_/]+)";
static FIELD_NAME: &str = r"([a-zA-Z][a-zA-Z0-9_]*)";
static EMPTY_BRACKETS: &str = r"\[\s*\]";
static NUMBER_BRACKETS: &str = r"\[\s*([0-9]+)\s*\]";

#[derive(Debug, PartialEq)]
struct FieldLine {
    field_type: String,
    field_name: String,
}

#[inline]
pub fn match_lines(data: &str) -> Result<Vec<FieldInfo>> {
    data.split('\n')
        .filter_map(match_line)
        .collect::<Result<_>>()
}

fn match_line(data: &str) -> Option<Result<FieldInfo>> {
    if let Some((info, data)) = match_const_string(data.trim()) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Const(data),
        ));
    }
    let data = match strip_useless(data) {
        Ok(v) => v,
        Err(v) => return Some(Err(v)),
    };

    if data.is_empty() {
        return None;
    }
    if let Some(info) = match_field(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Unit,
        ));
    }
    if let Some(info) = match_vector_field(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Vector,
        ));
    }
    if let Some((info, count)) = match_array_field(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Array(count),
        ));
    }
    if let Some((info, data)) = match_const_numeric(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Const(data),
        ));
    }
    Some(Err(Error::BadMessageContent(data.into())))
}

fn match_const_string(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^(string){}{}{}={}(.*)$",
            ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE, IGNORE_WHITESPACE
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((
        FieldLine {
            field_type: captures.get(1).unwrap().as_str().into(),
            field_name: captures.get(2).unwrap().as_str().into(),
        },
        captures.get(3).unwrap().as_str().into(),
    ))
}

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
            "^{}{}{}{}{}$",
            FIELD_TYPE, IGNORE_WHITESPACE, EMPTY_BRACKETS, ANY_WHITESPACE, FIELD_NAME
        );
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
            "^{}{}{}{}{}$",
            FIELD_TYPE, IGNORE_WHITESPACE, NUMBER_BRACKETS, ANY_WHITESPACE, FIELD_NAME
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((
        FieldLine {
            field_type: captures.get(1).unwrap().as_str().into(),
            field_name: captures.get(3).unwrap().as_str().into(),
        },
        captures.get(2).unwrap().as_str().parse().unwrap(),
    ))
}

fn match_const_numeric(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^{}{}{}{}={}(-?[0-9\.eE\+\-]+)$",
            FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE, IGNORE_WHITESPACE
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((
        FieldLine {
            field_type: captures.get(1).unwrap().as_str().into(),
            field_name: captures.get(2).unwrap().as_str().into(),
        },
        captures.get(3).unwrap().as_str().into(),
    ))
}

#[inline]
fn strip_useless(data: &str) -> Result<&str> {
    Ok(data
        .split('#')
        .next()
        .ok_or_else(|| Error::BadMessageContent(data.into()))?
        .trim())
}

#[cfg(test)]
mod tests;

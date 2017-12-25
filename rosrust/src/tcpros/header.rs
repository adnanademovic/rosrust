use std::collections::HashMap;
use std;
use serde_rosmsg::{from_reader, to_writer, Error};

pub fn decode<R: std::io::Read>(data: &mut R) -> Result<HashMap<String, String>, Error> {
    from_reader(data)
}

pub fn encode<W: std::io::Write>(
    writer: &mut W,
    data: &HashMap<String, String>,
) -> Result<(), Error> {
    to_writer(writer, data)
}

pub fn match_field(
    fields: &HashMap<String, String>,
    field: &str,
    expected: &str,
) -> Result<(), super::error::Error> {
    use super::error::ErrorKind;
    let actual = match fields.get(field) {
        Some(actual) => actual,
        None => bail!(ErrorKind::HeaderMissingField(field.into())),
    };
    if actual != expected {
        bail!(ErrorKind::HeaderMismatch(
            field.into(),
            expected.into(),
            actual.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;
    use std::collections::HashMap;

    static FAILED_TO_ENCODE: &'static str = "Failed to encode";
    static FAILED_TO_DECODE: &'static str = "Failed to decode";

    #[test]
    fn writes_empty_map() {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let data = HashMap::<String, String>::new();
        encode(&mut cursor, &data).expect(FAILED_TO_ENCODE);

        assert_eq!(vec![0, 0, 0, 0], cursor.into_inner());
    }

    #[test]
    fn writes_single_item() {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let mut data = HashMap::<String, String>::new();
        data.insert(String::from("abc"), String::from("123"));
        encode(&mut cursor, &data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            vec![11, 0, 0, 0, 7, 0, 0, 0, 97, 98, 99, 61, 49, 50, 51],
            cursor.into_inner()
        );
    }

    #[test]
    fn writes_multiple_items() {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let mut data = HashMap::<String, String>::new();
        data.insert(String::from("abc"), String::from("123"));
        data.insert(String::from("AAA"), String::from("B0"));
        encode(&mut cursor, &data).expect(FAILED_TO_ENCODE);
        let data = cursor.into_inner();
        assert!(
            vec![
                21, 0, 0, 0, 7, 0, 0, 0, 97, 98, 99, 61, 49, 50, 51, 6, 0, 0, 0, 65, 65, 65, 61,
                66, 48,
            ] == data
                || vec![
                    21, 0, 0, 0, 6, 0, 0, 0, 65, 65, 65, 61, 66, 48, 7, 0, 0, 0, 97, 98, 99, 61,
                    49, 50, 51,
                ] == data
        );
    }

    #[test]
    fn reads_empty_map() {
        let input = vec![0, 0, 0, 0];
        let data = decode(&mut std::io::Cursor::new(input)).expect(FAILED_TO_DECODE);
        assert_eq!(0, data.len());
    }

    #[test]
    fn reads_single_element() {
        let input = vec![11, 0, 0, 0, 7, 0, 0, 0, 97, 98, 99, 61, 49, 50, 51];
        let data = decode(&mut std::io::Cursor::new(input)).expect(FAILED_TO_DECODE);
        assert_eq!(1, data.len());
        assert_eq!(Some(&String::from("123")), data.get("abc"));
    }

    #[test]
    fn reads_typical_header() {
        let input = vec![
            0xb0, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67,
            0x65, 0x5f, 0x64, 0x65, 0x66, 0x69, 0x6e, 0x69, 0x74, 0x69, 0x6f, 0x6e, 0x3d, 0x73,
            0x74, 0x72, 0x69, 0x6e, 0x67, 0x20, 0x64, 0x61, 0x74, 0x61, 0x0a, 0x0a, 0x25, 0x00,
            0x00, 0x00, 0x63, 0x61, 0x6c, 0x6c, 0x65, 0x72, 0x69, 0x64, 0x3d, 0x2f, 0x72, 0x6f,
            0x73, 0x74, 0x6f, 0x70, 0x69, 0x63, 0x5f, 0x34, 0x37, 0x36, 0x37, 0x5f, 0x31, 0x33,
            0x31, 0x36, 0x39, 0x31, 0x32, 0x37, 0x34, 0x31, 0x35, 0x35, 0x37, 0x0a, 0x00, 0x00,
            0x00, 0x6c, 0x61, 0x74, 0x63, 0x68, 0x69, 0x6e, 0x67, 0x3d, 0x31, 0x27, 0x00, 0x00,
            0x00, 0x6d, 0x64, 0x35, 0x73, 0x75, 0x6d, 0x3d, 0x39, 0x39, 0x32, 0x63, 0x65, 0x38,
            0x61, 0x31, 0x36, 0x38, 0x37, 0x63, 0x65, 0x63, 0x38, 0x63, 0x38, 0x62, 0x64, 0x38,
            0x38, 0x33, 0x65, 0x63, 0x37, 0x33, 0x63, 0x61, 0x34, 0x31, 0x64, 0x31, 0x0e, 0x00,
            0x00, 0x00, 0x74, 0x6f, 0x70, 0x69, 0x63, 0x3d, 0x2f, 0x63, 0x68, 0x61, 0x74, 0x74,
            0x65, 0x72, 0x14, 0x00, 0x00, 0x00, 0x74, 0x79, 0x70, 0x65, 0x3d, 0x73, 0x74, 0x64,
            0x5f, 0x6d, 0x73, 0x67, 0x73, 0x2f, 0x53, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        let data = decode(&mut std::io::Cursor::new(input)).expect(FAILED_TO_DECODE);
        assert_eq!(6, data.len());
        assert_eq!(
            Some(&String::from("string data\n\n")),
            data.get("message_definition")
        );
        assert_eq!(
            Some(&String::from("/rostopic_4767_1316912741557")),
            data.get("callerid")
        );
        assert_eq!(Some(&String::from("1")), data.get("latching"));
        assert_eq!(
            Some(&String::from("992ce8a1687cec8c8bd883ec73ca41d1")),
            data.get("md5sum")
        );
        assert_eq!(Some(&String::from("/chatter")), data.get("topic"));
        assert_eq!(Some(&String::from("std_msgs/String")), data.get("type"));
    }
}

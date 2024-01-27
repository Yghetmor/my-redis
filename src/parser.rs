use crate::frame::Frame;
use bytes::Bytes;

pub fn parse(input: &'static str) -> Result<Frame, String> {
    let input: Vec<&str> = input.clone().split(' ').collect();
    let mut output = Frame::array();
    let cmd = input[0].to_uppercase();
    match cmd.as_str() {
        "PING" => {
            output.push_simple("PING".to_string());
            Ok(output)
        },
        "GET" => {
            if input.len() != 2 {
                Err("Incorrect number of arguments for GET command".to_string())
            } else {
                output.push_bulk(Bytes::from(cmd.clone()));
                output.push_bulk(Bytes::from(input[1].clone()));
                Ok(output)
            }
        },
        "SET" => {
            if input.len() != 3 {
                Err("Incorrect number of arguments for SET command".to_string())
            } else {
                output.push_bulk(Bytes::from(cmd.clone()));
                output.push_bulk(Bytes::from(input[1].clone()));
                output.push_bulk(Bytes::from(input[2].clone()));
                Ok(output)
            }
        },
        cmd => Err(format!("Unknown command: {}", cmd)),
    }
}

//
//TESTS

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use crate::frame::Frame;
    use crate::parser::*;

    #[test]
    fn parse_ping_test() {
        let input = "PING";
        let output = parse(input).unwrap();

        let mut expected = Frame::array();
        expected.push_simple("PING".to_string());

        assert_eq!(expected, output);
    }
    
    #[test]
    fn parse_get_test() {
        let input = "get key";
        let output = parse(input).unwrap();

        let mut expected = Frame::array();
        expected.push_bulk(Bytes::from("GET"));
        expected.push_bulk(Bytes::from("key"));

        assert_eq!(expected, output);
    }

    #[test]
    fn parse_set_test() {
        let input = "set key val";
        let output = parse(input).unwrap();

        let mut expected = Frame::array();
        expected.push_bulk(Bytes::from("SET"));
        expected.push_bulk(Bytes::from("key"));
        expected.push_bulk(Bytes::from("val"));

        assert_eq!(expected, output);
    }
}

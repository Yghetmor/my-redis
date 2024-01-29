use crate::frame::Frame;
use bytes::Bytes;

pub fn parse(input: String) -> Result<Frame, String> {
    let mut input: Vec<String> = input.clone().split(' ').map(String::from).collect();
    let last_elem = input.len() - 1;
    input[last_elem].pop();
    let mut output = Frame::array();
    let binding = input[0].to_uppercase().clone();
    let cmd = binding.as_str();
    match cmd {
        "PING" => {
            if input.len() != 1 {
                Err("Incorrect number of arguments for PING command".to_string())
            } else {
                output.push_simple("PING".to_string());
                Ok(output)
            }
        },
        "GET" => {
            if input.len() != 2 {
                Err("Incorrect number of arguments for GET command".to_string())
            } else {
                output.push_bulk(Bytes::from(input[0].to_uppercase().clone()));
                output.push_bulk(Bytes::from(input[1].clone()));
                Ok(output)
            }
        },
        "SET" => {
            if input.len() != 3 {
                Err("Incorrect number of arguments for SET command".to_string())
            } else {
                output.push_bulk(Bytes::from(input[0].to_uppercase().clone()));
                output.push_bulk(Bytes::from(input[1].clone()));
                output.push_bulk(Bytes::from(input[2].clone()));
                Ok(output)
            }
        },
        _ => Err(format!("Unknown command: {}", cmd)),
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
        let input = "PING".to_string();
        let output = parse(input).unwrap();

        let mut expected = Frame::array();
        expected.push_simple("PING".to_string());

        assert_eq!(expected, output);
    }
    
    #[test]
    fn parse_get_test() {
        let input = "get key".to_string();
        let output = parse(input).unwrap();

        let mut expected = Frame::array();
        expected.push_bulk(Bytes::from("GET"));
        expected.push_bulk(Bytes::from("key"));

        assert_eq!(expected, output);
    }

    #[test]
    fn parse_set_test() {
        let input = "set key val".to_string();
        let output = parse(input).unwrap();

        let mut expected = Frame::array();
        expected.push_bulk(Bytes::from("SET"));
        expected.push_bulk(Bytes::from("key"));
        expected.push_bulk(Bytes::from("val"));

        assert_eq!(expected, output);
    }
}

use bytes::{Buf, Bytes};

#[derive(Debug, PartialEq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub fn serialize(input: Vec<u8>) -> Result<Frame, String> {
        match input[0] as char {
            '+' => {
                Ok(ser_simple_string(input))
            },
            '-' => {
                Ok(ser_simple_error(input))
            },
            ':' => {
                Ok(ser_int(input))
            },
            '$' => {
                Ok(ser_bulk(input))
            },
            '_' => Ok(Frame::Null),
            '*' => {
                let input = input.splitn(2, |c| *c == b'\n').nth(1).unwrap().to_vec();
                if let Ok(res) = ser_array(input) {
                    Ok(res)
                } else {
                    return Err("Unimplemented data type".to_string());
                }
            },
            _ => Err("Unimplemented data type".to_string()),
        }
    }

    pub fn deserialize(&mut self) -> Vec<u8> {
        match self {
            Frame::Simple(s) => {
                deser_simple_string(s.to_string())
            },
            Frame::Error(s) => {
                deser_error(s.to_string())
            },
            Frame::Integer(val) => {
                deser_int(*val)
            },
            Frame::Bulk(ref mut vec) => {
                deser_string(vec)
            },
            Frame::Null => {
                let output: Vec<u8> = vec!('_' as u8, '\r' as u8, '\n' as u8);
                output
            },
            Frame::Array(ref mut vec) => {
                deser_array(vec)
            },
        }
    }
}
//SERIALIZATION

fn ser_simple_string(input: Vec<u8>) -> Frame {
    let vec = input.into_iter().skip(1).take_while(|x| *x != b'\r').collect();
    let strin = String::from_utf8(vec).unwrap();
    Frame::Simple(strin)
}

fn ser_simple_error(input: Vec<u8>) -> Frame {
    let vec = input.into_iter().skip(1).take_while(|x| *x != b'\r').collect();
    let err = String::from_utf8(vec).unwrap();
    Frame::Error(err)
}

fn ser_int(input: Vec<u8>) -> Frame {
    let mut vec: Vec<u8> = input.into_iter().skip(1).take_while(|x| *x != b'\r').collect();
    let sign = vec.remove(0);
    let num: i64 = std::str::from_utf8(&vec).unwrap().parse().unwrap();
    if sign as char == '+' {
        Frame::Integer(num)
    } else {
        Frame::Integer((-1) * num)
    }
}

fn ser_bulk(input: Vec<u8>) -> Frame {
    let length: Vec<u8> = input.iter().skip(1).take_while(|x| **x != b'\r').map(|x| *x).collect();
    let length: u32 = std::str::from_utf8(&length).unwrap().parse().unwrap();
    let strin = input.into_iter().skip_while(|x| *x != b'\n').skip(1).take(length as usize).collect();
    Frame::Bulk(strin)
}

fn ser_array(input: Vec<u8>) -> Result<Frame, String> {     //DOESNT WORK BC LINE PER LINE NOT GOOD
    let mut out: Vec<Frame> = Vec::new();
    let input: Vec<Vec<u8>> = input.split(|x| *x == '\n' as u8).map(Vec::from).collect(); 
    for (i, line) in input.iter().enumerate() {
        let mut temp: Frame = Frame::Null;
        match line[0] as char {
            '+' => {
                temp = ser_simple_string((*line).clone());
            },
            '-' => {
                temp = ser_simple_error((*line).clone());
            },
            ':' => {
                temp = ser_int((*line).clone());
            },
            '$' => {
                let mut input2 = input[i + 1].clone();
                let mut line = line.clone();
                line.append(&mut input2);
                temp = ser_bulk(line);
            },
            '_' => temp = Frame::Null,
            '*' => {
                if let Ok(res) = ser_array((*line).clone()) {
                    temp = res;
                } else {
                    return Err("Unimplemented data type".to_string());
                }
            },
            _ => {},
        }
        out.push(temp);
    }
    Ok(Frame::Array(out))
}

//DESERIALIZATION

fn deser_simple_string(s: String) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    output.push('+' as u8);
    let mut strin = s.into_bytes();
    output.append(&mut strin);
    output.push('\r' as u8);
    output.push('\n' as u8);
    output
}

fn deser_error(s: String) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    output.push('-' as u8);
    let mut strin = s.into_bytes();
    output.append(&mut strin);
    output.push('\r' as u8);
    output.push('\n' as u8);
    output
}

fn deser_int(val: i64) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    output.push(':' as u8);
    
    if val >= 0 {
        output.push('+' as u8);
    } else {
        output.push('-' as u8);
    }
    
    let mut strin = val.to_string().into_bytes();
    output.append(&mut strin);
    output.push('\r' as u8);
    output.push('\n' as u8);
    output
}

fn deser_string(mut vec: &mut Vec<u8>) -> Vec<u8> {
    let mut length = vec.len().to_string().as_bytes().to_vec();
    let mut output: Vec<u8> = Vec::new();
    output.push('$' as u8);
    output.append(&mut length);
    output.push('\r' as u8);
    output.push('\n' as u8);
    output.append(&mut vec);
    output.push('\r' as u8);
    output.push('\n' as u8);
    output
}

fn deser_array(vec: &mut Vec<Frame>) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    output.push('*' as u8);
    let mut length = vec.len().to_string().as_bytes().to_vec();
    output.append(&mut length);
    for frame in vec {
        let mut temp: Vec<u8> = Vec::new();
        match frame {
            Frame::Simple(s) => {
                temp = deser_simple_string(s.to_string());
            },
            Frame::Error(s) => {
                temp = deser_error(s.to_string());
            },
            Frame::Integer(val) => {
                temp = deser_int(*val);
            },
            Frame::Bulk(ref mut vec) => {
                temp = deser_string(vec);
            },
            Frame::Null => {
                let output: Vec<u8> = vec!('_' as u8, '\r' as u8, '\n' as u8);
                temp = output;
            },
            Frame::Array(ref mut this_vec) => {
                temp = deser_array(this_vec);
            },
        }
        output.append(&mut temp);
    }
     output
}


//TESTS

#[cfg(test)]
mod tests {
    use crate::Frame;

    use super::{deser_error, ser_array};

    #[test]
    fn simple_serialization() {
        let input = "+OK\r\n".as_bytes().to_vec();
        let output = Frame::serialize(input).unwrap();
        let expected = Frame::Simple("OK".to_string());
        assert_eq!(output, expected);
    }

    #[test]
    fn simple_deser() {
        let mut input = Frame::Simple("OK".to_string());
        let output = input.deserialize();
        let expected = "+OK\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn error_serialization() {
        let input = "-Error message\r\n".as_bytes().to_vec();
        let output = Frame::serialize(input).unwrap();
        let expected = Frame::Error("Error message".to_string());
        assert_eq!(output, expected);
    }

    #[test]
    fn error_deser() {
        let mut input = Frame::Error("Error message".to_string());
        let output = input.deserialize();
        let expected = "-Error message\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn error_deser_fn() {
        let input = "Error message".to_string();
        let output = deser_error(input);
        let expected = "-Error message\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn int_serialization() {
        let input = ":+231\r\n".as_bytes().to_vec();
        let output = Frame::serialize(input).unwrap();
        let expected = Frame::Integer(231);
        assert_eq!(output, expected);
    }

    #[test]
    fn int_deser() {
        let mut input = Frame::Integer(231);
        let output = input.deserialize();
        let expected = ":+231\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn bulk_serialization() {
        let input = "$5\r\nhello\r\n".as_bytes().to_vec();
        let output = Frame::serialize(input).unwrap();
        let expected = Frame::Bulk("hello".as_bytes().to_vec());
        assert_eq!(output, expected);
    }

    #[test]
    fn bulk_deser() {
        let mut input = Frame::Bulk("hello".as_bytes().to_vec());
        let output = input.deserialize();
        let expected = "$5\r\nhello\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn null_serialization() {
        let input = "_\r\n".as_bytes().to_vec();
        let output = Frame::serialize(input).unwrap();
        let expected = Frame::Null;
        assert_eq!(output, expected);
    }

    #[test]
    fn null_deser() {
        let mut input = Frame::Null;
        let output = input.deserialize();
        let expected = "_\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn array_serialization() {
        let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes().to_vec();
        let output = Frame::serialize(input).unwrap();
        let v: Vec<Frame> = vec![Frame::Bulk("hello".as_bytes().to_vec()), Frame::Bulk("world".as_bytes().to_vec())];
        let expected = Frame::Array(v);
        assert_eq!(output, expected);
    }

    #[test]
    fn array_ser_fn() {
        let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes().to_vec();
        let output = ser_array(input).unwrap();
        let v: Vec<Frame> = vec![Frame::Bulk("hello".as_bytes().to_vec()), Frame::Bulk("world".as_bytes().to_vec())];
        let expected = Frame::Array(v);
        assert_eq!(output, expected);
    }

    #[test]
    fn array_deser() {
        let v: Vec<Frame> = vec![Frame::Bulk("hello".as_bytes().to_vec()), Frame::Bulk("world".as_bytes().to_vec())];
        let mut input = Frame::Array(v);
        let output = input.deserialize();
        let expected = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
}

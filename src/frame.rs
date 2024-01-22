use bytes::{Buf, Bytes};
use std::io::Cursor;

#[derive(Debug, PartialEq, Clone)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub fn array() -> Frame {
        Frame::Array(vec![])
    }

    pub fn push_simple(&mut self, string: String) {
        match self {
            Frame::Array(vec) => vec.push(Frame::Simple(string)),
            _ => panic!("not an array frame"),
        }
    }

    pub fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Frame::Array(vec) => vec.push(Frame::Bulk(bytes)),
            _ => panic!("not an array frame"),
        }
    }

    pub fn push_int(&mut self, value: i64) {
        match self {
            Frame::Array(vec) => vec.push(Frame::Integer(value)),
            _ => panic!("not an array frame"),
        }
    }

    pub fn to_string(&mut self) -> Result<String, String> {
        match self {
            Frame::Simple(s) => Ok(s.to_string()),
            Frame::Error(s) => Ok(s.to_string()),
            Frame::Integer(i) => Ok(i.to_string()),
            Frame::Bulk(s) => {
                let s = String::from_utf8(s.to_vec()).unwrap();
                Ok(s)
            },
            _ => Err("Could not convert to string".to_string()),
        }
    }

    pub fn serialize(input: &mut Cursor<&[u8]>) -> Result<Frame, String> {
        match input.get_u8() {
            b'+' => {
                let mut line = Vec::new();
                while input.remaining() != 0 {
                    let i = input.get_u8();
                    if i != b'\r' && i != b'\n' {
                        line.push(i)
                    }
                }
                if let Ok(num) = String::from_utf8(line) {
                    Ok(Frame::Simple(num))
                } else {
                    Err("Error parsing simple string".to_string())
                }
            },
            b'-' => {
                let mut line = Vec::new();
                while input.remaining() != 0 {
                    let i = input.get_u8();
                    if i != b'\r' && i != b'\n' {
                        line.push(i)
                    }
                }
                if let Ok(num) = String::from_utf8(line) {
                    Ok(Frame::Error(num))
                } else {
                    Err("Error parsing simple error".to_string())
                }
            },
            b':' => {
                let mut line = Vec::new();
                while input.remaining() != 0 {
                    let i = input.get_u8();
                    if i != b'\r' && i != b'\n' {
                        line.push(i)
                    }
                }

                let num = String::from_utf8(line).unwrap();
                let num = num.parse::<i64>().unwrap();

                Ok(Frame::Integer(num))
            },
            b'$' => {
                let mut line = Vec::new();
                let mut i: u8 = 0;
                while i != b'\n' {
                    i = input.get_u8();
                    if i != b'\r' && i != b'\n' {
                        line.push(i)
                    }
                }

                let num = String::from_utf8(line).unwrap();
                println!("{:?}", &num);
                let num = num.parse::<usize>().unwrap();
                let n = num + 2;

                let data = Bytes::copy_from_slice(&input.chunk()[..num]);
                input.advance(n);

                Ok(Frame::Bulk(data))
            },
            b'_' => Ok(Frame::Null),
            b'*' => {
                let mut line = Vec::new();
                let mut i: u8 = 0;
                while i != b'\n' {
                    i = input.get_u8();
                    if i != b'\r' && i != b'\n' {
                        line.push(i);
                    }
                }

                let len = String::from_utf8(line).unwrap();
                let len = len.parse::<usize>().unwrap();
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::serialize(input).unwrap());
                }

                Ok(Frame::Array(out))
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
                deser_string(&mut vec.to_vec())
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

//HELPER FN

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
    output.push(b'\r');
    output.push(b'\n');
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
                temp = deser_string(&mut vec.to_vec());
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
    use bytes::Bytes;
    use std::io::Cursor;

    #[test]
    fn simple_serialization() {
        let input = "+OK\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
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
        let input = "-Error message\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
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
    fn int_serialization() {
        let input = ":+231\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
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
        let input = "$5\r\nhello\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let expected = Frame::Bulk(Bytes::from("hello"));
        assert_eq!(output, expected);
    }

    #[test]
    fn bulk_deser() {
        let mut input = Frame::Bulk(Bytes::from("hello"));
        let output = input.deserialize();
        let expected = "$5\r\nhello\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }

    #[test]
    fn null_serialization() {
        let input = "_\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
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
        let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let v: Vec<Frame> = vec![Frame::Bulk(Bytes::from("hello")), Frame::Bulk(Bytes::from("world"))];
        let expected = Frame::Array(v);
        assert_eq!(output, expected);
    }

    #[test]
    fn array_deser() {
        let v: Vec<Frame> = vec![Frame::Bulk(Bytes::from("hello")), Frame::Bulk(Bytes::from("world"))];
        let mut input = Frame::Array(v);
        let output = input.deserialize();
        let expected = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
}

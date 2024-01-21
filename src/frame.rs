use bytes::{Buf, Bytes};
use std::io::Cursor;

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
    pub fn array() -> Frame {
        Frame::Array(vec![])
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

    /*
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
    */
}

//HELPER FN

//SERIALIZATION

/*
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
*/

//DESERIALIZATION

/*
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
*/

//TESTS

#[cfg(test)]
mod tests {
    use crate::Frame;
    use bytes::Bytes;
    use std::io::Cursor;
    //use super::{deser_error, ser_array};

    #[test]
    fn simple_serialization() {
        let input = "+OK\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let expected = Frame::Simple("OK".to_string());
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn simple_deser() {
        let mut input = Frame::Simple("OK".to_string());
        let output = input.deserialize();
        let expected = "+OK\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
    */

    #[test]
    fn error_serialization() {
        let input = "-Error message\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let expected = Frame::Error("Error message".to_string());
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn error_deser() {
        let mut input = Frame::Error("Error message".to_string());
        let output = input.deserialize();
        let expected = "-Error message\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
    */

    #[test]
    fn int_serialization() {
        let input = ":+231\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let expected = Frame::Integer(231);
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn int_deser() {
        let mut input = Frame::Integer(231);
        let output = input.deserialize();
        let expected = ":+231\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
    */

    #[test]
    fn bulk_serialization() {
        let input = "$5\r\nhello\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let expected = Frame::Bulk(Bytes::from("hello"));
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn bulk_deser() {
        let mut input = Frame::Bulk("hello".as_bytes().to_vec());
        let output = input.deserialize();
        let expected = "$5\r\nhello\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
    */

    #[test]
    fn null_serialization() {
        let input = "_\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let expected = Frame::Null;
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn null_deser() {
        let mut input = Frame::Null;
        let output = input.deserialize();
        let expected = "_\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
    */

    #[test]
    fn array_serialization() {
        let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes();
        let mut input_cursor = Cursor::new(input);
        let output = Frame::serialize(&mut input_cursor).unwrap();
        let v: Vec<Frame> = vec![Frame::Bulk(Bytes::from("hello")), Frame::Bulk(Bytes::from("world"))];
        let expected = Frame::Array(v);
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn array_deser() {
        let v: Vec<Frame> = vec![Frame::Bulk("hello".as_bytes().to_vec()), Frame::Bulk("world".as_bytes().to_vec())];
        let mut input = Frame::Array(v);
        let output = input.deserialize();
        let expected = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes().to_vec();
        assert_eq!(output, expected);
    }
    */
}

use crate::frame::Frame;
use std::str;
use bytes::Bytes;

pub enum Command {
    PING,
    GET {
        name: String,
    },
    SET {
        name: String,
        val: Frame,
    },
    NULL,
}

pub struct Handler {
    command: Command,
}

impl Handler {
    fn new() -> Handler {
        Handler {
            command: Command::NULL,
        }
    }
    
    fn get_command(&mut self, frame: Frame) -> Result<(), String> {
        match frame {
            Frame::Array(mut vec) => {
                if vec.len() > 3 {
                    return Err("Too many arguments in command".to_string());
                } else {
                    match &vec[0] {
                        Frame::Simple(cmd) => {
                            if cmd.to_uppercase() == "PING".to_string() {
                                self.command = Command::PING;
                                return Ok(());
                            } else {
                                return Err("Unknown simple command".to_string());
                            }
                        },
                        Frame::Bulk(cmd) => {
                            match str::from_utf8(&cmd.to_ascii_uppercase()).unwrap() {
                                "GET" => {
                                    if vec.len() != 2 {
                                        return Err("incorrect number of arguments for GET command".to_string());
                                    } else {
                                        self.command = Command::GET{ name: vec[1].to_string().unwrap() };
                                        Ok(())
                                    }
                                },
                                "SET" => {
                                    if vec.len() != 3 {
                                        return Err("incorrect number of arguments for SET command".to_string());
                                    } else {
                                        self.command = Command::SET { 
                                            name: vec[1].to_string().unwrap(),
                                            val: vec[2].clone(),
                                        };
                                        Ok(())
                                    }
                                },
                                _ => return Err("Unknown command".to_string()),
                            }
                        }
                        _ => return Err("Unexpected frame".to_string()),
                    }
                }
            }
            _ => Err("Unexpected frame".to_string()),
        }
    }
    
    fn make_command(&mut self, input: &str) -> Result<(), String> {
        let vec: Vec<&str> = input.split(' ').collect();
        let cmd = vec[0].to_uppercase();
        match cmd.as_str() {
            "PING" => {
                self.command = Command::PING;
                Ok(())
            },
            "GET" => {
                if vec.len() != 2 {
                    Err("Incorrect number of arguments for GET command".to_string())
                } else {
                    self.command = Command::GET { name: vec[1].to_string() };
                    Ok(())
                }
            }
            "SET" => {
                if vec.len() != 3 {
                    Err("Incorrect number of arguments for SET command".to_string())
                } else {
                    self.command = Command::SET { 
                        name: vec[1].to_string(),
                        val: Frame::Bulk(Bytes::from(vec[2].to_string())),
                    };
                    Ok(())
                }
            }
            _ => Err("Unknown command".to_string()),
        }
    }
}

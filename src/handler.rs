use std::collections::HashMap;
use crate::frame::Frame;
use std::str;
use bytes::Bytes;

#[derive(PartialEq, Debug)]
pub enum Command {
    PING,
    GET ( String ),
    SET (String, Frame),
    NULL,
}

#[derive(PartialEq, Debug)]
pub struct Handler<'a> {
    command: Command,
    db: &'a mut HashMap<String, String>,
}

impl<'a> Handler<'a> {
    pub fn new(database: &mut HashMap<String, String>) -> Handler {
        Handler {
            command: Command::NULL,
            db: database,
        }
    }
    
    pub fn get_command(&mut self, frame: Frame) -> Result<(), String> {
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
                                        self.command = Command::GET( vec[1].to_string().unwrap() );
                                        Ok(())
                                    }
                                },
                                "SET" => {
                                    if vec.len() != 3 {
                                        return Err("incorrect number of arguments for SET command".to_string());
                                    } else {
                                        self.command = Command::SET ( 
                                            vec[1].to_string().unwrap(),
                                            vec[2].clone(),
                                        );
                                        Ok(())
                                    }
                                },
                                cmd=> return Err(format!("Unknown command: {}", cmd)),
                            }
                        }
                        _ => return Err("Unexpected frame".to_string()),
                    }
                }
            }
            _ => Err("Unexpected frame".to_string()),
        }
    }
    
    pub fn execute_cmd(self) -> Result<Frame, String> {
        match self.command {
            Command::PING => {
                Ok(Frame::Simple("PONG".to_string()))
            },
            Command::GET(key) => {
                if let Some(val) = self.db.get(&key) {
                    Ok(Frame::Bulk(Bytes::from((*val).clone())))
                } else {
                    Ok(Frame::Simple("Nil".to_string()))
                }
            },
            Command::SET(key, val) => {
                self.db.insert(key, val.clone().to_string().unwrap());
                Ok(Frame::Simple("Ok".to_string()))
            },
            Command::NULL => Err("Tried to execute null command".to_string()),
        }
    }
}

//TESTS

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use bytes::Bytes;
    use crate::Handler;
    use crate::handler::Command;
    use crate::frame::Frame;
    
    #[test]
    fn handler_ping_command_frame() {
        let mut db1 = HashMap::new();
        let mut db2 = HashMap::new();
        let ping = "PING".to_string();
        let mut input = Frame::array();
        input.push_simple(ping);

        let mut handler = Handler::new(&mut db1);
        handler.get_command(input).unwrap();

        let expected = Handler{ 
            command: Command::PING, 
            db: &mut db2,
        };

        assert_eq!(handler.command, expected.command);
    }

    #[test]
    fn handler_get_command_frame() {
        let mut db1 = HashMap::new();
        let mut db2 = HashMap::new();
        let cmd = Bytes::from("GET");
        let name = Bytes::from("test");
        let mut input = Frame::array();
        input.push_bulk(cmd);
        input.push_bulk(name);

        let mut handler = Handler::new(&mut db1);
        handler.get_command(input).unwrap();

        let expected = Handler {
            command: Command::GET (
                "test".to_string(),
            ),
            db: &mut db2,
        };

        assert_eq!(handler.command, expected.command);
    }

    #[test]
    fn handler_set_command_frame() {
        let mut db1 = HashMap::new();
        let mut db2 = HashMap::new();
        let cmd = Bytes::from("SET");
        let name = Bytes::from("test");
        let val = Bytes::from("testval");
        let mut input = Frame::array();
        input.push_bulk(cmd);
        input.push_bulk(name);
        input.push_bulk(val);

        let mut handler = Handler::new(&mut db1);
        handler.get_command(input).unwrap();

        let expected = Handler {
            command: Command::SET (
                "test".to_string(),
                Frame::Bulk(Bytes::from("testval")),
            ),
            db: &mut db2,
        };

        assert_eq!(handler.command, expected.command);
    }
    
    #[test]
    fn handler_execute_command_test() {
        let mut db1 = HashMap::new();
        let mut db2 = HashMap::new();
        let mut db3 = HashMap::new();

        let ping_handler = Handler{ 
            command: Command::PING, 
            db: &mut db1,
        };

        let get_handler = Handler {
            command: Command::GET (
                "test".to_string(),
            ),
            db: &mut db2,
        };

        let set_handler = Handler {
            command: Command::SET (
                "test".to_string(),
                Frame::Bulk(Bytes::from("testval")),
            ),
            db: &mut db3,
        };

        let ping_output = ping_handler.execute_cmd().unwrap();
        let get_output = get_handler.execute_cmd().unwrap();
        let set_output = set_handler.execute_cmd().unwrap();

        let ping_expected = Frame::Simple("PONG".to_string());
        let get_expected = Frame::Simple("Nil".to_string());
        let set_expected = Frame::Simple("Ok".to_string());

        assert_eq!(ping_output, ping_expected);
        assert_eq!(get_output, get_expected);
        assert_eq!(set_output, set_expected);
    }
}

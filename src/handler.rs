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
    
}

//TESTS

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use crate::Handler;
    use crate::handler::Command;
    use crate::frame::Frame;
    
    #[test]
    fn handler_ping_command_frame() {
        let ping = "PING".to_string();
        let mut input = Frame::array();
        input.push_simple(ping);

        let mut handler = Handler::new();
        handler.get_command(input).unwrap();

        let expected = Handler{ command: Command::PING };

        assert_eq!(handler, expected);
    }

    #[test]
    fn handler_get_command_frame() {
        let cmd = Bytes::from("GET");
        let name = Bytes::from("test");
        let mut input = Frame::array();
        input.push_bulk(cmd);
        input.push_bulk(name);

        let mut handler = Handler::new();
        handler.get_command(input).unwrap();

        let expected = Handler {
            command: Command::GET (
                "test".to_string(),
            )
        };

        assert_eq!(handler, expected);
    }

    #[test]
    fn handler_set_command_frame() {
        let cmd = Bytes::from("SET");
        let name = Bytes::from("test");
        let val = Bytes::from("testval");
        let mut input = Frame::array();
        input.push_bulk(cmd);
        input.push_bulk(name);
        input.push_bulk(val);

        let mut handler = Handler::new();
        handler.get_command(input).unwrap();

        let expected = Handler {
            command: Command::SET (
                "test".to_string(),
                Frame::Bulk(Bytes::from("testval")),
            )
        };

        assert_eq!(handler, expected);
    }
}

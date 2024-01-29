use std::io;
use std::io::Write;
use tokio::net::TcpStream;
use std::error::Error;
use my_redis::frame::Frame;
use my_redis::parser::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:6379").await?;
    println!("Succesfully connected to redis server");

    loop {
        print!("my-redis$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input == "END" {
            break;
        } else {
            match parse(input) {
                Ok(ref mut frame) => {
                    stream.writable().await?;
                    match stream.try_write(&mut frame.deserialize()) {
                        Ok(_) => {
                            println!("Succesfully wrote to stream");
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                        Err(e) => return Err(e.into()),
                    }
                },
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            }
        }
        
        stream.readable().await?;
        let mut buf = vec![0; 1024];
        loop {
            match stream.try_read(&mut buf) {
                Ok(n) => {
                    buf.truncate(n);
                    println!("Read {n} chars");
                    break;
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                },
                Err(e) => return Err(e.into()),
            }
        }

        let mut response = Frame::serialize(&mut io::Cursor::new(&buf)).unwrap();
        println!("{}", response.to_string().unwrap());
    }

    println!("/nDisconnected from server");

    Ok(())
}

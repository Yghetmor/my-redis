use std::io;
use tokio::net::TcpStream;
use std::error::Error;
use my_redis::frame::Frame;
use my_redis::parser::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;
    println!("Succesfully connected to redis server");

    loop {
        print!("my-redis$ ");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input == "END" {
            break;
        } else {
            match parse(input) {
                Ok(ref mut frame) => {
                    stream.writable().await?;
                    match stream.try_write(&mut frame.deserialize()) {
                        Ok(_) => {},
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                        Err(e) => return Err(e.into()),
                    }
                },
                Err(e) => println!("{e}"),
            }

            stream.writable().await?;

        }
    }

    Ok(())
}

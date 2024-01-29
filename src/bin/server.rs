use tokio::net::TcpListener;
use std::error::Error;
use std::io;
use std::collections::HashMap;
use my_redis::Frame;
use my_redis::Handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut db: HashMap<String, String> = HashMap::new();
    let mut handler = Handler::new(&mut db);
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let (stream, _) = listener.accept().await?;
    println!("Got a connection");

    loop {
        let mut buf = vec![0; 4096];
        stream.readable().await?;

        match stream.try_read(&mut buf) {
            Ok(n) => {
                buf.truncate(n);
                println!("read some stuff");
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }

        let command = Frame::serialize(&mut io::Cursor::new(&buf)).unwrap();
        handler.get_command(command).unwrap();
        let mut response = handler.execute_cmd().unwrap();
        println!("Result from command execution : {}", response.to_string().unwrap());

        stream.writable().await?;
        match stream.try_write(&mut response.deserialize()) {
            Ok(n) => {
                println!("Wrote {n} chars to stream");
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }

    //Ok(())
}

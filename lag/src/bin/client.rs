use std::{
    io::{BufWriter, Write},
    net::TcpStream,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use lag::Message;

fn send_msg(msg: Message) -> Result<()> {
    let addr = "127.0.0.1:33443";
    let mut stream = TcpStream::connect(addr)?;
    let mut buf_writer = BufWriter::new(&mut stream);
    let s = ron::to_string(&msg)?;
    buf_writer.write(s.as_bytes())?;
    Ok(())
}

fn gen_msg() -> Result<Message> {
    let val = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    Ok(Message::Micro(val))
}

fn main() -> Result<()> {
    for _ in 0..100 {
        let msg = gen_msg()?;
        send_msg(msg)?;
    }
    send_msg(Message::Finish)?;
    Ok(())
}

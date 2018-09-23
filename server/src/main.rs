use std::{str};
use std::env;
use std::fs::File;
use std::io::{Result, Cursor, BufWriter};
use std::io::{Error, ErrorKind};
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};

extern crate rand;
extern crate md5;
extern crate byteorder;
use rand::Rng;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

fn ack_message(socket: &UdpSocket, p_error: f32) -> Result<String>
{
    let mut recv_buf: Vec<u8> = vec![0; 32768];
    let (number_of_bytes, address) = socket.recv_from(&mut recv_buf)?;

    // check md5
    let mut ack_buf = vec![];
    ack_buf.append(&mut recv_buf[0..8].to_vec());

    let mut cursor = Cursor::new(&recv_buf);
    cursor.read_u64::<BigEndian>().unwrap();
    cursor.read_u64::<BigEndian>().unwrap();
    cursor.read_u32::<BigEndian>().unwrap();
    let string_size: usize = cursor.read_u16::<BigEndian>().unwrap() as usize;

    let length = recv_buf.len();
    let data = &recv_buf[0..22+string_size];
    let received_hash = recv_buf[22+string_size..22+string_size+16].to_vec();
    let right_hash = <[u8; 16]>::from(md5::compute(&data)).to_vec();
    if right_hash != received_hash
    {
        return Err(Error::new(ErrorKind::Other, "mensagem com md5 errado"));
    }

    // send ack
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Computador com defeito.");
    let seconds: u64 = since_epoch.as_secs();
    let nanoseconds: u32 = since_epoch.subsec_nanos() as u32;
    ack_buf.write_u64::<BigEndian>(seconds)?;
    ack_buf.write_u32::<BigEndian>(nanoseconds)?;

    // md5
    let hash = md5::compute(&ack_buf);
    ack_buf.append(&mut <[u8; 16]>::from(hash).to_vec());

    // breaks md5 with p_error
    let rng: f32 = rand::thread_rng().gen();
    let end = ack_buf.len() - 1;
    if rng < p_error
    {
        if ack_buf[end] == 255
        {
            ack_buf[end] -= 1;
        }
        else 
        {
            ack_buf[end] += 1;
        }
    }

    // send buffer
    socket.send_to(&ack_buf, address)?;
    
    // return message
    let message = str::from_utf8(&recv_buf[22..string_size+22]).unwrap();
    Ok(message.to_string())
}

fn main() -> Result<()>
{
    let args: Vec<String> = env::args().collect();
    if args.len() < 5
    {
        panic!("Não foram passados argumentos suficientes para o programa.")
    }
    // args parsing
    let file_name = &args[1];
    let port = &args[2];
    let window_size: u16 = args[3].parse().unwrap();
    let p_error: f32 = args[4].parse().unwrap();
    // output
    let mut file = File::create(file_name)?;

    // bind socket port
    let socket = UdpSocket::bind(format!("127.0.0.1:{}", port))?;
    socket.set_read_timeout(None);

    while true
    {
        let result = ack_message(&socket, p_error);
            match result
        {
            Ok(message) => println!("{}", message),
            Err(e) => println!("{}", e)
        };
    }
    
    Ok(())
}

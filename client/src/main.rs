use std::{u64, u32, u16, f32, str};
use std::fs::File;
use std::env;
use std::io::{Result, BufRead, BufReader};
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

extern crate rand;
extern crate md5;
extern crate byteorder;
use rand::Rng;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

fn send_message(socket: &UdpSocket, number: u64, address: &String, message: &String, p_error: f32) -> Result<usize>
{
    // get parameters
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Computador com defeito.");
    let seconds: u64 = since_epoch.as_secs();
    let nanoseconds: u32 = since_epoch.subsec_nanos() as u32;
    let m_size = message.len() as u16;
    let mut send_buf = vec![];

    // make send buffer
    send_buf.write_u64::<BigEndian>(number).unwrap();
    send_buf.write_u64::<BigEndian>(seconds).unwrap();
    send_buf.write_u32::<BigEndian>(nanoseconds).unwrap();
    send_buf.write_u16::<BigEndian>(m_size).unwrap();
    send_buf.append(&mut message.as_bytes().to_vec());

    // md5
    let hash = md5::compute(&send_buf);
    send_buf.append(&mut <[u8; 16]>::from(hash).to_vec());

    // breaks md5 with p_error
    let rng: f32 = rand::thread_rng().gen();
    let end = send_buf.len() - 1;
    if rng < p_error
    {
        if send_buf[end] == 255
        {
            send_buf[end] -= 1;
        }
        else 
        {
            send_buf[end] += 1;
        }
    }

    // send buffer
    let send_result = socket.send_to(&send_buf, address);
    send_result
}

fn receive_ack(socket: &UdpSocket) -> bool
{
    let mut recv_buf = vec![0; 32768];
    let result = socket.recv_from(&mut recv_buf);
    match result
    {
        Ok(_) => println!("Ok"),
        Err(e) =>
        {
            println!("Timeout ou erro na ack");
            return false
        }
    };
    
    // confere md5
    let data = &recv_buf[0..20];
    let hash = recv_buf[20..36].to_vec();
    let right_hash = <[u8; 16]>::from(md5::compute(&data)).to_vec();
    if right_hash != hash
    {
        return false
    }
    true
}

fn main() -> Result<()>
{
    let args: Vec<String> = env::args().collect();
    if args.len() < 6
    {
        panic!("Não foram passados argumentos suficientes para o programa.")
    }
    // args parsing
    let file_name = &args[1];
    let ip_port: Vec<&str> = args[2].split(":")
        .take(2)
        .collect();
    let ip = ip_port[0];
    let port: u16 = ip_port[1].parse().unwrap();
    let window_size: u16 = args[3].parse().unwrap();
    let timeout: u64 = args[4].parse().unwrap();
    let p_error: f32 = args[5].parse().unwrap();
    // file reader
    let file = File::open(file_name)?;
    let mut reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    // set up upd socket
    let socket = UdpSocket::bind("127.0.0.1:4444")?;
    socket.set_read_timeout(Some(Duration::new(timeout, 0)))?;
    let mut counter: u64 = 0;
    send_message(&socket, counter, &args[2], &lines[counter as usize], p_error)?;
    let ok = receive_ack(&socket);

    Ok(())
}
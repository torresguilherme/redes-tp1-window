use std::{u64, u32, u16, f32};
use std::fs::File;
use std::env;
use std::io::{Result, BufRead, BufReader};
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};

extern crate rand;
extern crate md5;

struct Message
{
    seqnum: u64,
    timestamp: u64,
    timestamp2: u32,
    m_size: u16,
    text: String,
    hash: String
}

fn send_message(socket: UdpSocket, number: u64, address: &String, m: &String) -> Result<usize>
{
    // get parameters
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Computador com defeito.");
    let seconds: u64 = since_epoch.as_secs();
    let nanoseconds: u32 = since_epoch.subsec_nanos() as u32;
    let m_size = m.len();
    Ok(0)
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
    let timeout: f32 = args[4].parse().unwrap();
    let p_error: f32 = args[5].parse().unwrap();
    // file reader
    let file = File::open(file_name)?;
    let mut reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    // set up upd socket
    let mut socket = UdpSocket::bind("127.0.0.1:4444")?;
    let mut counter: u64 = 0;
    send_message(socket, counter, &args[2], &lines[counter as usize]);

    Ok(())
}
use std::{u64, u32, u16, f32, str};
use std::fs::File;
use std::env;
use std::io::{Result, Error, ErrorKind, Cursor, BufRead, BufReader};
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

fn receive_ack(socket: &UdpSocket) -> Result<u64>
{
    let mut recv_buf = vec![0; 32768];
    let result = socket.recv_from(&mut recv_buf);
    match result
    {
        Ok(_) => println!("Ok"),
        Err(e) =>
        {
            return Err(e);
        }
    };
    
    // confere md5
    let data = &recv_buf[0..20];
    let hash = recv_buf[20..36].to_vec();
    let right_hash = <[u8; 16]>::from(md5::compute(&data)).to_vec();
    if right_hash != hash
    {
        return Err(Error::new(ErrorKind::Other, "ack com md5 errado"))
    }
    
    let mut cursor = Cursor::new(&recv_buf);
    let pack_number = cursor.read_u64::<BigEndian>().unwrap();
    return Ok(pack_number);
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
    let mut optim = false;
    if args.len() == 7
    {
        if args[6] == "optim"
        {
            optim = true;
        }
    }

    // file reader
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    // set up upd socket
    let socket = UdpSocket::bind("127.0.0.1:4444")?;
    socket.set_read_timeout(Some(Duration::new(timeout, 0)))?;

    if optim
    {
        // set up package queue to send
        let mut package_queue: Vec<u64> = vec![];
        for i in 0..window_size
        {
            if (i as usize) < lines.len()
            {
                package_queue.push(i.into());
            }
        }
        let mut next_package: u64 = if (window_size as usize) > lines.len() { lines.len() as u64 } else { window_size as u64 };

        // send packages
        while !package_queue.is_empty()
        {
            for i in 0..package_queue.len()
            {
                send_message(&socket, package_queue[i], &args[2], &lines[package_queue[i] as usize], p_error)?;
            }
            // get acks
            for i in 0..package_queue.len()
            {
                let result = receive_ack(&socket);
                match result
                {
                    Ok(pack_number) =>
                    {
                        let index = package_queue.iter().position(|&n| n == pack_number).unwrap();
                        package_queue.remove(index);
                        if next_package < lines.len() as u64
                        {
                            package_queue.push(next_package);
                            next_package += 1;
                        }
                    },
                    Err(e) => println!("{}", e)
                };
            }
        }
    }
    else
    {
        let mut first_package: u64 = 0;
        let mut last_package: u64 = if (window_size as usize) > lines.len() { lines.len() as u64 } else { window_size as u64 };
        let mut is_package_ok: Vec<bool> = vec![];
        for i in 0..window_size
        {
            is_package_ok.push(false);
        }
        while first_package != last_package
        {
            // envia pacotes
            for i in first_package..last_package
            {
                if !is_package_ok[(i-first_package) as usize]
                {
                    send_message(&socket, i.into(), &args[2], &lines[i as usize], p_error)?;
                }
            }

            // recebe acks
            for i in first_package..last_package
            {
                if !is_package_ok[(i-first_package) as usize]
                {
                    let result = receive_ack(&socket);
                    match result
                    {
                        Ok(pack_number) =>
                        {
                            is_package_ok[(pack_number-first_package) as usize] = true;
                        },
                        Err(e) => println!("{}", e)
                    };
                }
            }

            // desliza janela
            for i in 0..window_size
            {
                if is_package_ok.len() > 0 && is_package_ok[0]
                {
                    if first_package < last_package
                    {
                        first_package += 1;
                        is_package_ok.remove(0);
                    }
                    if last_package < lines.len() as u64
                    {
                        last_package += 1;
                        is_package_ok.push(false);
                    }
                }
                else
                {
                    break;
                }
            }
        }
    }

    Ok(())
}
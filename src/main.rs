use ping_rs::{PingOptions, send_ping_async};
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Simple IP range scanner, as netscan but for CLI. Version 0.1.0");
    println!("Enter start and end IPs");
    println!(
        "CONSTRAINTS. Only one simple subnet at a time (e.g. 192.168.1.1/24). IPv4 only. Start IP must be less than end IP."
    );
    let start_ip: Ipv4Addr;
    let end_ip: Ipv4Addr;
    loop {
        if let Some(ip) = input_value("Enter start IP (ex. 192.168.1.1): ") {
            start_ip = ip;
            break;
        }
    }
    loop {
        if let Some(ip) = input_value("Enter end IP (ex. 192.168.1.255): ") {
            end_ip = ip;
            break;
        }
    }
    verify_input(start_ip, end_ip);
    let total = end_ip.octets()[3] - start_ip.octets()[3] + 1;
    println!(
        "Scanning range: {}-{}, total {}",
        start_ip.to_string(), end_ip.to_string(), total.to_string()
    );
    let active_hosts = ping_range(start_ip, end_ip).await;
    if !active_hosts.is_empty() {
        for host in active_hosts {
            println!("{}", host.to_string());
        }
    }
    else {
        println!("No active hosts found in this range");
    }

}

async fn ping_range (start_ip_r: Ipv4Addr, end_ip_r: Ipv4Addr) -> Vec<IpAddr> {
    let mut ping_handles = Vec::new();
    for target in start_ip_r..=end_ip_r {
        let t = IpAddr::V4(target);
        let handle = tokio::spawn(async move {ping_single(t).await});
        ping_handles.push(handle);
    }
    let mut hosts = Vec::new();
    for handle in ping_handles {
        match handle.await {
            Ok((ip, is_up)) =>if is_up { hosts.push(ip) },
            Err(_e) => {}
        }
    }
    hosts
}

async fn ping_single(target: IpAddr) -> (IpAddr, bool) {
    let timeout = Duration::from_secs(3);
    let data = [0u8; 4];
    let ping_data = Arc::new(&data[..]);
    let options = PingOptions {ttl: 64, dont_fragment: true};
    match send_ping_async(&target, timeout, ping_data.clone(), Some(&options) ).await {
        Ok(_duration) => (target, true),
        Err(_e) => (target, false),
    }
}

fn verify_input(start_ip: Ipv4Addr, end_ip: Ipv4Addr) {
    //validates user input and terminates if something is wrong
    if start_ip > end_ip {
        println!("Start IP must be less than end IP");
        std::process::exit(1);
    }
    for octet in 0..3 {
        if start_ip.octets()[octet] != end_ip.octets()[octet] {
            println!("Only one subnet is allowed in this version.");
            std::process::exit(1);
        }
    }
}
fn input_value<T>(prompt: &str) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    loop {
        print!("{}", prompt);
        io::stdout().flush().expect("Failed to flush stdout");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error reading line");
        input = input.trim().to_string();

        if input.is_empty() {
            return None; // Вернём None, если пользователь ничего не ввёл
        }

        match input.parse::<T>() {
            Ok(num) => return Some(num), // Обязательно обернуть в Some(T)!
            Err(_) => println!("Invalid input, please try again."),
        }
    }
}

mod ip_checker;

use regex::Regex;
use ipnet::{Ipv6Net};
use std::net::Ipv6Addr;

use std::str::FromStr;
use clap::{Arg, ArgAction, command};

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    hosts: Vec<String>,
    prefix_size: u8,
    record_db_path: String,
}

fn get_ipv6_address_from_record_db_line(record_db_line: &str, prefix_size: u8) -> Option<Ipv6Net> {
    let re = Regex::new(r"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))").unwrap();

    if let Some(capture) = re.captures(record_db_line) {
        let ipv6_address = capture.get(0).map_or("", |m| m.as_str());
        return Some(Ipv6Net::new(Ipv6Addr::from_str(ipv6_address).unwrap(), prefix_size).unwrap());
    }
    return None
}

fn change_ipv6_prefix(ipv6_address: Ipv6Net, prefix_new: &Vec<u16>, prefix_size: u8) -> Ipv6Net {
    let host_segments: Vec<u16> = ipv6_address.addr().segments().iter()
        .zip(ipv6_address.network().segments().iter())
        .map(|(x,y)| x - y).collect();

    let segments: Vec<u16> = prefix_new.iter()
        .zip(host_segments.iter())
        .map(|(x, y)| x + y).collect();

    return Ipv6Net::new(Ipv6Addr::from([segments[0], segments[1], segments[2], segments[3],
        segments[4], segments[5], segments[6], segments[7]]), prefix_size).unwrap();
}

fn update_host(hostname: &String, record_db_line: &String, prefix: &Vec<u16>, prefix_size: u8) -> Option<String> {
    if record_db_line.contains(hostname) && record_db_line.contains("AAAA") {
        let ipv6_address = get_ipv6_address_from_record_db_line(record_db_line, prefix_size)
            .expect(format!("Ip-address isn't correct. {}", record_db_line).as_str());

        for i in 0..ipv6_address.network().segments().len() {
            if ipv6_address.network().segments()[i] != prefix[i] {
                let new_ipv6_address = change_ipv6_prefix(ipv6_address, &prefix, prefix_size);
                return Some(record_db_line.replace(ipv6_address.addr().to_string().as_str(), new_ipv6_address.addr().to_string().as_str()));
            }
        }
    }
    return None
}

fn write_record_db(record_db_path: &String, record_db: &Vec<String>) {
    std::fs::write(record_db_path, record_db.iter().map(|str: &String| format!("{}\n", str.as_str())).collect::<String>())
        .expect("Failed to write to file.");
}

fn main() {
    let mut all_prefixes_correct = true;
    let args = command!()
        .about("Define hostnames from your Bind config in a json and update their v6 Prefix.\nCurrently only myip.is web-requests are supported to gather the Prefix.")
        .version("2.0.0")
        .arg(Arg::new("config").long("config").short('f').help("Path to the json config").required(true))
        .arg(Arg::new("dry-run").long("dry-run").help("Output to Console, not to disk").action(ArgAction::SetTrue))
        .get_matches();

    let config_path = args.get_one::<String>("config").unwrap().to_string();
    let config: Config = serde_json::from_str(&*std::fs::read_to_string(&config_path).unwrap()).unwrap();
    let mut record_db: Vec<String> = std::fs::read_to_string(&config.record_db_path)
        .expect("Failed to read Binds Record db")
        .lines().map(|line| line.to_string()).collect();

    let prefix: Vec<u16> = Vec::from(ip_checker::MyIp::get_ipv6_address(config.prefix_size).network().segments());

    for i in 0..record_db.len() {
        for host in &config.hosts {
            let new_line = update_host(host, &mut record_db[i], &prefix, config.prefix_size);
            if new_line.is_some() {
                all_prefixes_correct = false;
                if args.get_flag("dry-run") {
                    println!("old: {}\nnew: {}\n", record_db[i], new_line.as_ref().unwrap());
                }
                record_db[i] = new_line.unwrap().to_string();
            }
        }
    }

    if all_prefixes_correct {
        if args.get_flag("dry-run") {
            println!("No updates needed!");
        }
        std::process::exit(0);
    }

    if args.get_flag("dry-run") {
        println!("All Prefixes where updated.")
    } else {
        write_record_db(&config.record_db_path, &record_db);
    }
}

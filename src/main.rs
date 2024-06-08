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

fn update_ipv6_address_in_record_db_line(record_db_line: &str, old_address: Ipv6Net, new_address: Ipv6Net) -> String {
    return record_db_line.replace(old_address.addr().to_string().as_str(), new_address.addr().to_string().as_str());
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

fn main() {
    //new program
    let prefix_size: u8 = 64;
    let mut ipv6_addresses: Vec<Ipv6Net> = vec![];

    bind::get_ipv6_addresses_from_config("src/db.maximizzar.io", &mut ipv6_addresses, prefix_size);
    for ipv6_address in ipv6_addresses {
        println!("{}", ipv6_address);
    }

    // old program
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        help();
        std::process::exit(0);
    }
    // put cli arguments into variables
    let hostname: String = args.get(1).unwrap().to_string();
    let config_path: &String = args.get(2).unwrap();
    let prefix_size: u8 = args.get(3).unwrap().parse().unwrap();

    //load data into variables
    let config: String = read_config(config_path);
    let address_from_config = get_address_from_config(&config, &hostname, &prefix_size);
    let address_from_web = web_ip_checker::MyIp::get_ipv6_address(&prefix_size);



    compare_prefixes(&address_from_config, &address_from_web);
    println!("prefixes differ from each other. Start to update config now!\n");

    write_config(&config_path, update_config(&config, &address_from_config, &address_from_web));
    println!(r#"Changed prefix from "{}" to "{}""#,
             address_from_config.network().to_string(),
             address_from_web.network().to_string()
    );
}
fn compare_prefixes(address_from_config: &Ipv6Net, address_from_web: &Ipv6Net) {
    if address_from_config.network() == address_from_web.network() {
        std::process::exit(0);
    }
}

fn get_address_from_config(config: &String, hostname: &String, netmask: &u8) -> Ipv6Net {
    let mut host_record = String::new();
    for line in config.lines() {
        if line.contains(hostname) && line.contains("AAAA") {
            host_record = line.to_string();
        }
    }
    let mut prefix = String::new();
    {
        let re: Regex = Regex::new(r#"(\w+\s+)(IN\s+)(AAAA\s+)"#).unwrap();
        let Some(prefix_in_config) = re.captures(host_record.as_ref()) else {
            panic!("AAAA-record for {} wasn't present in config!", hostname.as_str());
        };
        let _ = &prefix_in_config[0].clone_into(&mut prefix);
    }
    host_record = host_record.strip_prefix(&prefix).unwrap().to_string();
    return Ipv6Net::new(Ipv6Addr::from_str(host_record.as_str()).unwrap(), *netmask).unwrap();
}

fn read_config(config_path: &String) -> String {
    let mut file = File::open(config_path).expect("Can't open file!");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Can't read file!");

    return contents.to_string();
}

fn update_config(config: &String, address_from_config: &Ipv6Net, address_from_web: &Ipv6Net) -> String {
    return config.replace(
        &address_from_config.network().to_string().strip_suffix("::").unwrap().to_string(),
        &address_from_web.network().to_string().strip_suffix("::").unwrap(),
    );
}

fn write_config(config_path: &String, config: String) {
    let mut file = File::create(config_path)
        .expect("Wasn't able to create file!");

    let _ = file.write_all(config.as_ref()).expect("config couldn't be written");
}

fn help() {
    println!("To function this tool needs some arguments:\n\
    - record-name = the name of a record that has the desired prefix\
    - config path = path to the file, where bind stores your records\
    - netmask = how big the subnet ist as number (48, 64 ,...")
}

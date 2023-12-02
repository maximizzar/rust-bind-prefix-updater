mod web_ip_checker;

use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Ipv6Addr;
use regex::Regex;
use ipnet::{Ipv4Net, Ipv6Net};

use std::str::FromStr;

enum RecordType {
    A(Ipv4Net),
    AAAA(Ipv6Net),
    CNAME(String),
}
struct BindRecord {
    hostname: String,
    record_type: String,
    record: RecordType,
}
fn main() {
    let filepath = "src/db.maximizzar.io";
    read(filepath);

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
fn read(filepath: &'static str) {
    let bind_db_file = File::open(filepath)
        .expect(&*format!("Bind DB File under {} not found.", filepath));
    let reader = BufReader::new(bind_db_file);

    for line in reader.lines() {
        let mut line_part = line.unwrap();

        //for header in line_part.split_whitespace() {
        //    print!("{}", &header);

        //    if header.contains(")") {
        //        break;
        //    }
        //}
        println!();

        for lp in line_part.split_whitespace() {

            if line_part.contains(" A ")
                || line_part.contains(" AAAA ")
                || line_part.contains(" CNAME ") {
                print!("{}", &lp);
            }

        }
    }
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
    let mut file = std::fs::File::open(config_path).expect("Can't open file!");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Can't read file!");

    return contents.to_string();
}

fn update_config(config: &String, address_from_config: &Ipv6Net, address_from_web: &Ipv6Net) -> String {
    return config.replace(
        &address_from_config.network().to_string().strip_suffix("::").unwrap().to_string(),
        &address_from_web.network().to_string().strip_suffix("::").unwrap()
    );
}
fn write_config(config_path: &String, config: String) {
    let mut file = std::fs::File::create(config_path)
        .expect("Wasn't able to create file!");

    let _ = file.write_all(config.as_ref()).expect("config couldn't be written");
}
fn help() {
    println!("To function this tool needs some arguments:\n\
    - record-name = the name of a record that has the desired prefix\
    - config path = path to the file, where bind stores your records\
    - netmask = how big the subnet ist as number (48, 64 ,...")
}

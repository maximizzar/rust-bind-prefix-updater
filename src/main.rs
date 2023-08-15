use curl::easy::Easy;
use regex::Regex;
use ipnet::*;

use std::env;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::str::FromStr;

fn main() {
    let args: Vec<_> = args().collect();

    if args.len() < 2 {
        help();
        exit(0);
    }
    // put cli arguments into variables
    let hostname: String = env::args().nth(1).unwrap();
    let config: String = read_config(env::args().nth(2).unwrap());
    let _netmask:u8 = env::args().nth(3).unwrap().parse().unwrap();

    let address_from_config = get_address_from_config(&config, &hostname);
    let address_from_web = get_address_from_web();

    println!("{}\n{}", address_from_config.network(), address_from_web.network());

    compare_prefixes(address_from_config, address_from_web);
    println!("prefixes differ from each other. Start to update config now!");
}

fn compare_prefixes(address_from_config: Ipv6Net, address_from_web: Ipv6Net) {
    if address_from_config.network() == address_from_web.network() {
        exit(0);
    }
}

fn get_address_from_config(config: &String, hostname: &String) -> Ipv6Net {
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
    host_record += &*"/64".to_string();

    return Ipv6Net::from_str(host_record.as_str()).unwrap();
}
fn get_address_from_web() -> Ipv6Net {
    // First write everything into a `Vec<u8>`
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url("https://6.myip.is/").unwrap();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    // Convert it to `String`
    let mut body: String = String::from_utf8(data).expect("body is not valid UTF8!");

    let prefix: String = r#"{"ip":""#.to_string();
    let mut suffix: String = String::new();
    {
        let re: Regex = Regex::new(r#"(","host\S+)"#).unwrap();
        let Some(suffix_in_body) = re.captures(body.as_ref()) else {
            panic!("There wasn't a ivp6 address in web-request present!");
        };
        let _ = &suffix_in_body[0].clone_into(&mut suffix);
    }

    //Remove prefix and suffix from returned body
    body = body.strip_prefix(&prefix).unwrap().to_string();
    body = body.strip_suffix(&suffix).unwrap().to_string();
    body += &*"/64".to_string();

    return Ipv6Net::from_str(body.as_str()).unwrap();
}

fn read_config(config_path: String) -> String {
    let mut file = File::open(config_path).expect("Can't open file!");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Can't read file!");

    return contents.to_string();
}

fn help() {
    println!("provide a hostname to check against and the path to your bind zone-file, where your Records are located.
                -> prefix-swapper hostname path/to/config

                It makes sense to add the program into a cronjob =)")
}

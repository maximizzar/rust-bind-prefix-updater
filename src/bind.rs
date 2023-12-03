use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::Ipv6Addr;
use std::str::FromStr;
use ipnet::Ipv6Net;

pub(crate) fn get_ipv6_addresses_from_config(filepath: &'static str, ipv6_addresses: &mut Vec<Ipv6Net>, prefix_size: &u8) {
    let bind_db_file = File::open(filepath)
        .expect(&*format!("Bind DB File under {} not found.", filepath));
    let reader = BufReader::new(bind_db_file);

    for line in reader.lines() {
        if !line.as_ref().unwrap().contains(" AAAA ") || line.as_ref().unwrap().contains(";") {
            continue;
        }
        let ipv6_str = line.as_ref().unwrap().split_whitespace().last().unwrap();
        ipv6_addresses.push(Ipv6Net::new(Ipv6Addr::from_str(ipv6_str).unwrap(), *prefix_size).unwrap());
    }
}

pub(crate) fn get_representative_ip(hostname: &String, config: &String) -> Ipv6Net {
    return Ipv6Net::from_str("").unwrap();
}

pub(crate) fn update_netmask(representative: &Ipv6Net, mut current: Ipv6Net, new: &Ipv6Net) {
    if representative.netmask() != current.netmask() {
        return;
    }
    current = Ipv6Net::with_netmask(current.hostmask(), new.netmask()).unwrap();
}

pub(crate) fn update_config(config: &String) {}

pub(crate) fn write_config(config_path: &String, config: &String) {
    let mut file = File::create(config_path)
        .expect("Wasn't able to create file!");

    let _ = file.write_all(config.as_ref()).expect("config couldn't be written");
}
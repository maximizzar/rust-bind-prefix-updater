use serde_json;
use ipnet::Ipv6Net;
use std::net::Ipv6Addr;
use std::str::FromStr;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyIp {
    ip: Ipv6Addr,
    host: String,
    timestamp: i64,
}

pub struct Interface;

impl MyIp {
    pub fn get_ipv6_address(prefix_length: u8) -> Ipv6Net {
        let web_request = MyIp::web_request().unwrap();
        return Ipv6Net::new(web_request.ip, prefix_length)
            .expect("Failed to obtain an ipv6 address.");
    }

    fn web_request() -> Option<MyIp> {
        let url = "https://6.myip.is/";
        match reqwest::blocking::get(url) {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    return Some(serde_json::from_str(response.text().unwrap().as_str()).unwrap());
                } else {
                    println!("Request failed: {}", response.status())
                }
            }
            Err(_) => eprintln!("Request failed!")
        }
        None
    }
}

impl Interface {
    pub fn get_ipv6_address(iface_name: &str) -> Ipv6Net {
        use pnet::datalink;
        for iface in datalink::interfaces() {
            if iface.name == iface_name {
                for ip in iface.ips {
                    if ip.is_ipv6() {
                        let ip_address = ip[0];
                        return Ipv6Net::from_str(ip_address).addr().to_string().as_str().unwrap();
                    }
                }
            }
        }
    }
}

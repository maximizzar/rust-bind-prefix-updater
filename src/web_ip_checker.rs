use serde_json;
use ipnet::*;
use std::net::Ipv6Addr;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyIp {
    ip: Ipv6Addr,
    host: String,
    timestamp: i64,
}
impl MyIp {
    pub fn get_current_netmask(network_prefix: &u8) -> Option<Ipv6Net> {
        let web_request = MyIp::web_request().unwrap();
        Some(Ipv6Net::new(web_request.ip, *network_prefix).unwrap())
    }
    pub fn web_request() -> Option<MyIp> {
        let url = "https://6.myip.is/";
        match reqwest::blocking::get(url) {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    let myip: MyIp = serde_json::from_str(response.text().unwrap().as_str()).unwrap();
                    return Some(myip);
                } else {
                    println!("Request failed: {}", response.status())
                }
            }
            Err(_) => eprintln!("Request failed!")
        }
        None
    }
}
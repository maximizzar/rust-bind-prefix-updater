use serde_json;
use ipnet::Ipv6Net;
use std::net::Ipv6Addr;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyIp {
    ip: Ipv6Addr,
    host: String,
    timestamp: i64,
}
impl MyIp {
    pub fn get_ipv6_address(prefix_length: &u8) -> Ipv6Net {
        let web_request = MyIp::web_request().unwrap();
        return Ipv6Net::new(web_request.ip, *prefix_length)
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
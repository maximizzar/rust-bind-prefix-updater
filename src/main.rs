
use curl::easy::Easy;

use std::env;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::process::exit;

fn main() {
    let args: Vec<_> = args().collect();

    if args.len() < 2 {
        help();
        exit(0);
    }

    let hostname: String = env::args().nth(1).unwrap();
    let config: String = read_config(env::args().nth(2).unwrap());

    let address_from_config: String = get_address_from_config(&config, &hostname);
    let address_from_web: String = get_address_from_web();

    println!("{}", address_from_config);
    println!("{}", address_from_web);
}

fn get_address_from_config(config: &String, hostname: &String) -> String {
    for line in config.lines() {
        if line.contains(hostname) && line.contains("AAAA") {
            return line.to_string();
        }
    }
    return "".to_string();
}

fn get_address_from_web() -> String {
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
    let body: String = String::from_utf8(data).expect("body is not valid UTF8!");
    return body;
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

                It makes sense to add the programm into a cronjob =)")
}

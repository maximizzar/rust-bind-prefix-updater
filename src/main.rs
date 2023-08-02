
use curl::easy::Easy;
use std::{env, fs};
use std::env::args;
use std::process::exit;


fn main() {
    let args: Vec<_> = args().collect();

    if args.len() < 2 {
        help();
        exit(0);
    }

    let hostname = env::args().nth(1).unwrap();
    let config_path = env::args().nth(2).unwrap();

    let prefix_from_web = get_address_from_web();
    //let prefix_form_config = get_address_from_config(config_path.clone());

    println!("{}", hostname);
    println!("{}", config_path);
    println!("{}", prefix_from_web);
    //println!("{}", prefix_form_config);
}

fn get_address_from_config(config_path: String) -> String {
    let config = fs::read_to_string(config_path)
        .expect("Should have been able to read the file");
    return config;
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
    let mut body = String::from_utf8(data).expect("body is not valid UTF8!");

    for i in 0..body.len() {
        //println!("{}", i);
    }

    return body;
}


fn help() {
    println!("provide a hostname to check against and the path to your bind zone-file, where your Records are located.
                -> prefix-swapper hostname path/to/config

                It makes sense to add the programm into a cronjob =)")
}

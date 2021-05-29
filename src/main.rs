use std::{net::Ipv4Addr, path::Path};

use clap::{App, Arg, SubCommand};

use crate::manager::Manager;

mod manager;
mod wg;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let app = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about("Tool to help manage a WireGuard server.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("FILE")
                .help("Config file path"),
        )
        .subcommand(
            SubCommand::with_name("new")
                .about("Configure a new server")
                .arg(Arg::with_name("ip-range")
                    .help("Client IPv4 range for VPN server, in CIDR notation")
                    .index(1)
                    .required(true))
        );

    let app_m = app.get_matches();

    let config = match app_m.value_of("config") {
        Some(path) => Path::new(path),
        // use default if no config path specified
        None => Path::new("./wgman.conf"),
    };

    match app_m.subcommand() {
        ("new", Some(sub_m)) => {
            println!("new subcommand used");
            let ip_range = sub_m.value_of("ip-range").unwrap();
            let (ip, subnet_bits) = parse_cidr_ipv4(ip_range);

            let manager = Manager::new(ip, subnet_bits);
            manager.save_config(config).expect("Failed to save config");
        }
        _ => {}
    }
}

// TODO: actually parse it, of the form xxx.xxx.xxx.xxx/xx, e.g. 10.33.7.0/24
fn parse_cidr_ipv4(string: &str) -> (Ipv4Addr, u8) {
    (Ipv4Addr::new(10, 33, 7, 0), 24)
}
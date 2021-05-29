use std::{net::{Ipv4Addr, SocketAddrV4}, path::Path};
use clap::{App, AppSettings, Arg, SubCommand};
use crate::manager::Manager;

#[macro_use]
extern crate clap;

mod manager;
mod wg;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let app = clap_app!((NAME) =>
        (version: VERSION)
        (author: AUTHOR)
        (about: "Tool to help manage a WireGuard server")
        (@setting SubcommandRequiredElseHelp)
        (@arg CONFIG: -c --config [FILE] "Path to config file")
        (@subcommand new => 
            (about: "Configure a new server (and create config)")
            (@arg ("IP-RANGE"): * "IPv4 range for the VPN in CIDR notation")
            (@arg ("ADDR"): * "The IPv4 address to bind to")
            (@arg ("PORT"): * "The port to bind to")
        )
        (@subcommand client =>
            (about: "Client-related commands")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand new =>
                (about: "Configure a new client")
                (@arg NAME: * "A unique name for the client")
                (@arg IP: * "The IPv4 address for the client")
            )
            (@subcommand list =>
                (about: "List configured clients")
            )
            (@subcommand delete =>
                (about: "Delete a configured client")
                (@arg NAME: * "The unique name of the client")
            )

        )
    );
    let app_m = app.get_matches();


    let config = match app_m.value_of("CONFIG") {
        Some(path) => Path::new(path),
        // use default if no config path specified
        None => Path::new("./wgman.conf"),
    };

    if let Some(sub_m) = app_m.subcommand_matches("new") {
        println!("new subcommand used");
        let ip_range = sub_m.value_of("IP-RANGE").unwrap();
        let (ip_range, subnet_bits) = parse_cidr_ipv4(ip_range);

        let bind_ip = sub_m.value_of("ADDR").unwrap();
        let bind_port = sub_m.value_of("PORT").unwrap();
        // TODO: input validation
        let endpoint = SocketAddrV4::new(bind_ip.parse().unwrap(), bind_port.parse().unwrap());

        let manager = Manager::new(endpoint, ip_range, subnet_bits);
        manager.save_config(config).expect("Failed to save config");
    } else {
        let mut manager = match Manager::from_config(config) {
            Ok(manager) => manager,
            Err(e) => {
                println!("Failed to load config: {}", e);
                return;
            }
        };

        if let Some(sub_m) = app_m.subcommand_matches("client") {
            match sub_m.subcommand() {
                ("new", Some(sub_m)) => {
                    // TODO: validate these
                    let name = sub_m.value_of("NAME").unwrap();
                    let ip = sub_m.value_of("IP").unwrap();
                    let ip: Ipv4Addr = ip.parse().unwrap();
                    let endpoint = manager.endpoint();

                    let (client, privkey) = manager.new_client(name.to_owned(), ip);
                    let pubkey = client.public_key();

                    let config_string = create_client_config(ip, pubkey, privkey, endpoint);

                    println!("Here is auto-generated config:\n{}", config_string);
                },
                ("list", Some(sub_m)) => todo!(),
                ("delete", Some(sub_m)) => todo!(),
                _ => panic!("Impossible"),
            }
        } else {
        }
    }
}


// TODO: create a wg-quick style config
fn create_client_config(ip: Ipv4Addr, pubkey: String, privkey: String, endpoint: SocketAddrV4) -> String {
    String::from("placeholder")
}

// TODO: actually parse it, of the form xxx.xxx.xxx.xxx/xx, e.g. 10.33.7.0/24
fn parse_cidr_ipv4(string: &str) -> (Ipv4Addr, u8) {
    (Ipv4Addr::new(10, 33, 7, 0), 24)
}

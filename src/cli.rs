use std::{f32::consts::PI, net::{Ipv4Addr, SocketAddrV4}, path::Path};

use clap::{App, ArgMatches};
use ipnet::Ipv4Net;

use crate::{manager::ManagerError, utils::cli_table};
use crate::manager::Manager;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

type CLIResult = std::result::Result<(), CLIError>;

#[derive(Debug)]
enum CLIError {
    FailedToLoadConfig(ManagerError),
    FailedToSaveConfig(ManagerError),
    ClapError(clap::Error),
    ClientNameExistsError(String),
    Other(String),
}

impl From<clap::Error> for CLIError {
    fn from(e: clap::Error) -> Self {
        Self::ClapError(e)
    }
}

impl From<ManagerError> for CLIError {
    fn from(e: ManagerError) -> Self {
        match e {
            ManagerError::ClientNameExistsError(name) => CLIError::ClientNameExistsError(name),
            _ => CLIError::Other(e.to_string())
        }
    }
}

pub fn run() {
    let app = clap_app!((NAME) =>
        (version: VERSION)
        (author: AUTHOR)
        (about: "Tool to help manage a WireGuard server")
        (@setting SubcommandRequiredElseHelp)
        (@arg CONFIG: -c --config [FILE] "Path to config file")
        (@subcommand new =>
            (about: "Configure a new server (and create config)")
            (@arg ("IP-RANGE"): * "IPv4 range for the VPN in CIDR notation")
            (@arg ("BIND-SOCKET-ADDR"): * "The IPv4 address and port to bind to (e.g. 127.0.0.1:51900), default port is 51900")
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

    match process_commands(&app_m, config) {
        Ok(()) => {},
        Err(e) => match e {
            CLIError::ClapError(e) => e.exit(),
            CLIError::FailedToLoadConfig(e) => err(&format!("Failed to load config: {}", e.to_string())),
            CLIError::FailedToSaveConfig(e) => err(&format!("Failed to save config: {}", e.to_string())),
            CLIError::ClientNameExistsError(name) => err(&format!("client with name '{}' already exists", name)),
            CLIError::Other(e) => err(&e),
        },
    };
}

// Slight bodge in order to use clap's nicely formatted errors.
fn err(msg: &str) {
    clap::Error::with_description(msg, clap::ErrorKind::Io).exit()
}

fn process_commands(app_m: &ArgMatches, config: &Path) -> CLIResult {
    match app_m.subcommand() {
        ("new", Some(sub_m)) => sub_new(sub_m, config)?,
        ("client", Some(sub_m)) => match sub_m.subcommand() {
            ("new", Some(sub_m)) => sub_client_new(sub_m, config)?,
            ("list", Some(sub_m)) => sub_client_list(sub_m, config)?,
            ("delete", Some(sub_m)) => sub_client_delete(sub_m, config)?,
            _ => panic!("Impossible"),
        }
        _ => panic!("Impossible"),
    }

    Ok(())
}

fn sub_new(sub_m: &ArgMatches, config: &Path) -> CLIResult {
    let ip_range = value_t!(sub_m, "IP-RANGE", Ipv4Net)?;
    let endpoint = value_t!(sub_m ,"BIND-SOCKET-ADDR", SocketAddrV4)?;

    let manager = Manager::new(endpoint, ip_range);
    save_manager(&manager, config)?;
    Ok(())
}

fn sub_client_new(sub_m: &ArgMatches, config: &Path) -> CLIResult {
    let mut manager = load_manager(config)?;

    let name = value_t!(sub_m, "NAME", String)?;
    let ip = value_t!(sub_m, "IP", Ipv4Addr)?;
    let endpoint = manager.endpoint();

    let (client, privkey) = manager.new_client(name, ip)?;
    let pubkey = client.public_key();

    let config_string = create_client_config(ip, pubkey, &privkey, endpoint);

    println!("Here is auto-generated config:\n{}", config_string);

    save_manager(&manager, config)?;
    Ok(())
}

fn sub_client_list(sub_m: &ArgMatches, config: &Path) -> CLIResult {
    let manager = load_manager(config)?;

    manager.clients();

    let mut table: Vec<Vec<&str>> = Vec::new();

    table.push(vec!["Name", "Pubkey"]);
    for client in manager.clients() {
        table.push(vec![client.name(), client.public_key()]);
    }

    let lines = cli_table(table);

    for line in lines {
        println!("{}", line);
    }

    Ok(())
}

fn sub_client_delete(sub_m: &ArgMatches, config: &Path) -> CLIResult {
    todo!();
}

fn load_manager(config: &Path) -> Result<Manager, CLIError> {
    Manager::from_config(config).map_err(|e| CLIError::FailedToLoadConfig(e))
}

fn save_manager(manager: &Manager, config: &Path) -> CLIResult {
    manager
        .save_config(config)
        .map_err(|e| CLIError::FailedToSaveConfig(e))
}

// TODO: create a wg-quick style config
fn create_client_config(
    ip: Ipv4Addr,
    pubkey: &String,
    privkey: &String,
    endpoint: SocketAddrV4,
) -> String {
    format!("placeholder({}, {}, {}, {})", ip, pubkey, privkey, endpoint)
}

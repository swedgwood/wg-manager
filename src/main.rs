use std::{net::{Ipv4Addr, SocketAddrV4}, path::Path};
use clap::{App, AppSettings, Arg, SubCommand};
use crate::manager::Manager;

#[macro_use]
extern crate clap;

mod manager;
mod wg;
mod cli;

fn main() {
    cli::CLI::run();
}
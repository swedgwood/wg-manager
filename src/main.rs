#[macro_use]
extern crate clap;

mod cli;
mod manager;
mod utils;
mod wg;

fn main() {
    cli::CLI::run();
}

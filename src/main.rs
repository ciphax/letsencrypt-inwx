#[macro_use]
extern crate log;
use env_logger;
use env_logger::Env;
mod cli;
mod config;
mod dns;
mod inwx;
mod rpc;

use std::process::exit;

fn main() {
    let env = Env::new().filter_or("LOG", "letsencrypt_inwx=info");
    env_logger::init_from_env(env);
    openssl_probe::init_ssl_cert_env_vars();

    if cli::run().is_err() {
        exit(1);
    }
}

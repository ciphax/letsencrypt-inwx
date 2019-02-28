extern crate clap;
extern crate openssl_probe;
extern crate serde;
extern crate serde_json;
extern crate cookie;
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate trust_dns;

mod config;
mod rpc;
mod inwx;
mod dns;
mod cli;

use std::process::exit;

fn main() {
	openssl_probe::init_ssl_cert_env_vars();

	if let Err(msg) = cli::run() {
		eprintln!("=> Error: {}", msg);
		exit(1);
	}
}

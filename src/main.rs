extern crate clap;
extern crate letsencrypt_inwx;

use clap::{Arg, App, SubCommand};
use std::fs::File;
use std::process::exit;
use std::io::prelude::*;
use letsencrypt_inwx::inwx::{Inwx, InwxError};

fn read_file(path: &str) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn read_credentials(path: &str) -> Result<(String, String), &'static str> {
    let content = match read_file(path) {
        Ok(content) => content,
        Err(_) => return Err("Could not read the credential file!")
    };

    let content = content.replace("\r\n", "\n").replace("\r", "\n");
    let mut lines = content.split("\n");

    if let Some(user) = lines.next() {
        if let Some(pass) = lines.next() {
            if !user.is_empty () && !pass.is_empty() {
                return Ok((user.to_owned(), pass.to_owned()));
            }
        }
    }

    Err("The credential file is invalid!")
}

fn run() -> Result<(), String> {
    let mut app = App::new("letsencrypt-inwx")
        .version("1.0.0")
        .about("A small cli utility for automating the letsencrypt dns-01 challenge for domains hosted by inwx")
        .subcommand(SubCommand::with_name("create")
            .about("create a TXT record")
            .arg(Arg::with_name("credentialfile")
                .short("c")
                .value_name("CREDENTIAL_FILE")
                .help("specify the path to a file which contains the username and password for the inwx account seperated by a newline")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::with_name("domain")
                .short("d")
                .value_name("DOMAIN")
                .help("the domain of the record (i.e. \"_acme-challenge.example.com\"")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::with_name("value")
                .short("v")
                .value_name("VALUE")
                .help("the value of the record")
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("delete")
            .about("delete a TXT record")
            .arg(Arg::with_name("credentialfile")
                .short("c")
                .value_name("CREDENTIAL_FILE")
                .help("specify the path to a file which contains the username and password for the inwx account seperated by a newline")
                .takes_value(true)
                .required(true)
            )
            .arg(Arg::with_name("domain")
                .short("d")
                .value_name("DOMAIN")
                .help("the domain of the record (i.e. \"_acme-challenge.example.com\"")
                .takes_value(true)
                .required(true)
            )
        );
    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let domain = matches.value_of("domain").unwrap();
        let value = matches.value_of("value").unwrap();
        let (user, pass) = read_credentials(matches.value_of("credentialfile").unwrap())?;

        execute_api_commands(&user, &pass, |api| {
            api.create_txt_record(&domain, &value)?;
            Ok(())
        })?;

        println!("Record has been created successfully.");
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        let domain = matches.value_of("domain").unwrap();
        let (user, pass) = read_credentials(matches.value_of("credentialfile").unwrap())?;

        execute_api_commands(&user, &pass, |api| {
            api.delete_txt_record(&domain)?;
            Ok(())
        })?;

        println!("Record has been deleted successfully.");
    } else {
        app.print_help().unwrap();
        std::process::exit(1);
    }

    Ok(())
}

fn execute_api_commands<F>(user: &str, pass: &str, op: F) -> Result<(), String> where F: Fn(&Inwx) -> Result<(), InwxError> {
    let api = Inwx::new(&user, &pass).map_err(|err| format!("{}", err))?;

    let mut err = None;
    match op(&api) {
        Err(e) => {
            err = Some(e);
        },
        _ => {}
    }

    if let Err(e) = api.logout() {
        if let None = err {
            err = Some(e);
        }
    }

    match err {
        Some(e) => Err(format!("{}", e)),
        None => Ok(())
    }
}

fn main() {
    if let Err(msg) = run() {
        eprintln!("Error: {}", msg);
        exit(1);
    }
}
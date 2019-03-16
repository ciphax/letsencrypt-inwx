use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use std::time::{Duration, Instant};
use clap::{Arg, App, SubCommand};
use crate::config::Config;
use crate::inwx::{Inwx, InwxError};
use crate::dns::{check_txt_record, lookup_real_domain};

impl From<InwxError> for String {
    fn from(inwx_error: InwxError) -> String {
        format!("{}", inwx_error)
    }
}

fn execute_api_commands<F>(config: &Config, domain: &str, op: F) -> Result<bool, String> where F: Fn(&mut Inwx) -> Result<(), InwxError> {
    if config.accounts.len() == 0 {
        return Err("No accounts configured".to_owned());
    }

    let accounts = match(&config.accounts).into_iter().find(|account|
        (&account.domains).into_iter().any(|d| domain == d || domain.ends_with(&format!(".{}", d)))
    ) {
        Some(account) => vec!(account.clone()),
        None => config.accounts.clone()
    };


    for account in accounts {
        let mut success = false;
        let mut api = Inwx::new(&account)?;

        let mut err = None;
        match op(&mut api) {
            Err(InwxError::DomainNotFound) => {},
            Err(e) => {
                err = Some(e);
            },
            _ => {
                success = true;
            }
        }

        if let Err(e) = api.logout() {
            if let None = err {
                err = Some(e);
            }
        }

        if let Some(e) = err {
            return Err(String::from(e));
        } else if success {
            return Ok(account.ote);
        }
    }

    Err(String::from(InwxError::DomainNotFound))
}

fn read_config(path: &str) -> Result<Config, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open config file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader).map_err(|e| format!("Failed to parse config file: {}", e))?)
}

fn create(config: &Config, domain: &str, value: &str) -> Result<(), String> {
    println!("Creating TXT record...");

    let is_ote = execute_api_commands(&config, &domain, |api| {
        api.create_txt_record(&domain, &value)?;
        Ok(())
    })?;

    println!("=> done!");

    if !is_ote && !config.options.no_dns_check {
        println!("Waiting for the dns record to be publicly visible...");

        let start = Instant::now();
        let mut wait_secs = 5;

        loop {
            // timeout after 10 minutes
            if start.elapsed() > Duration::from_secs(60 * 10) {
                return Err("timeout!".to_owned());
            }

            if check_txt_record(&config.options.dns_server, &domain, value) {
                break;
            }

            wait_secs *= 2;

            sleep(Duration::from_secs(wait_secs));
        }

        println!("=> done!");
    }

    if config.options.wait_interval > 0 {
        println!("Waiting {} additional seconds...", &config.options.wait_interval);

        sleep(Duration::from_secs(config.options.wait_interval));

        println!("=> done!");
    }

    Ok(())
}

fn delete(config: &Config, domain: &str) -> Result<(), String> {
    println!("Deleting TXT record...");

    execute_api_commands(&config, &domain, |api| {
        api.delete_txt_record(&domain)?;
        Ok(())
    })?;

    println!("=> done!");

    Ok(())
}

pub fn run() -> Result<(), String> {
    let mut app = App::new("letsencrypt-inwx")
        .version("2.0.0")
        .about("A small cli utility for automating the letsencrypt dns-01 challenge for domains hosted by inwx")
        .subcommand(SubCommand::with_name("create")
            .about("create a TXT record")
            .arg(Arg::with_name("configfile")
                .short("c")
                .value_name("CONFIG_FILE")
                .help("specify the path to the configfile")
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
            .arg(Arg::with_name("configfile")
                .short("c")
                .value_name("CONFIG_FILE")
                .help("specify the path to the configfile")
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
        let config = read_config(matches.value_of("configfile").unwrap())?;
        let domain = lookup_real_domain(&config.options.dns_server, matches.value_of("domain").unwrap());
        let value = matches.value_of("value").unwrap();

        create(&config, &domain, &value)?;
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        let config = read_config(matches.value_of("configfile").unwrap())?;
        let domain = lookup_real_domain(&config.options.dns_server, matches.value_of("domain").unwrap());

        delete(&config, &domain)?;
    } else {
        app.print_help().unwrap();
        std::process::exit(1);
    }

    Ok(())
}

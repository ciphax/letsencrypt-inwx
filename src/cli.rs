use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use std::time::{Duration, Instant};
use clap::{Arg, App, SubCommand};
use crate::config::Config;
use crate::inwx::{Inwx, InwxError};
use crate::dns::{check_txt_record, lookup_real_domain};

fn execute_api_commands<F>(config: &Config, domain: &str, op: F) -> Result<bool, ()> where F: Fn(&mut Inwx) -> Result<(), InwxError> {
    if config.accounts.len() == 0 {
        error!("No accounts configured");
        return Err(());
    }

    let mut filtered_accounts = Vec::new();

    match config.accounts.iter().find(|account|
        account.domains.iter().any(|d| domain == d || domain.ends_with(&format!(".{}", d)))
    ) {
        Some(account) => {
            info!("Using account {}", account.username);
            filtered_accounts.push(account);
        },
        None => {
            warn!("Domain not configured: Trying {} configured domains", config.accounts.len());
            filtered_accounts.extend(config.accounts.iter());
        }
    };


    for account in filtered_accounts {
        let mut success = false;
        let mut api = Inwx::new(&account).map_err(|e| error!("{}", e))?;

        match op(&mut api) {
            Err(InwxError::DomainNotFound) => {},
            Err(e) => {
                error!("{}", e);
                return Err(());
            },
            _ => {
                success = true;
            }
        }

        if let Err(e) = api.logout() {
            error!("{}", e);
        }

        if success {
            return Ok(account.ote);
        }
    }

    error!("{}", InwxError::DomainNotFound);
    Err(())
}

fn read_config(path: &str) -> Result<Config, ()> {
    let file = File::open(path).map_err(|e| error!("Failed to open config file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader).map_err(|e| error!("Failed to parse config file: {}", e))?)
}

fn create(config: &Config, domain: &str, value: &str) -> Result<(), ()> {
    info!("Creating TXT record...");

    let is_ote = execute_api_commands(&config, &domain, |api| {
        api.create_txt_record(&domain, &value)?;
        Ok(())
    })?;

    info!("=> done!");

    if !is_ote && !config.options.no_dns_check {
        info!("Waiting for the dns record to be publicly visible...");

        let start = Instant::now();
        let mut wait_secs = 5;

        loop {
            // timeout after 10 minutes
            if start.elapsed() > Duration::from_secs(60 * 10) {
                error!("=> timeout!");
                return Err(());
            }

            if check_txt_record(&config.options.dns_server, &domain, value) {
                break;
            }

            wait_secs *= 2;

            sleep(Duration::from_secs(wait_secs));
        }

        info!("=> done!");
    }

    if config.options.wait_interval > 0 {
        info!("Waiting {} additional seconds...", &config.options.wait_interval);

        sleep(Duration::from_secs(config.options.wait_interval));

        info!("=> done!");
    }

    Ok(())
}

fn delete(config: &Config, domain: &str) -> Result<(), ()> {
    info!("Deleting TXT record...");

    execute_api_commands(&config, &domain, |api| {
        api.delete_txt_record(&domain)?;
        Ok(())
    })?;

    info!("=> done!");

    Ok(())
}

pub fn run() -> Result<(), ()> {
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

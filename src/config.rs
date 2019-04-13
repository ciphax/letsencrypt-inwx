use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub accounts: Vec<Account>,
    pub options: Options,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            accounts: vec![],
            options: Options::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Account {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub ote: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Options {
    pub no_dns_check: bool,
    pub wait_interval: u64,
    pub dns_server: String,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            no_dns_check: false,
            wait_interval: 5,
            dns_server: "8.8.8.8".to_owned(),
        }
    }
}

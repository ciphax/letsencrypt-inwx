use std::str::FromStr;
use trust_dns::client::{Client, SyncClient};
use trust_dns::udp::UdpClientConnection;
use trust_dns::op::DnsResponse;
use trust_dns::rr::{DNSClass, Name, RData, Record, RecordType};

fn dns_client(dns_server: &str) -> SyncClient<UdpClientConnection> {
    let address = format!("{}:53", dns_server).parse().unwrap();
    let conn = UdpClientConnection::new(address).unwrap();
    SyncClient::new(conn)
}

pub fn add_trailing_dot(domain: &str) -> String {
    let mut domain = domain.to_owned();

    if !domain.ends_with(".") {
        domain += ".";
    }

    domain
}

pub fn remove_trailing_dot(domain: &str) -> String {
    let mut domain = domain.to_owned();

    if domain.ends_with(".") {
        domain.pop();
    }

    domain
}

fn check_cname(dns_server: &str, domain: &str) -> Option<String> {
    let client = dns_client(dns_server);
    let name = Name::from_str(&add_trailing_dot(domain)).ok()?;
    let response: DnsResponse = client.query(&name, DNSClass::IN, RecordType::CNAME).ok()?;
    let answers: &[Record] = response.answers();

    for record in answers {
        if let RData::CNAME(ref cname) = record.rdata() {
            return Some(remove_trailing_dot(&cname.to_utf8()));
        }
    }

    None
}

pub fn lookup_real_domain(dns_server: &str, domain: &str) -> String {
    let mut depth = 0;

    let mut domain = domain.to_owned();
    while let Some(real_name) = check_cname(dns_server, &domain) {
        domain = real_name;

        if depth >= 10 {
            break;
        }

        depth += 1;
    }

    domain
}

pub fn check_txt_record(dns_server: &str, domain: &str, value: &str) -> bool {
    let client = dns_client(dns_server);
    let name = match Name::from_str(&add_trailing_dot(domain)) {
        Ok(name) => name,
        Err(_) => return false
    };

    if let Ok(response) = client.query(&name, DNSClass::IN, RecordType::TXT) {
        for record in response.answers() {
            if record.name().to_utf8().to_lowercase() == name.to_utf8().to_lowercase() {
                if let RData::TXT(data) = record.rdata() {
                    for data in data.txt_data().iter() {
                        let data = String::from_utf8_lossy(data);

                        if data == value {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

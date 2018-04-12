use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver;

pub fn check_txt_record(domain: &str, value: &str) -> bool {
    let mut opts = ResolverOpts::default();
    opts.cache_size = 0;

    let resolver = match Resolver::new(ResolverConfig::default(), opts) {
        Ok(resolver) => resolver,
        _ => return false
    };

    let result = match resolver.txt_lookup(domain) {
        Ok(result) => result,
        _ => return false
    };

    for record in result.iter() {
        for data in record.txt_data().iter() {
            let data = String::from_utf8_lossy(data);
            
            if data == value {
                return true;
            }
        }
    }

    false
}
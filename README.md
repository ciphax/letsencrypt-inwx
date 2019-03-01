# letsencrypt-inwx [![Build Status](https://travis-ci.org/kegato/letsencrypt-inwx.svg?branch=master)](https://travis-ci.org/kegato/letsencrypt-inwx) [![Docker Pulls](https://img.shields.io/docker/pulls/kegato/letsencrypt-inwx.svg)](https://hub.docker.com/r/kegato/letsencrypt-inwx/) [![Crates.io](https://img.shields.io/crates/v/letsencrypt-inwx.svg)](https://crates.io/crates/letsencrypt-inwx)

A small cli utility for automating the letsencrypt dns-01 challenge for domains hosted by inwx. This allows you to obtain wildcard certificates from letsencrypt.

## Installation
### Ubuntu / Debian
- Build the .deb package or download it from [releases](https://github.com/kegato/letsencrypt-inwx/releases/latest) and install it with `sudo dpkg -i <path_to_the_deb_file>`

### Other linux
- Build the executable or download it from [releases](https://github.com/kegato/letsencrypt-inwx/releases/latest) and copy it to `/usr/bin/`
- Copy both certbot scripts from `./etc/` to `/usr/lib/letsencrypt-inwx/`

### With cargo
- Run `cargo install letsencrypt-inwx`

## Configuration
You can store the configuration file at `/etc/letsencrypt-inwx.json` or at `~/.config/letsencrypt-inwx.json` when used with certbot or specify it's path with the `-c` option.
The configuration file should look like this (without the comments):
```json
{
	"accounts": [
		{
			"username": "user",
			"password": "pass",
			// optional, if the domain is not configured all accounts will be tried
			"domains": [
				"example.com"
			],
			// optional, if true the public inwx test server will be used
			"ote": false
		}
	],
	// optional
	"options": {
		// optional, if true letsencrypt-inwx will not wait until the created record is publicly visible, default: false
		"no_dns_check": false,
		// optional, the amount of time in seconds to wait after creating a record, default: 5 seconds
		"wait_interval": 5,
		// optional: the dns server to use, default: the google public dns server
		"dns_server": "8.8.8.8"
	}
}
```

## Usage
### With Docker and certbot
- Generate your certificate by running `docker run --rm -it -v /etc/letsencrypt-inwx.json:/etc/letsencrypt-inwx.json -v /etc/letsencrypt:/etc/letsencrypt kegato/letsencrypt-inwx certonly --email <your_email> --preferred-challenges=dns-01 --manual --manual-auth-hook /usr/lib/letsencrypt-inwx/certbot-inwx-auth --manual-cleanup-hook /usr/lib/letsencrypt-inwx/certbot-inwx-cleanup --manual-public-ip-logging-ok -d <your_domain>`
- You can find your certificate in `/etc/letsencrypt/live/<your_domain>/`
- You can renew your certificate by running `docker run --rm -it -v /etc/letsencrypt-inwx.json:/etc/letsencrypt-inwx.json -v /etc/letsencrypt:/etc/letsencrypt kegato/letsencrypt-inwx renew`

### With certbot
- You can get certificates from [certbot](https://certbot.eff.org/) by running `sudo certbot certonly -n --agree-tos --server https://acme-v02.api.letsencrypt.org/directory --email <your_email> --preferred-challenges=dns-01 --manual --manual-auth-hook /usr/lib/letsencrypt-inwx/certbot-inwx-auth --manual-cleanup-hook /usr/lib/letsencrypt-inwx/certbot-inwx-cleanup --manual-public-ip-logging-ok -d <your_domain>`

### Manually
- Create a txt record with `letsencrypt-inwx create -c <config_file> -d _acme-challenge.<your_domain> -v <acme_token>`
- Delete it with `letsencrypt-inwx delete -c <config_file> -d _acme-challenge.<your_domain>`

## Building
### Requirements
`libssl-dev` and `pkg-config` are required when building on Ubuntu / Debian see [here](https://github.com/sfackler/rust-openssl).

### .deb package
- Install [cargo-deb](https://github.com/mmstick/cargo-deb) by running `cargo install cargo-deb`
- Run `cargo deb` to build the package

### only the executable
- Run `cargo build --release` to build the `letsencrypt-inwx` executable

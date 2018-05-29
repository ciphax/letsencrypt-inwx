# letsencrypt-inwx [![Build Status](https://travis-ci.org/kegato/letsencrypt-inwx.svg?branch=master)](https://travis-ci.org/kegato/letsencrypt-inwx)
A small cli utility for automating the letsencrypt dns-01 challenge for domains hosted by inwx. This allows you to obtain wildcard certificates from letsencrypt.

## Installation
### Ubuntu / Debian
- Build the .deb package or download it from [releases](https://github.com/kegato/letsencrypt-inwx/releases/latest) and install it with `sudo dpkg -i <path_to_the_deb_file>`

### Other linux
- Build the executable and copy it to `/usr/bin/`
- Copy both certbot scripts from `./etc/` to `/usr/lib/letsencrypt-inwx/`

## Usage
### With certbot
- Put your inwx login data seperated by a newline into `/etc/letsencrypt-inwx-cred`
- Make sure the file is only readable for root `sudo chmod 600 /etc/letsencrypt-inwx-cred`
- You can now get certificates from [certbot](https://certbot.eff.org/) by running `sudo certbot certonly -n --agree-tos --email <your_email> --manual --preferred-challenges=dns --manual-auth-hook /usr/lib/letsencrypt-inwx/certbot-inwx-auth --manual-cleanup-hook /usr/lib/letsencrypt-inwx/certbot-inwx-cleanup --manual-public-ip-logging-ok -d <your_domain>`

Note: You need atleast certbot 0.22.0 to issue wildcard certificates.

### Manually
- Put your inwx login data seperated by a newline into a file
- Create a txt record with `letsencrypt-inwx create -c <credential_file> -d _acme-challenge.your-domain.com -v <acme_token>`
- Delete it with `letsencrypt-inwx delete -c <credential_file> -d _acme-challenge.your-domain.com`

### With Docker (supports wildcard certificates)
- Put your inwx login data into the `docker-compose.yml` file
- Also configure there your email and a local folder for persistance
- The domains you want to authorize go there too
- finally run `docker-compose up` (this can take a while)

## Building
### Requirements
`openssl-devel` and `pkg-config` are required when building on Ubuntu / Debian see [here](https://github.com/sfackler/rust-openssl).

### .deb package
- Install [cargo-deb](https://github.com/mmstick/cargo-deb) by running `cargo install cargo-deb`
- Run `cargo deb` to build the package
### only the executable
- Run `cargo build --release` to build the `letsencrypt-inwx` executable
# letsencrypt-inwx [![Build Status](https://travis-ci.org/kegato/letsencrypt-inwx.svg?branch=master)](https://travis-ci.org/kegato/letsencrypt-inwx) [![Docker Build Status](https://img.shields.io/docker/build/kegato/letsencrypt-inwx.svg)](https://hub.docker.com/r/kegato/letsencrypt-inwx/) [![Crates.io](https://img.shields.io/crates/v/letsencrypt-inwx.svg)](https://crates.io/crates/letsencrypt-inwx)

A small cli utility for automating the letsencrypt dns-01 challenge for domains hosted by inwx. This allows you to obtain wildcard certificates from letsencrypt.

## Installation
### Ubuntu / Debian
- Build the .deb package or download it from [releases](https://github.com/kegato/letsencrypt-inwx/releases/latest) and install it with `sudo dpkg -i <path_to_the_deb_file>`

### Other linux
- Build the executable or download it from [releases](https://github.com/kegato/letsencrypt-inwx/releases/latest) and copy it to `/usr/bin/`
- Copy both certbot scripts from `./etc/` to `/usr/lib/letsencrypt-inwx/`

### With cargo
- Run `cargo install letsencrypt-inwx`

## Usage
### With certbot
- Put your inwx login data seperated by a newline into `/etc/letsencrypt-inwx-cred`
- Make sure the file is only readable for root `sudo chmod 600 /etc/letsencrypt-inwx-cred`
- You can now get certificates from [certbot](https://certbot.eff.org/) by running `sudo certbot certonly -n --agree-tos --email <your_email> --server https://acme-v02.api.letsencrypt.org/directory --preferred-challenges=dns-01 --manual --manual-auth-hook /usr/lib/letsencrypt-inwx/certbot-inwx-auth --manual-cleanup-hook /usr/lib/letsencrypt-inwx/certbot-inwx-cleanup --manual-public-ip-logging-ok -d <your_domain>`

#### Notes
- You need atleast certbot 0.22.0 to issue wildcard certificates.
- You can put your inwx login data into `~/.config/letsencrypt-inwx-cred` if you want to run certbot as non-root user

### With Docker and certbot
- Put your inwx login data into a docker env file like this
```sh
INWX_USER=username
INWX_PASSWD=password
```
- Generate your certificate by running `docker run --rm -it --env-file <your-env-file> -v /etc/letsencrypt:/etc/letsencrypt kegato/letsencrypt-inwx certonly --email <your_email> --preferred-challenges=dns-01 --manual --manual-auth-hook /usr/lib/letsencrypt-inwx/certbot-inwx-auth --manual-cleanup-hook /usr/lib/letsencrypt-inwx/certbot-inwx-cleanup --manual-public-ip-logging-ok -d <your_domain>`
- Your certificate is now at `/etc/letsencrypt/live/<your_domain>/`
- You can renew your certificate by running `docker run --rm -it --env-file <your-env-file> -v /etc/letsencrypt:/etc/letsencrypt kegato/letsencrypt-inwx renew`

### Manually
- Put your inwx login data seperated by a newline into a file
- Create a txt record with `letsencrypt-inwx create -c <credential_file> -d _acme-challenge.your-domain.com -v <acme_token>`
- Delete it with `letsencrypt-inwx delete -c <credential_file> -d _acme-challenge.your-domain.com`

## Building
### Requirements
`libssl-dev` and `pkg-config` are required when building on Ubuntu / Debian see [here](https://github.com/sfackler/rust-openssl).

### .deb package
- Install [cargo-deb](https://github.com/mmstick/cargo-deb) by running `cargo install cargo-deb`
- Run `cargo deb` to build the package

### only the executable
- Run `cargo build --release` to build the `letsencrypt-inwx` executable
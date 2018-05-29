FROM rust:1.26.0

VOLUME /etc/letsencrypt
VOLUME /var/log/letsencrypt
WORKDIR /letsencrypt-inwx-src

RUN wget https://dl.eff.org/certbot-auto \
	&& chmod a+x ./certbot-auto \
	&& ./certbot-auto --non-interactive --install-only \
    && ./certbot-auto --version

COPY . .

RUN cargo install \
    && cp target/release/letsencrypt-inwx /usr/bin/letsencrypt-inwx

RUN cp -R etc/ /usr/lib/letsencrypt-inwx/ \
	&& chmod a+x docker-entrypoint.sh /usr/lib/letsencrypt-inwx/*

ENTRYPOINT ["/letsencrypt-inwx-src/docker-entrypoint.sh"]
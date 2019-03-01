FROM ekidd/rust-musl-builder:stable as builder
COPY . .
RUN cargo install cargo-deb
RUN cargo deb --target x86_64-unknown-linux-musl

FROM certbot/certbot:rolling
VOLUME /etc/letsencrypt
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/letsencrypt-inwx /usr/bin/
COPY etc/* /usr/lib/letsencrypt-inwx/
RUN chmod +x /usr/lib/letsencrypt-inwx/*

ENTRYPOINT ["/usr/lib/letsencrypt-inwx/docker-entrypoint.sh"]

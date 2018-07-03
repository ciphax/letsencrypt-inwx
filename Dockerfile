FROM ekidd/rust-musl-builder:stable
ADD . .
RUN cargo install cargo-deb
RUN cargo deb --target x86_64-unknown-linux-musl
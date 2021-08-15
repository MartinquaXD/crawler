# 1: Build the exe
FROM rust:1.54 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup toolchain install nightly && \
    rustup target add x86_64-unknown-linux-musl --toolchain nightly

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN USER=root cargo new crawler
WORKDIR /usr/src/crawler
ADD Cargo.toml Cargo.lock ./
RUN cargo +nightly build --target x86_64-unknown-linux-musl --release
ADD . .

# 1c: update file timesamp otherwise cargo will not properly rebuild
RUN touch ./src/main.rs
RUN cargo +nightly build --target x86_64-unknown-linux-musl --release

# 1d: strip debug symbols to shrink the binary size
RUN strip /usr/src/crawler/target/x86_64-unknown-linux-musl/release/crawler

# 2 create new docker image
FROM alpine:latest

# 2a install ssl certificates
RUN apk update && apk add ca-certificates && rm -rf /var/cache/apk/*
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /usr/local/share/ca-certificates/mycert.crt
RUN update-ca-certificates

COPY --from=builder /usr/src/crawler/target/x86_64-unknown-linux-musl/release/crawler .
USER 1000
CMD ["./crawler"]

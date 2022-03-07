FROM rust:latest as builder
WORKDIR /usr/src/app
COPY Cargo.toml ./
COPY src ./src
RUN cargo install --path .

FROM debian:latest
COPY --from=builder /usr/local/cargo/bin/dkms-resolver /usr/local/bin/dkms-resolver

# API port
EXPOSE 9599
# DHT port
EXPOSE 9145

CMD ["dkms-resolver"]

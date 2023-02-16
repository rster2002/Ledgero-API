FROM rust:slim-buster as builder

RUN cargo new --bin rust-and-docker
WORKDIR /rust-and-docker
COPY ./piggybanks/Cargo.toml ./Cargo.toml
COPY ./piggybanks/src ./src
COPY ./piggybanks/migrations ./migrations
COPY ./piggybanks/sqlx-data.json ./sqlx-data.json
RUN cargo build --release

FROM scratch
COPY --from=builder /rust-and-docker/target/release/piggy_banks_rust ./piggy_banks_rust
CMD ["./piggy_banks_rust"]

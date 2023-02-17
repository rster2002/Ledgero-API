FROM rust:slim-buster as builder

RUN cargo new --bin rust-and-docker
COPY ./piggybanks/Cargo.toml ./Cargo.toml
COPY ./piggybanks/src ./src
COPY ./piggybanks/migrations ./migrations
COPY ./piggybanks/sqlx-data.json ./sqlx-data.json
RUN cargo build --release

FROM ubuntu
COPY --from=builder ./target/release/piggy_banks_rust ./piggy_banks_rust
EXPOSE 8000
CMD ["./piggy_banks_rust"]

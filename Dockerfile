FROM rust:slim-buster as builder

RUN cargo new --bin rust-and-docker
COPY Cargo.toml ./Cargo.toml
COPY src ./src
COPY migrations ./migrations
COPY sqlx-data.json ./sqlx-data.json
RUN cargo build --release

FROM ubuntu
COPY --from=builder ./target/release/ledgero-api ./ledgero-api
EXPOSE 8000
CMD ["./ledgero-api"]

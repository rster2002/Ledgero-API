FROM rust:slim-buster as builder

RUN cargo new --bin rust-and-docker

COPY Cargo.toml ./Cargo.toml

COPY server/Cargo.toml ./server/Cargo.toml
COPY server/src ./server/src
COPY server/migrations ./server/migrations
COPY server/sqlx-data.json ./server/sqlx-data.json

COPY cli/Cargo.toml ./cli/Cargo.toml
COPY cli/src ./cli/src

RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo build --release

FROM ubuntu
COPY --from=builder ./target/release/ledgero-cli ./ledgero-api
EXPOSE 8000
CMD ["./ledgero-api", "start"]

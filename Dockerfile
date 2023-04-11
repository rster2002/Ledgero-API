FROM rust:slim-buster as builder

RUN cargo new --bin rust-and-docker
COPY server/Cargo.toml ./Cargo.toml
COPY server/src ./src
COPY server/migrations ./migrations
COPY sqlx-data.json ./sqlx-data.json
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo build --release

FROM ubuntu
COPY --from=builder ./target/release/ledgero_api ./ledgero_api
EXPOSE 8000
CMD ["./ledgero_api"]

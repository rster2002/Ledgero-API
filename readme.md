# Ledgero API

This repository contains the back-end code for the personal finance application I'm working on.

## Development

To start developing for this repository, follow these steps:

1. Clone the repository.
2. Make sure you've installed the [Rust toolchain](https://www.rust-lang.org/learn/get-started).
3. Install the [sqlx-cli](https://crates.io/crates/sqlx-cli) using cargo.
4. Generate a key file for the JWT signatures using:

   ```bash
   ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key
   ```

5. Make sure you're running a Postgres database (you can use the `docker-compose.yaml` for this)
6. Create an `.env` file using the `.env.example` file and update the values accordingly.
7. Make sure to run the migrations during development using:

   ```bash
   sqlx migrate run
   ```

8. Start the application using:

   ```bash
   cargo run
   ```

## Creating a docker image

The application is build into a docker image for development of the Ledgero-UI and for deploying to production. To build
a docker image, run the `build-images.sh` script.

## Deploying

Deploying a production build is done using the docker image:

1. First, make sure the Postgres database is up.
2. Create an `.env` file using the `.env.example` file and update the values accordingly.
3. Pull and run the docker image. The application will automatically run all required migrations on the database.

#!/bin/bash

cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" || exit

cd server || exit
cargo sqlx prepare

cd .. || exit
docker build -t ledgero-api .

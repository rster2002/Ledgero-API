#!/bin/bash

cargo sqlx prepare
docker build -t ledgero-api .

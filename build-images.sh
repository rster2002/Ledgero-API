#!/bin/bash

cd piggybanks || exit

cargo sqlx prepare

cd .. || exit

docker build -t piggybanks-rust .

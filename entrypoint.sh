#!/bin/bash

if [ -n "$PROXY" ]; then
    PROXY_CMD="--proxy=$PROXY"
    echo "Using proxy $PROXY"
else
    PROXY_CMD=""
fi

# Launch the BDCS API
cd /bdcs-api-rs
cargo run -- --host 0.0.0.0 --port 80 /bdcs-db/metadata.db /composer-UI/public/ /bdcs-recipes/

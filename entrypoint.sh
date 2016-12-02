#!/bin/bash

if [ -n "$PROXY" ]; then
    PROXY_CMD="--proxy=$PROXY"
    echo "Using proxy $PROXY"
else
    PROXY_CMD=""
fi

# Launch the BDCS API
cd /bdcs-api-rs
cp ./examples/recipes/* /bdcs-recipes/
cargo run -- --host 0.0.0.0 --port 4000 /mddb/metadata.db /bdcs-recipes/

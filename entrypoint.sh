#!/bin/bash

# Launch the BDCS API Server on port 4000
cd /bdcs-api-rs
cp ./examples/recipes/* /bdcs-recipes/
./target/debug/bdcs-api-server --host 0.0.0.0 --port 4000 --mockfiles /mockfiles /mddb/metadata.db /bdcs-recipes/

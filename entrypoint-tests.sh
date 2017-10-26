#!/bin/bash

# any failure afterwards will exit the script
set -e

cd /bdcs-api-rs/

make test

for file in target/debug/*tests-*; do
    if [[ "${file: -2}" != ".d" ]]; then
      mkdir -p "target/kcov/$(basename $file)"
      kcov --include-path=./src --verify "target/kcov/$(basename $file)" "$file"
    fi
done

make KCOV=/root/.local/bin/kcov depclose

# submit coverage results if we're in Travis
if [ -n "$TRAVIS_JOB_ID" ]; then
    kcov --coveralls-id=$TRAVIS_JOB_ID --merge target/kcov target/kcov/*
fi

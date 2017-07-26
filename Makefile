default: all

all: bdcs-api

bdcs-api:
	GIT_COMMIT=$(shell git describe) cargo build

doc: bdcs-api
	cargo doc

clean:
	cargo clean

clippy:
	command -v cargo-clippy >/dev/null 2>&1 || cargo install clippy
	cargo clippy -- --cfg test -D warnings -A doc-markdown -A type-complexity

test: bdcs-api clippy
	RUST_BACKTRACE=1 cargo test --features "strict"

depclose: bdcs-api
	wget https://s3.amazonaws.com/weldr/metadata.db
	METADATA_DB=`realpath ./metadata.db` make -C ./tests/depclose-integration/ test

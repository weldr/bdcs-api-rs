clippy:
	command -v cargo-clippy >/dev/null 2>&1 || cargo install clippy
	cargo clippy -- --cfg test -D warnings -A doc-markdown

test:
	RUST_BACKTRACE=1 cargo test --features "strict"

depclose-travis:
	RUST_BACKTRACE=1 cargo build
	wget https://s3.amazonaws.com/atodorov/metadata_centos7.db.gz
	gunzip ./metadata_centos7.db.gz
	METADATA_DB=./metadata_centos7.db make -C ./tests/depclose-integration/ test

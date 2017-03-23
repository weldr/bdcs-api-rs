clippy:
	command -v cargo-clippy >/dev/null 2>&1 || cargo install clippy
	cargo clippy -- --cfg test -D warnings -A doc-markdown

test:
	RUST_BACKTRACE=1 cargo test --features "strict"

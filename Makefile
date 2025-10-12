.PHONY: fmt clippy audit check

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -W clippy::pedantic

audit:
	cargo audit

run:
	cargo build --release
	cargo strip
	cargo run --release

check: fmt clippy audit

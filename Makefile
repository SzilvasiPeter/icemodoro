.PHONY: fmt clippy audit check

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -W clippy::pedantic

audit:
	cargo audit

build:
	cargo build --release
	cargo strip

check: fmt clippy audit

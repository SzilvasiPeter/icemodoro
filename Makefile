.PHONY: fmt clippy audit check

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -W clippy::pedantic

audit:
	cargo audit

check: fmt clippy audit

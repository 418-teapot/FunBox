RUST_CARGO ?= cargo

.PHONY: rust rust-test rust-format rust-check rust-clean

rust:
	$(RUST_CARGO) build --workspace

rust-test:
	$(RUST_CARGO) test --workspace

rust-format:
	$(RUST_CARGO) fmt --all

rust-check:
	$(RUST_CARGO) fmt --all --check
	$(RUST_CARGO) clippy --workspace --all-targets --all-features

rust-clean:
	$(RUST_CARGO) clean

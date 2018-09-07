all: format build examples test

build:
	cargo build --all --all-targets

test:
    RUST_BACKTRACE=1 cargo test --all -- --nocapture

format:
	@cargo fmt --all -- --check >/dev/null || \
	cargo fmt --all

clean:
	cargo clean

examples:
	cargo build --examples

.PHONY: all

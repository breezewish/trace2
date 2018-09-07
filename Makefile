all: format build examples test

build:
	cargo build

test:
	cargo test -- --nocapture

format:
	@cargo fmt --all -- --check >/dev/null || \
	cargo fmt --all

clean:
	cargo clean

examples:
	cargo build --examples

.PHONY: all

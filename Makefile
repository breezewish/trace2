all: format build examples

build:
	cargo build

format:
	@cargo fmt --all -- --check >/dev/null || \
	cargo fmt --all

clean:
	cargo clean

examples:
	cargo build --examples

.PHONY: all

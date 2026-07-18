APP_NAME := coins-rust
CARGO := cargo

-include .env
export

.PHONY: build check test lint lint-fix format format-check clean run scan

build:
	$(CARGO) build --release

check:
	$(CARGO) check

test:
	$(CARGO) test

lint:
	$(CARGO) clippy -- -D warnings

lint-fix:
	$(CARGO) clippy --fix --allow-dirty

format:
	$(CARGO) fmt

format-check:
	$(CARGO) fmt --check

clean:
	$(CARGO) clean

run:
	$(CARGO) run --bin coins-api

scan:
	semgrep scan --config=auto

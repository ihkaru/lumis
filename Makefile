.PHONY: all build check test lint clean dev ui-install ui-build audit

all: check test

build:
	cargo build --release

check:
	bash scripts/check-line-limits.sh
	cargo check --workspace

test:
	cargo test --workspace

lint:
	bash scripts/check-line-limits.sh
	cargo check --workspace

audit:
	bash scripts/check-line-limits.sh
	cargo test --workspace

ui-install:
	cd ui && bun install

ui-build:
	cd ui && bun run build

dev:
	cargo run -p lumis-server

clean:
	cargo clean
	rm -rf ui/dist ui/node_modules

.PHONY: install install-gui
install:
	cargo build --release --target aarch64-apple-darwin
	sudo cp target/aarch64-apple-darwin/release/dbhub /usr/local/bin/dbhub

install-gui:
	cd gui && cargo tauri build

.PHONY: build build-gui test clean
build:
	cargo build

build-gui:
	cargo build -p dbhub-gui

test:
	cargo test

clean:
	cargo clean

release:
	cargo login
	cargo publish --dry-run
	cargo publish
.PHONY: install
install:
	cargo build --release --target aarch64-apple-darwin
	sudo cp target/aarch64-apple-darwin/release/dbhub /usr/local/bin/dbhub

release:
	cargo login

	cargo publish --dry-run

	cargo publish
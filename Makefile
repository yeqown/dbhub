.PHONY: install install-gui package-gui icon
install:
	cargo build --release --target aarch64-apple-darwin
	sudo cp target/aarch64-apple-darwin/release/dbhub /usr/local/bin/dbhub

icon:
	@bash scripts/create-icon.sh

package-gui:
	cargo build --release -p dbhub-gui
	@cd gui && bash ../scripts/build-app.sh

install-gui:
	cargo build --release -p dbhub-gui
	@cd gui && bash ../scripts/build-app.sh
	@cp -R gui/DBHub.app ~/Applications/ && echo "✅ Installed to ~/Applications/DBHub.app"

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
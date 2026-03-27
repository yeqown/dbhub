.PHONY: all icon build-cli install-cli build-gui package-gui install-gui test clean release

all: build-cli build-gui

icon:
	@bash scripts/create-icon.sh

build-cli:
	cargo build --release -p dbhub
	@echo "✅ Built CLI into target/release/dbhub"

install-cli:
	cargo install --path cli
	@echo "✅ Installed CLI to ~/.cargo/bin/dbhub"

build-gui:
	cargo build --release -p dbhub-gui
	@echo "✅ Built GUI into target/release/dbhub-gui"

package-gui: build-gui
	@cd gui && bash ../scripts/build-app.sh
	@echo "✅ Packaged DBHub.app"

install-gui: package-gui
	@cp -R gui/DBHub.app ~/Applications/
	@echo "✅ Installed to ~/Applications/DBHub.app"

test:
	cargo test --workspace

clean:
	cargo clean

release:
	@echo "📦 Publishing dbhub v$(shell cargo metadata --format 1 2>/dev/null | grep '"version":"' | head -1 | cut -d'"' -f4)"
	cargo login
	cargo publish -p dbhub-core
	cargo publish -p dbhub
	@echo "✅ Release complete!"

# MyTerm Makefile
# Provides common build, test, and packaging targets

.PHONY: all build release test clean install uninstall package help dev

# Default target
all: build

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run tests
test:
	cargo test --all-features

# Run tests with coverage
test-coverage:
	cargo tarpaulin --verbose --all-features --workspace --timeout 120

# Run benchmarks
bench:
	cargo bench

# Run clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Format code
format:
	cargo fmt --all

# Check formatting
format-check:
	cargo fmt --all -- --check

# Run security audit
audit:
	cargo audit

# Clean build artifacts
clean:
	cargo clean

# Development setup
dev:
	@echo "Setting up development environment..."
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-tarpaulin

# Install system-wide (requires root)
install: release
	install -Dm755 target/release/myterm $(DESTDIR)/usr/bin/myterm
	install -Dm644 myterm.desktop $(DESTDIR)/usr/share/applications/myterm.desktop

# Uninstall system-wide (requires root)
uninstall:
	rm -f $(DESTDIR)/usr/bin/myterm
	rm -f $(DESTDIR)/usr/share/applications/myterm.desktop

# Create Debian package
package-deb: release
	@echo "Creating Debian package..."
	mkdir -p debian/myterm/DEBIAN
	mkdir -p debian/myterm/usr/bin
	mkdir -p debian/myterm/usr/share/applications
	cp target/release/myterm debian/myterm/usr/bin/
	cp myterm.desktop debian/myterm/usr/share/applications/
	cp debian/control debian/myterm/DEBIAN/
	dpkg-deb --build debian/myterm myterm.deb

# Create RPM package (requires rpmbuild)
package-rpm: release
	@echo "Creating RPM package..."
	mkdir -p ~/rpmbuild/SOURCES ~/rpmbuild/SPECS
	cp target/release/myterm ~/rpmbuild/SOURCES/
	cp myterm.spec ~/rpmbuild/SPECS/
	rpmbuild -ba ~/rpmbuild/SPECS/myterm.spec

# Create AppImage (requires appimagetool)
package-appimage: release
	@echo "Creating AppImage..."
	mkdir -p AppDir/usr/bin AppDir/usr/share/applications
	cp target/release/myterm AppDir/usr/bin/
	cp myterm.desktop AppDir/usr/share/applications/
	cp myterm.png AppDir/myterm.png
	echo '#!/bin/bash\nexec "$${APPDIR}/usr/bin/myterm" "$$@"' > AppDir/AppRun
	chmod +x AppDir/AppRun
	appimagetool AppDir MyTerm.AppImage

# Create Flatpak package
package-flatpak:
	@echo "Creating Flatpak package..."
	flatpak-builder --force-clean build-dir com.github.yuvalk.myterm.yml

# Show help
help:
	@echo "Available targets:"
	@echo "  build         - Build debug version"
	@echo "  release       - Build release version"
	@echo "  test          - Run tests"
	@echo "  test-coverage - Run tests with coverage"
	@echo "  bench         - Run benchmarks"
	@echo "  clippy        - Run clippy linting"
	@echo "  format        - Format code"
	@echo "  format-check  - Check code formatting"
	@echo "  audit         - Run security audit"
	@echo "  clean         - Clean build artifacts"
	@echo "  dev           - Set up development environment"
	@echo "  install       - Install system-wide"
	@echo "  uninstall     - Uninstall system-wide"
	@echo "  package-deb   - Create Debian package"
	@echo "  package-rpm   - Create RPM package"
	@echo "  package-appimage - Create AppImage"
	@echo "  package-flatpak  - Create Flatpak package"
	@echo "  help          - Show this help message"
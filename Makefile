.PHONY: all build run test clean fmt help

all: build

build:
	cargo build

run:
	cargo run

test:
	cargo test

clean:
	cargo clean

fmt:
	cargo fmt

help:
	@echo "Available targets:"
	@echo "  all     - Build the project (default)"
	@echo "  build   - Build the project using cargo"
	@echo "  run     - Run the project using cargo"
	@echo "  test    - Run tests using cargo"
	@echo "  clean   - Clean the build directory"
	@echo "  fmt     - Format the code using cargo fmt"
	@echo "  help    - Show this help message"

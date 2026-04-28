.PHONY: help all clean test test-ci test-quick build release lint fmt check-fmt markdownlint nixie typecheck


TARGET ?= libgauss.rlib

CARGO ?= $(or $(wildcard $(HOME)/.cargo/bin/cargo),cargo)
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
CARGO_FLAGS ?= --workspace --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
MDLINT ?= $(or $(wildcard $(HOME)/.bun/bin/markdownlint-cli2),markdownlint-cli2)
NIXIE ?= nixie

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test ## Perform a comprehensive check of code

clean: ## Remove build artifacts
	$(CARGO) clean

test: ## Run tests (nextest if available, otherwise cargo test)
	@if $(CARGO) nextest --version >/dev/null 2>&1; then \
		RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) nextest run --profile default $(TEST_FLAGS) $(BUILD_JOBS); \
	else \
		echo "cargo-nextest not installed, falling back to cargo test"; \
		RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) test $(TEST_FLAGS) $(BUILD_JOBS); \
	fi

test-ci: ## Run tests with CI profile (stricter settings)
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) nextest run --profile ci $(TEST_FLAGS) $(BUILD_JOBS)

test-quick: ## Run unit tests only (skip GPUI integration tests)
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) nextest run --profile default --lib $(TEST_FLAGS) $(BUILD_JOBS)

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	$(CARGO) clippy $(CLIPPY_FLAGS)
	RUSTFLAGS="$(RUST_FLAGS)" whitaker --all -- $(CARGO_FLAGS)

typecheck:
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt --all
	mdformat-all

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check

markdownlint: ## Lint Markdown files
	$(MDLINT) '**/*.md'

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'

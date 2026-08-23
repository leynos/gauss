.PHONY: help all clean test test-ci test-quick build release lint fmt \
	check-fmt check-integration-test-inventory integration-test-inventory-test \
	integration-test-inventory-format integration-test-inventory-lint \
	integration-test-inventory-pytest \
	markdownlint nixie typecheck \
	spelling spelling-helper-test


TARGET ?= libgauss.rlib

export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/bin:$(HOME)/.bun/bin:$(PATH)

CARGO ?= $(or $(wildcard $(HOME)/.cargo/bin/cargo),cargo)
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
RUSTDOC_FLAGS ?= --cfg docsrs -D warnings
CARGO_FLAGS ?= --workspace --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
MDLINT ?= $(or $(wildcard $(HOME)/.bun/bin/markdownlint-cli2),markdownlint-cli2)
WHITAKER ?= $(or $(wildcard $(HOME)/.local/bin/whitaker),whitaker)
NIXIE ?= nixie
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools
RUFF_VERSION ?= 0.15.12
TYPOS_VERSION ?= 1.48.0
TYPOS = $(UV) tool run typos@$(TYPOS_VERSION)
INTEGRATION_TEST_INVENTORY_FILES = \
	scripts/check_integration_test_inventory.py \
	scripts/tests/test_integration_test_inventory.py \
	scripts/tests/test_integration_test_inventory_cli.py
INTEGRATION_TEST_INVENTORY_TESTS = $(filter scripts/tests/%, $(INTEGRATION_TEST_INVENTORY_FILES))

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test spelling ## Perform a comprehensive check of code

clean: ## Remove build artifacts
	$(CARGO) clean

test: ## Run tests (nextest if available, otherwise cargo test)
	@if $(CARGO) nextest --version >/dev/null 2>&1; then \
		RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) nextest run --profile default $(TEST_FLAGS) $(BUILD_JOBS); \
	else \
		echo "cargo-nextest not installed, falling back to cargo test"; \
		RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) test $(TEST_FLAGS) $(BUILD_JOBS); \
	fi
	RUSTFLAGS="$(RUST_FLAGS)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) test --workspace --doc --all-features $(BUILD_JOBS)

test-ci: ## Run tests with CI profile (stricter settings)
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) nextest run --profile ci $(TEST_FLAGS) $(BUILD_JOBS)

test-quick: ## Run unit tests only (skip GPUI integration tests)
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) nextest run --profile default --lib $(TEST_FLAGS) $(BUILD_JOBS)

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	$(CARGO) clippy $(CLIPPY_FLAGS)
	RUSTFLAGS="$(RUST_FLAGS)" $(WHITAKER) --all -- $(CARGO_FLAGS)

typecheck:
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt --all
	mdformat-all

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check

markdownlint: spelling check-integration-test-inventory ## Lint Markdown files and enforce repository spelling
	$(MDLINT) '**/*.md'

check-integration-test-inventory: integration-test-inventory-test ## Verify documented integration-test counts against Cargo metadata
	@$(UV_ENV) $(UV) run scripts/check_integration_test_inventory.py

.PHONY: integration-test-inventory-test integration-test-inventory-format \
	integration-test-inventory-lint integration-test-inventory-pytest
integration-test-inventory-test: integration-test-inventory-format integration-test-inventory-lint integration-test-inventory-pytest ## Test the integration-test inventory checker

integration-test-inventory-format: ## Check inventory checker formatting
	@$(UV_ENV) $(UV) tool run ruff@$(RUFF_VERSION) format --isolated \
		--target-version py313 --check $(INTEGRATION_TEST_INVENTORY_FILES)

integration-test-inventory-lint: ## Lint the inventory checker
	@$(UV_ENV) $(UV) tool run ruff@$(RUFF_VERSION) check --isolated \
		--target-version py313 $(INTEGRATION_TEST_INVENTORY_FILES)

integration-test-inventory-pytest: ## Test the inventory checker
	@PYTHONPATH=scripts HYPOTHESIS_STORAGE_DIRECTORY=/tmp/gauss-hypothesis \
		$(UV_ENV) $(UV) run --no-project --python 3.13 \
		--with pytest==9.0.2 --with hypothesis==6.151.9 \
		python -m pytest $(INTEGRATION_TEST_INVENTORY_TESTS) \
		-c /dev/null --rootdir=. -p no:cacheprovider

spelling: spelling-helper-test ## Enforce en-GB-oxendict spelling in Markdown prose
	@$(UV_ENV) $(UV) run scripts/generate_typos_config.py
	@git ls-files -z '*.md' | \
		xargs -0 -r env $(UV_ENV) $(TYPOS) --config typos.toml --force-exclude

spelling-helper-test: ## Validate the shared spelling-policy integration
	@$(UV_ENV) $(UV) tool run ruff@$(RUFF_VERSION) format --isolated \
		--target-version py313 --check scripts/generate_typos_config.py \
		scripts/typos_rollout.py scripts/typos_rollout_cache.py \
		scripts/tests/test_typos_rollout.py
	@$(UV_ENV) $(UV) tool run ruff@$(RUFF_VERSION) check --isolated \
		--target-version py313 scripts/generate_typos_config.py \
		scripts/typos_rollout.py scripts/typos_rollout_cache.py \
		scripts/tests/test_typos_rollout.py
	@PYTHONPATH=scripts $(UV_ENV) $(UV) run --no-project --python 3.13 \
		--with pytest==9.0.2 --with pytest-cov==7.0.0 \
		python -m pytest scripts/tests/test_typos_rollout.py \
		-c /dev/null --rootdir=. -p no:cacheprovider \
		--cov=generate_typos_config --cov=typos_rollout \
		--cov=typos_rollout_cache --cov-fail-under=90

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'

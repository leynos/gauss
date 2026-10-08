.PHONY: help all clean test test-ci test-quick build release lint fmt \
	check-fmt check-integration-test-inventory integration-test-inventory-test \
	integration-test-inventory-format integration-test-inventory-lint \
	integration-test-inventory-pytest \
	markdownlint nixie typecheck spelling workflow-contracts spelling-exceptions-test


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
MDLINT ?= $(shell command -v markdownlint-cli2 2>/dev/null || printf '%s' "$$HOME/.bun/bin/markdownlint-cli2")
# `make fmt` and `make check-fmt` call mdtablefix directly. `--git` selects the
# Markdown files Git tracks and `--include-untracked` adds the untracked files
# Git does not ignore, so a new document is formatted before it is staged.
# Both modes need mdtablefix 0.6.0 or later; CI pins the version at the
# install-mdtablefix step.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = --git --include-untracked
MDTABLEFIX_RULES = --wrap --renumber --breaks --ellipsis --fences
WHITAKER ?= $(or $(wildcard $(HOME)/.local/bin/whitaker),whitaker)
NIXIE ?= nixie
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools

# The CV-005 CodeScene contracts live in shared-actions and run from a full
# commit, so a fix is a pin bump. `.github/cv005.toml` holds this repository's
# only parameters.
CV005_CONTRACTS_REF ?= 88977798a5c3bae1549afb99642529488c665276
CV005_CONTRACTS = $(UV_ENV) $(UV) tool run --python 3.13 \
	--from 'git+https://github.com/leynos/shared-actions@$(CV005_CONTRACTS_REF)\#subdirectory=packages/cv005-contracts' \
	cv005-contracts

RUFF_VERSION ?= 0.15.12
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.3
TYPOS_CONFIG_BUILDER = $(UV) tool run --python 3.14 --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder
INTEGRATION_TEST_INVENTORY_FILES = \
	scripts/check_integration_test_inventory.py \
	scripts/tests/test_integration_test_inventory.py \
	scripts/tests/test_integration_test_inventory_cli.py
INTEGRATION_TEST_INVENTORY_TESTS = $(filter scripts/tests/%, $(INTEGRATION_TEST_INVENTORY_FILES))

# The development build standard (concordat rule `rust-build-defaults`):
# the mold linker on Linux. The pin is stable, so the nightly-only parallel
# frontend flag is not used. An assigned RUSTFLAGS replaces every
# `rustflags` table in .cargo/config.toml, so each recipe that sets it
# composes this onto any inherited value (CI's setup-rust exports one),
# except coverage, which stays on the platform linker.
BUILD_HOST_OS := $(shell uname -s)
STANDARD_RUSTFLAGS := $(if $(filter Linux,$(BUILD_HOST_OS)),-Clink-arg=-fuse-ld=mold)

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test spelling workflow-contracts ## Perform a comprehensive check of code

clean: ## Remove build artifacts
	$(CARGO) clean

test: spelling-exceptions-test ## Run tests (nextest if available, otherwise cargo test)
	@if RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)" $(CARGO) nextest --version >/dev/null 2>&1; then \
		RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) nextest run --profile default $(TEST_FLAGS) $(BUILD_JOBS); \
	else \
		echo "cargo-nextest not installed, falling back to cargo test"; \
		RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) test $(TEST_FLAGS) $(BUILD_JOBS); \
	fi
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) test --workspace --doc --all-features $(BUILD_JOBS)

test-ci: ## Run tests with CI profile (stricter settings)
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) nextest run --profile ci $(TEST_FLAGS) $(BUILD_JOBS)

test-quick: ## Run unit tests only (skip GPUI integration tests)
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) nextest run --profile default --lib $(TEST_FLAGS) $(BUILD_JOBS)

target/%/$(TARGET): ## Build binary in debug or release mode
	$(if $(findstring release,$(@)),RUSTFLAGS="$${RUSTFLAGS-}",RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)") $(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)" $(CARGO) doc --workspace --no-deps
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)" $(CARGO) clippy $(CLIPPY_FLAGS)
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(WHITAKER) --all -- $(CARGO_FLAGS)

typecheck:
	RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt --all
	$(MDTABLEFIX) --in-place $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)
	$(MDLINT) --fix "**/*.md"

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check
	$(MDTABLEFIX) --check $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)

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

spelling-exceptions-test: ## Test that local spelling exceptions are exact patterns
	@PYTHONPATH=scripts $(UV_ENV) $(UV) run --no-project --python 3.13 \
		--with pytest==9.0.2 \
		python -m pytest scripts/tests/test_spelling_exceptions.py \
		-c /dev/null --rootdir=. -p no:cacheprovider

spelling: ## Enforce en-GB-oxendict spelling
	$(TYPOS_CONFIG_BUILDER) gate --repository .

workflow-contracts: ## Check the CV-005 CodeScene workflow contract
	$(CV005_CONTRACTS) check --repository .

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'

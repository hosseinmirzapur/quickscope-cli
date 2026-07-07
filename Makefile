# QuickScope Development Makefile
# ==============================
# Fast commands for building, testing, and running QuickScope.

.PHONY: help build check test run web clippy fmt clean db

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the main crate (release)
	cargo build --release

build-dev: ## Build the main crate (debug)
	cargo build

check: ## Check without building
	cargo check

test: ## Run all tests
	cargo test -p quickscope

test-quick: ## Run tests without recompiling unchanged code
	cargo test -p quickscope --no-fail-fast

run: ## Run the TUI
	cargo run

web: ## Run the web server (default port 3000)
	cargo run -- --web

web-port: ## Run web server on custom port: make web-port PORT=8080
	cargo run -- --web --port $(PORT)

clippy: ## Run clippy lints (zero warnings required)
	cargo clippy -- -D warnings

fmt: ## Format all code
	cargo fmt

fmt-check: ## Check formatting without changing files
	cargo fmt --check

clean: ## Clean build artifacts
	cargo clean

audit: ## Check for security vulnerabilities (requires cargo-audit)
	cargo audit

# ── Web frontend ──────────────────────────────────────────────────────

FRONTEND_DIR = web-frontend
TRUNK = trunk

frontend-deps: ## Install wasm target and trunk for frontend builds
	rustup target add wasm32-unknown-unknown
	cargo install trunk

frontend-serve: ## Serve the Leptos frontend in dev mode
	cd $(FRONTEND_DIR) && $(TRUNK) serve

frontend-build: ## Build the Leptos frontend for production
	cd $(FRONTEND_DIR) && $(TRUNK) build --release

frontend-check: ## Check frontend crate compiles (requires wasm32 target)
	cargo check -p quickscope-web --target wasm32-unknown-unknown

# ── Database ──────────────────────────────────────────────────────────

db-init: ## Initialize (or reinitialize) the database
	@echo "Database is auto-created on first run at path from QUICKSCOPE_DB_PATH"
	@echo "Run 'cargo run' to initialize it."

# ── Git ───────────────────────────────────────────────────────────────

git-status: ## Show working tree status
	git status

git-log: ## Show recent commits
	git log --oneline -20

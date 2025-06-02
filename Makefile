# TGraph Bot Rust Edition - Development Makefile
# Provides convenient shortcuts for common development tasks

.PHONY: help setup check test fmt clippy clean build release run dev audit pre-commit-all

# Default target
help: ## Show this help message
	@echo "TGraph Bot Rust Edition - Development Commands"
	@echo "============================================="
	@awk 'BEGIN {FS = ":.*##"}; /^[a-zA-Z_-]+:.*?##/ { printf "  %-15s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

setup: ## Set up development environment
	@echo "🔧 Setting up development environment..."
	@./scripts/setup-dev.sh

check: ## Check code compilation
	@echo "🔍 Checking code compilation..."
	@cargo check --all-targets --all-features

test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test --all-targets --all-features

fmt: ## Format code
	@echo "🎨 Formatting code..."
	@cargo fmt --all

fmt-check: ## Check code formatting
	@echo "🎨 Checking code formatting..."
	@cargo fmt --all -- --check

clippy: ## Run clippy lints
	@echo "🔧 Running clippy lints..."
	@cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean

build: ## Build in debug mode
	@echo "🔨 Building in debug mode..."
	@cargo build --all-targets --all-features

release: ## Build in release mode
	@echo "🚀 Building in release mode..."
	@cargo build --release --all-targets --all-features

run: ## Run the bot in debug mode
	@echo "🤖 Starting TGraph Bot..."
	@cargo run

dev: ## Development mode with file watching
	@echo "👨‍💻 Starting development mode with file watching..."
	@cargo watch -x check -x test -x "run"

audit: ## Run security audit
	@echo "🔒 Running security audit..."
	@cargo audit

coverage: ## Generate code coverage report (Linux only)
	@echo "📊 Generating code coverage report..."
	@cargo tarpaulin --out Html --output-dir target/tarpaulin

doc: ## Generate documentation
	@echo "📚 Generating documentation..."
	@cargo doc --all-features --no-deps --open

pre-commit-all: ## Run all pre-commit hooks
	@echo "🔍 Running all pre-commit hooks..."
	@pre-commit run --all-files

pre-commit-install: ## Install pre-commit hooks
	@echo "⚙️  Installing pre-commit hooks..."
	@pre-commit install

update: ## Update dependencies
	@echo "⬆️  Updating dependencies..."
	@cargo update

# Development workflow shortcuts
quick-check: fmt clippy test ## Quick development check (format, lint, test)

full-check: clean quick-check build ## Full development check

ci-check: fmt-check clippy test ## CI-style checks without modifications

# Release workflow
prepare-release: clean full-check audit ## Prepare for release

# Benchmarking (when implemented)
bench: ## Run benchmarks
	@echo "⚡ Running benchmarks..."
	@cargo bench

# Database and migration commands (for future use)
migrate: ## Run database migrations (placeholder)
	@echo "📊 Database migrations not yet implemented"

# Docker commands (for future use)
docker-build: ## Build Docker image
	@echo "🐳 Docker support not yet implemented"

docker-run: ## Run in Docker container
	@echo "🐳 Docker support not yet implemented"

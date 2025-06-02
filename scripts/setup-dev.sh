#!/bin/bash
# TGraph Bot Rust Edition - Development Environment Setup
# This script sets up the development environment with all necessary tools

set -euo pipefail

echo "🦀 Setting up TGraph Bot Rust Edition development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running in correct directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "crates" ]]; then
    print_error "This script must be run from the root of the TGraph Bot workspace"
    exit 1
fi

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
fi

print_status "Rust installation found: $(rustc --version)"

# Update Rust toolchain
print_status "Updating Rust toolchain..."
rustup update

# Install required Rust components
print_status "Installing Rust components..."
rustup component add rustfmt clippy rust-analyzer

# Install additional targets if needed
print_status "Installing additional targets..."
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    rustup target add x86_64-unknown-linux-gnu
elif [[ "$OSTYPE" == "darwin"* ]]; then
    rustup target add x86_64-apple-darwin
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    rustup target add x86_64-pc-windows-msvc
fi

# Check for Python (needed for pre-commit)
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
else
    print_warning "Python not found. Pre-commit hooks will not be installed."
    PYTHON_CMD=""
fi

# Install pre-commit if Python is available
if [[ -n "$PYTHON_CMD" ]]; then
    print_status "Installing pre-commit..."

    # Try to install pre-commit
    if $PYTHON_CMD -m pip install --user pre-commit; then
        print_success "Pre-commit installed successfully"

        # Install pre-commit hooks
        print_status "Installing pre-commit hooks..."
        if pre-commit install; then
            print_success "Pre-commit hooks installed"
        else
            print_warning "Failed to install pre-commit hooks"
        fi
    else
        print_warning "Failed to install pre-commit. You may need to install it manually."
    fi
else
    print_warning "Skipping pre-commit installation (Python not found)"
fi

# Check project compilation
print_status "Checking project compilation..."
if cargo check --all-targets --all-features; then
    print_success "Project compiles successfully"
else
    print_warning "Project compilation issues detected. Run 'cargo check' for details."
fi

# Run formatting check
print_status "Checking code formatting..."
if cargo fmt -- --check; then
    print_success "Code formatting is correct"
else
    print_warning "Code formatting issues detected. Run 'cargo fmt' to fix."
fi

# Run clippy
print_status "Running clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "No clippy warnings detected"
else
    print_warning "Clippy warnings detected. Review and fix as needed."
fi

# Check for additional tools
print_status "Checking for additional development tools..."

# Check for cargo-watch
if ! command -v cargo-watch &> /dev/null; then
    print_status "Installing cargo-watch for file watching..."
    cargo install cargo-watch
fi

# Check for cargo-audit
if ! command -v cargo-audit &> /dev/null; then
    print_status "Installing cargo-audit for security auditing..."
    cargo install cargo-audit
fi

# Check for cargo-tarpaulin (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]] && ! command -v cargo-tarpaulin &> /dev/null; then
    print_status "Installing cargo-tarpaulin for code coverage..."
    cargo install cargo-tarpaulin
fi

# Create local development config if it doesn't exist
if [[ ! -f "config/config.yml" ]]; then
    if [[ -f "config/config.yml.sample" ]]; then
        print_status "Creating local config from sample..."
        cp config/config.yml.sample config/config.yml
        print_warning "Please edit config/config.yml with your actual configuration"
    fi
fi

# Final status
echo
print_success "Development environment setup complete!"
echo
echo "Next steps:"
echo "1. Edit config/config.yml with your bot token and Tautulli settings"
echo "2. Run 'cargo run' to start the bot"
echo "3. Run 'cargo test' to run the test suite"
echo "4. Use 'cargo watch -x check -x test' for continuous testing during development"
echo
echo "Available commands:"
echo "  cargo check          - Check compilation"
echo "  cargo test           - Run tests"
echo "  cargo clippy         - Run lints"
echo "  cargo fmt            - Format code"
echo "  cargo audit          - Security audit"
echo "  pre-commit run --all - Run all pre-commit hooks"
echo

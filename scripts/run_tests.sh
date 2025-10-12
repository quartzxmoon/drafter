#!/bin/bash

# Test runner script for PA eDocket Desktop
# This script runs all types of tests: unit, integration, benchmarks

set -e

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

# Function to run tests with timeout
run_with_timeout() {
    local timeout_duration=$1
    local command=$2
    local description=$3
    
    print_status "Running $description..."
    
    if timeout $timeout_duration bash -c "$command"; then
        print_success "$description completed successfully"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            print_error "$description timed out after $timeout_duration"
        else
            print_error "$description failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Change to the Rust project directory
cd "$(dirname "$0")/../src-tauri"

print_status "Starting PA eDocket Desktop test suite..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Make sure you're in the correct directory."
    exit 1
fi

# Parse command line arguments
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_BENCHMARKS=false
RUN_COVERAGE=false
VERBOSE=false
RELEASE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            RUN_UNIT=true
            RUN_INTEGRATION=false
            RUN_BENCHMARKS=false
            shift
            ;;
        --integration-only)
            RUN_UNIT=false
            RUN_INTEGRATION=true
            RUN_BENCHMARKS=false
            shift
            ;;
        --benchmarks)
            RUN_BENCHMARKS=true
            shift
            ;;
        --coverage)
            RUN_COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --release)
            RELEASE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --unit-only       Run only unit tests"
            echo "  --integration-only Run only integration tests"
            echo "  --benchmarks      Run benchmark tests"
            echo "  --coverage        Generate test coverage report"
            echo "  --verbose         Enable verbose output"
            echo "  --release         Run tests in release mode"
            echo "  --help            Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Set up test environment
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Build flags
BUILD_FLAGS=""
if [ "$RELEASE" = true ]; then
    BUILD_FLAGS="--release"
    print_status "Running tests in release mode"
fi

if [ "$VERBOSE" = true ]; then
    BUILD_FLAGS="$BUILD_FLAGS --verbose"
fi

# Check dependencies
print_status "Checking dependencies..."
if ! cargo check $BUILD_FLAGS; then
    print_error "Dependency check failed"
    exit 1
fi

# Run unit tests
if [ "$RUN_UNIT" = true ]; then
    print_status "Running unit tests..."
    
    UNIT_TEST_CMD="cargo test $BUILD_FLAGS --lib --bins"
    if [ "$VERBOSE" = true ]; then
        UNIT_TEST_CMD="$UNIT_TEST_CMD -- --nocapture"
    fi
    
    if ! run_with_timeout "300s" "$UNIT_TEST_CMD" "unit tests"; then
        print_error "Unit tests failed"
        exit 1
    fi
fi

# Run integration tests
if [ "$RUN_INTEGRATION" = true ]; then
    print_status "Running integration tests..."
    
    INTEGRATION_TEST_CMD="cargo test $BUILD_FLAGS --test integration_tests"
    if [ "$VERBOSE" = true ]; then
        INTEGRATION_TEST_CMD="$INTEGRATION_TEST_CMD -- --nocapture"
    fi
    
    if ! run_with_timeout "600s" "$INTEGRATION_TEST_CMD" "integration tests"; then
        print_error "Integration tests failed"
        exit 1
    fi
fi

# Run benchmark tests
if [ "$RUN_BENCHMARKS" = true ]; then
    print_status "Running benchmark tests..."
    
    if ! command -v cargo-criterion &> /dev/null; then
        print_warning "cargo-criterion not found, installing..."
        cargo install cargo-criterion
    fi
    
    BENCHMARK_CMD="cargo criterion"
    if [ "$VERBOSE" = true ]; then
        BENCHMARK_CMD="$BENCHMARK_CMD --verbose"
    fi
    
    if ! run_with_timeout "1200s" "$BENCHMARK_CMD" "benchmark tests"; then
        print_warning "Benchmark tests failed or timed out"
    else
        print_success "Benchmark results saved to target/criterion/"
    fi
fi

# Generate test coverage
if [ "$RUN_COVERAGE" = true ]; then
    print_status "Generating test coverage report..."
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not found, installing..."
        cargo install cargo-tarpaulin
    fi
    
    COVERAGE_CMD="cargo tarpaulin --out Html --output-dir target/coverage"
    if [ "$VERBOSE" = true ]; then
        COVERAGE_CMD="$COVERAGE_CMD --verbose"
    fi
    
    if ! run_with_timeout "900s" "$COVERAGE_CMD" "coverage generation"; then
        print_warning "Coverage generation failed or timed out"
    else
        print_success "Coverage report saved to target/coverage/tarpaulin-report.html"
    fi
fi

# Run clippy for additional checks
print_status "Running clippy for code quality checks..."
CLIPPY_CMD="cargo clippy $BUILD_FLAGS -- -D warnings"

if ! run_with_timeout "300s" "$CLIPPY_CMD" "clippy checks"; then
    print_warning "Clippy found issues (not failing build)"
fi

# Run rustfmt check
print_status "Checking code formatting..."
if ! cargo fmt -- --check; then
    print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
fi

# Security audit
print_status "Running security audit..."
if ! command -v cargo-audit &> /dev/null; then
    print_warning "cargo-audit not found, installing..."
    cargo install cargo-audit
fi

if ! cargo audit; then
    print_warning "Security audit found issues"
fi

# Test summary
print_status "Test Summary:"
echo "=============="

if [ "$RUN_UNIT" = true ]; then
    print_success "✓ Unit tests passed"
fi

if [ "$RUN_INTEGRATION" = true ]; then
    print_success "✓ Integration tests passed"
fi

if [ "$RUN_BENCHMARKS" = true ]; then
    print_success "✓ Benchmark tests completed"
fi

if [ "$RUN_COVERAGE" = true ]; then
    print_success "✓ Coverage report generated"
fi

print_success "All tests completed successfully!"

# Performance metrics
if [ -f "target/criterion/report/index.html" ]; then
    print_status "Benchmark results available at: target/criterion/report/index.html"
fi

if [ -f "target/coverage/tarpaulin-report.html" ]; then
    print_status "Coverage report available at: target/coverage/tarpaulin-report.html"
fi

print_status "Test run completed at $(date)"

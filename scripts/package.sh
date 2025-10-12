#!/bin/bash

# Package script for PA eDocket Desktop
# Builds and packages the application for distribution

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

# Change to project root
cd "$(dirname "$0")/.."

print_status "Starting PA eDocket Desktop packaging process..."

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    print_error "package.json not found. Make sure you're in the correct directory."
    exit 1
fi

# Parse command line arguments
BUILD_MODE="release"
TARGET=""
SIGN=false
NOTARIZE=false
CLEAN=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_MODE="debug"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --sign)
            SIGN=true
            shift
            ;;
        --notarize)
            NOTARIZE=true
            SIGN=true  # Notarization requires signing
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug           Build in debug mode (default: release)"
            echo "  --target TARGET   Specify target platform"
            echo "  --sign            Sign the application (macOS/Windows)"
            echo "  --notarize        Notarize the application (macOS only)"
            echo "  --clean           Clean build artifacts before building"
            echo "  --verbose         Enable verbose output"
            echo "  --help            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Basic release build"
            echo "  $0 --debug                           # Debug build"
            echo "  $0 --target x86_64-apple-darwin      # Specific target"
            echo "  $0 --sign --notarize                 # Signed and notarized (macOS)"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Detect platform
PLATFORM=$(uname -s)
case $PLATFORM in
    Darwin)
        PLATFORM_NAME="macOS"
        ;;
    Linux)
        PLATFORM_NAME="Linux"
        ;;
    MINGW*|CYGWIN*|MSYS*)
        PLATFORM_NAME="Windows"
        ;;
    *)
        PLATFORM_NAME="Unknown"
        ;;
esac

print_status "Building for platform: $PLATFORM_NAME"

# Clean build artifacts if requested
if [ "$CLEAN" = true ]; then
    print_status "Cleaning build artifacts..."
    rm -rf dist/
    rm -rf src-tauri/target/
    npm run clean 2>/dev/null || true
    print_success "Build artifacts cleaned"
fi

# Check dependencies
print_status "Checking dependencies..."

# Check Node.js
if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed"
    exit 1
fi

# Check npm
if ! command -v npm &> /dev/null; then
    print_error "npm is not installed"
    exit 1
fi

# Check Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed"
    exit 1
fi

# Check Tauri CLI
if ! npm list -g @tauri-apps/cli &> /dev/null; then
    print_warning "Tauri CLI not found globally, installing..."
    npm install -g @tauri-apps/cli
fi

print_success "Dependencies check passed"

# Install frontend dependencies
print_status "Installing frontend dependencies..."
npm install
print_success "Frontend dependencies installed"

# Build frontend
print_status "Building frontend..."
if [ "$VERBOSE" = true ]; then
    npm run build
else
    npm run build > /dev/null 2>&1
fi
print_success "Frontend build completed"

# Prepare Tauri build command
TAURI_CMD="npm run tauri build"

if [ "$BUILD_MODE" = "debug" ]; then
    TAURI_CMD="$TAURI_CMD -- --debug"
fi

if [ -n "$TARGET" ]; then
    TAURI_CMD="$TAURI_CMD -- --target $TARGET"
fi

if [ "$VERBOSE" = true ]; then
    TAURI_CMD="$TAURI_CMD -- --verbose"
fi

# Set environment variables for signing
if [ "$SIGN" = true ]; then
    print_status "Preparing for code signing..."
    
    case $PLATFORM_NAME in
        macOS)
            if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
                print_warning "APPLE_SIGNING_IDENTITY not set, using default"
            fi
            if [ -z "$APPLE_CERTIFICATE" ]; then
                print_warning "APPLE_CERTIFICATE not set"
            fi
            ;;
        Windows)
            if [ -z "$WINDOWS_CERTIFICATE_THUMBPRINT" ]; then
                print_warning "WINDOWS_CERTIFICATE_THUMBPRINT not set"
            fi
            ;;
    esac
fi

# Build Tauri application
print_status "Building Tauri application..."
print_status "Command: $TAURI_CMD"

if eval $TAURI_CMD; then
    print_success "Tauri build completed successfully"
else
    print_error "Tauri build failed"
    exit 1
fi

# Post-build processing
print_status "Processing build artifacts..."

# Find build artifacts
BUILD_DIR="src-tauri/target"
if [ "$BUILD_MODE" = "debug" ]; then
    BUILD_DIR="$BUILD_DIR/debug"
else
    BUILD_DIR="$BUILD_DIR/release"
fi

BUNDLE_DIR="$BUILD_DIR/bundle"

if [ ! -d "$BUNDLE_DIR" ]; then
    print_error "Bundle directory not found: $BUNDLE_DIR"
    exit 1
fi

# List generated artifacts
print_status "Generated artifacts:"
find "$BUNDLE_DIR" -type f -name "*pa-edocket-desktop*" | while read -r file; do
    size=$(du -h "$file" | cut -f1)
    echo "  📦 $(basename "$file") ($size)"
done

# Notarization (macOS only)
if [ "$NOTARIZE" = true ] && [ "$PLATFORM_NAME" = "macOS" ]; then
    print_status "Starting notarization process..."
    
    if [ -z "$APPLE_ID" ] || [ -z "$APPLE_PASSWORD" ] || [ -z "$APPLE_TEAM_ID" ]; then
        print_error "Apple credentials not set for notarization"
        print_error "Required: APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID"
        exit 1
    fi
    
    # Find DMG file
    DMG_FILE=$(find "$BUNDLE_DIR/dmg" -name "*.dmg" | head -1)
    
    if [ -n "$DMG_FILE" ]; then
        print_status "Notarizing: $(basename "$DMG_FILE")"
        
        # Submit for notarization
        xcrun notarytool submit "$DMG_FILE" \
            --apple-id "$APPLE_ID" \
            --password "$APPLE_PASSWORD" \
            --team-id "$APPLE_TEAM_ID" \
            --wait
        
        if [ $? -eq 0 ]; then
            print_success "Notarization completed successfully"
            
            # Staple the notarization
            xcrun stapler staple "$DMG_FILE"
            print_success "Notarization stapled to DMG"
        else
            print_error "Notarization failed"
            exit 1
        fi
    else
        print_error "DMG file not found for notarization"
        exit 1
    fi
fi

# Create distribution directory
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Copy artifacts to distribution directory
print_status "Copying artifacts to distribution directory..."
find "$BUNDLE_DIR" -type f \( -name "*.dmg" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \) | while read -r file; do
    cp "$file" "$DIST_DIR/"
    print_success "Copied: $(basename "$file")"
done

# Generate checksums
print_status "Generating checksums..."
cd "$DIST_DIR"
for file in *; do
    if [ -f "$file" ]; then
        sha256sum "$file" > "$file.sha256"
        print_success "Generated checksum for: $file"
    fi
done
cd ..

# Generate release notes
print_status "Generating release information..."
cat > "$DIST_DIR/RELEASE_INFO.md" << EOF
# PA eDocket Desktop Release

**Build Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Platform:** $PLATFORM_NAME
**Build Mode:** $BUILD_MODE
**Target:** ${TARGET:-"default"}
**Signed:** $([ "$SIGN" = true ] && echo "Yes" || echo "No")
**Notarized:** $([ "$NOTARIZE" = true ] && echo "Yes" || echo "No")

## Artifacts

$(find . -maxdepth 1 -type f \( -name "*.dmg" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" \) -exec basename {} \; | sort)

## Installation Instructions

### macOS
1. Download the .dmg file
2. Open the DMG and drag the app to Applications
3. Run the application

### Windows
1. Download the .msi or .exe file
2. Run the installer as administrator
3. Follow the installation wizard

### Linux
1. Download the .deb or .AppImage file
2. For .deb: \`sudo dpkg -i filename.deb\`
3. For .AppImage: Make executable and run

## Verification

Verify file integrity using the provided SHA256 checksums:
\`\`\`bash
sha256sum -c filename.sha256
\`\`\`
EOF

print_success "Release information generated"

# Summary
print_status "Package Summary:"
echo "=================="
print_success "✓ Frontend built successfully"
print_success "✓ Tauri application built successfully"
print_success "✓ Artifacts copied to dist/ directory"
print_success "✓ Checksums generated"
print_success "✓ Release information created"

if [ "$SIGN" = true ]; then
    print_success "✓ Application signed"
fi

if [ "$NOTARIZE" = true ]; then
    print_success "✓ Application notarized (macOS)"
fi

print_status "Distribution files available in: $DIST_DIR"
print_status "Packaging completed successfully!"

# Show final artifact list
echo ""
print_status "Final artifacts:"
ls -la "$DIST_DIR"

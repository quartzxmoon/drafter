#!/bin/bash

# PA eDocket Desktop Production Build Script
# Builds signed installers for all platforms

set -e

echo "🚀 PA eDocket Desktop Production Build"
echo "======================================"

# Configuration
VERSION=$(node -p "require('./package.json').version")
BUILD_DIR="./dist"
RELEASE_DIR="./release"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "Version: $VERSION"
echo "Build ID: $TIMESTAMP"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf "$BUILD_DIR"
rm -rf "$RELEASE_DIR"
mkdir -p "$BUILD_DIR"
mkdir -p "$RELEASE_DIR"

# Environment check
echo "🔍 Checking environment..."
if [ ! -f ".env" ]; then
    echo "❌ .env file not found. Please create from .env.example"
    exit 1
fi

# Load environment variables
source .env

# Verify required environment variables
required_vars=("COURTLISTENER_API_TOKEN" "GOVINFO_API_KEY" "DATABASE_URL")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ Required environment variable $var is not set"
        exit 1
    fi
done

echo "✅ Environment check passed"

# Install dependencies
echo "📦 Installing dependencies..."
npm ci --production=false

# Run tests
echo "🧪 Running tests..."
npm run test || {
    echo "❌ Tests failed. Aborting build."
    exit 1
}

# Build frontend
echo "🏗️  Building frontend..."
npm run build

# Verify frontend build
if [ ! -d "dist" ]; then
    echo "❌ Frontend build failed - dist directory not found"
    exit 1
fi

echo "✅ Frontend build completed"

# Build Tauri applications
echo "🦀 Building Tauri applications..."

cd src-tauri

# Verify Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust toolchain."
    exit 1
fi

# Install Tauri CLI if not present
if ! command -v cargo-tauri &> /dev/null; then
    echo "📦 Installing Tauri CLI..."
    cargo install tauri-cli
fi

# Build for current platform first
echo "🔨 Building for current platform..."
cargo tauri build

# Check if we should build for other platforms
if [ "$BUILD_ALL_PLATFORMS" = "true" ]; then
    echo "🌍 Building for all platforms..."
    
    # macOS builds
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "🍎 Building for macOS (x64 and ARM64)..."
        cargo tauri build --target x86_64-apple-darwin
        cargo tauri build --target aarch64-apple-darwin
    fi
    
    # Windows build (if on Windows or with cross-compilation)
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]] || [ "$CROSS_COMPILE" = "true" ]; then
        echo "🪟 Building for Windows..."
        cargo tauri build --target x86_64-pc-windows-msvc
    fi
    
    # Linux build
    if [[ "$OSTYPE" == "linux-gnu"* ]] || [ "$CROSS_COMPILE" = "true" ]; then
        echo "🐧 Building for Linux..."
        cargo tauri build --target x86_64-unknown-linux-gnu
    fi
fi

cd ..

# Copy build artifacts
echo "📋 Copying build artifacts..."

# Find and copy all built applications
find src-tauri/target -name "*.dmg" -o -name "*.msi" -o -name "*.deb" -o -name "*.AppImage" | while read file; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        cp "$file" "$RELEASE_DIR/$filename"
        echo "  ✅ Copied $filename"
    fi
done

# Generate checksums
echo "🔐 Generating checksums..."
cd "$RELEASE_DIR"
for file in *; do
    if [ -f "$file" ]; then
        sha256sum "$file" > "$file.sha256"
        echo "  ✅ Generated checksum for $file"
    fi
done
cd ..

# Code signing (if certificates are available)
echo "✍️  Code signing..."

if [ -n "$APPLE_CERTIFICATE" ] && [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 Signing macOS applications..."
    for dmg in "$RELEASE_DIR"/*.dmg; do
        if [ -f "$dmg" ]; then
            codesign --sign "$APPLE_CERTIFICATE" --timestamp "$dmg"
            echo "  ✅ Signed $(basename "$dmg")"
            
            # Notarize if credentials are available
            if [ -n "$APPLE_ID" ] && [ -n "$APPLE_PASSWORD" ]; then
                echo "📋 Notarizing $(basename "$dmg")..."
                xcrun notarytool submit "$dmg" \
                    --apple-id "$APPLE_ID" \
                    --password "$APPLE_PASSWORD" \
                    --team-id "$APPLE_TEAM_ID" \
                    --wait
                echo "  ✅ Notarized $(basename "$dmg")"
            fi
        fi
    done
fi

if [ -n "$WINDOWS_CERTIFICATE" ] && [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    echo "🪟 Signing Windows applications..."
    for msi in "$RELEASE_DIR"/*.msi; do
        if [ -f "$msi" ]; then
            signtool sign /f "$WINDOWS_CERTIFICATE" /p "$WINDOWS_CERTIFICATE_PASSWORD" \
                /t http://timestamp.digicert.com "$msi"
            echo "  ✅ Signed $(basename "$msi")"
        fi
    done
fi

# Generate release manifest
echo "📄 Generating release manifest..."
cat > "$RELEASE_DIR/manifest.json" << EOF
{
  "version": "$VERSION",
  "build_id": "$TIMESTAMP",
  "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": [
EOF

first=true
for file in "$RELEASE_DIR"/*; do
    if [[ "$file" == *.dmg ]] || [[ "$file" == *.msi ]] || [[ "$file" == *.deb ]] || [[ "$file" == *.AppImage ]]; then
        filename=$(basename "$file")
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        checksum=$(cat "$file.sha256" | cut -d' ' -f1)
        
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$RELEASE_DIR/manifest.json"
        fi
        
        cat >> "$RELEASE_DIR/manifest.json" << EOF
    {
      "filename": "$filename",
      "size": $size,
      "sha256": "$checksum",
      "platform": "$(echo "$filename" | sed -E 's/.*\.(dmg|msi|deb|AppImage)$/\1/')"
    }
EOF
    fi
done

cat >> "$RELEASE_DIR/manifest.json" << EOF
  ],
  "requirements": {
    "min_os_version": {
      "macos": "10.15",
      "windows": "10.0",
      "linux": "Ubuntu 20.04"
    }
  }
}
EOF

# Generate SBOM (Software Bill of Materials)
echo "📋 Generating SBOM..."
npm list --json > "$RELEASE_DIR/sbom-frontend.json"
cd src-tauri
cargo tree --format json > "../$RELEASE_DIR/sbom-backend.json"
cd ..

# Create release archive
echo "📦 Creating release archive..."
cd "$RELEASE_DIR"
tar -czf "pa-edocket-desktop-$VERSION-$TIMESTAMP.tar.gz" *
cd ..

# Generate update manifest for auto-updater
echo "🔄 Generating update manifest..."
cat > "$RELEASE_DIR/update-manifest.json" << EOF
{
  "version": "$VERSION",
  "notes": "Production release $VERSION",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
EOF

first=true
for file in "$RELEASE_DIR"/*; do
    if [[ "$file" == *.dmg ]]; then
        if [ "$first" = true ]; then first=false; else echo "," >> "$RELEASE_DIR/update-manifest.json"; fi
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        checksum=$(cat "$file.sha256" | cut -d' ' -f1)
        echo "    \"darwin-x86_64\": { \"signature\": \"$checksum\", \"url\": \"$(basename "$file")\", \"size\": $size }" >> "$RELEASE_DIR/update-manifest.json"
    elif [[ "$file" == *.msi ]]; then
        if [ "$first" = true ]; then first=false; else echo "," >> "$RELEASE_DIR/update-manifest.json"; fi
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        checksum=$(cat "$file.sha256" | cut -d' ' -f1)
        echo "    \"windows-x86_64\": { \"signature\": \"$checksum\", \"url\": \"$(basename "$file")\", \"size\": $size }" >> "$RELEASE_DIR/update-manifest.json"
    elif [[ "$file" == *.AppImage ]]; then
        if [ "$first" = true ]; then first=false; else echo "," >> "$RELEASE_DIR/update-manifest.json"; fi
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        checksum=$(cat "$file.sha256" | cut -d' ' -f1)
        echo "    \"linux-x86_64\": { \"signature\": \"$checksum\", \"url\": \"$(basename "$file")\", \"size\": $size }" >> "$RELEASE_DIR/update-manifest.json"
    fi
done

echo "  }" >> "$RELEASE_DIR/update-manifest.json"
echo "}" >> "$RELEASE_DIR/update-manifest.json"

# Build summary
echo ""
echo "🎉 Build completed successfully!"
echo "================================"
echo "Version: $VERSION"
echo "Build ID: $TIMESTAMP"
echo "Artifacts location: $RELEASE_DIR"
echo ""
echo "📦 Built packages:"
ls -la "$RELEASE_DIR"/*.dmg "$RELEASE_DIR"/*.msi "$RELEASE_DIR"/*.deb "$RELEASE_DIR"/*.AppImage 2>/dev/null || echo "  (Platform-specific packages)"
echo ""
echo "🔐 Checksums and signatures generated"
echo "📋 Release manifest created"
echo "🔄 Update manifest created"
echo ""
echo "✅ Ready for distribution!"

# Optional: Upload to release server
if [ "$UPLOAD_RELEASE" = "true" ] && [ -n "$RELEASE_SERVER" ]; then
    echo "📤 Uploading to release server..."
    # Implementation would depend on your release server setup
    echo "  Upload functionality not implemented in this script"
fi

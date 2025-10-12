#!/bin/bash

# Setup script for code signing and auto-updater keys
# Generates Tauri signing keys and sets up signing certificates

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

print_status "Setting up signing configuration for PA eDocket Desktop..."

# Create keys directory
KEYS_DIR=".keys"
mkdir -p "$KEYS_DIR"

# Generate Tauri updater keys
print_status "Generating Tauri updater keys..."

if [ ! -f "$KEYS_DIR/tauri.key" ]; then
    # Check if tauri CLI is available
    if ! command -v tauri &> /dev/null; then
        if ! npm list -g @tauri-apps/cli &> /dev/null; then
            print_status "Installing Tauri CLI..."
            npm install -g @tauri-apps/cli
        fi
        TAURI_CMD="npx tauri"
    else
        TAURI_CMD="tauri"
    fi
    
    # Generate the key pair
    $TAURI_CMD signer generate -w "$KEYS_DIR/tauri.key"
    
    if [ -f "$KEYS_DIR/tauri.key" ] && [ -f "$KEYS_DIR/tauri.key.pub" ]; then
        print_success "Tauri updater keys generated successfully"
        
        # Read the public key for display
        PUBKEY=$(cat "$KEYS_DIR/tauri.key.pub")
        print_status "Public key: $PUBKEY"
        
        # Update tauri.conf.json with the public key
        print_status "Updating tauri.conf.json with public key..."
        
        # Create a temporary file with the updated config
        jq --arg pubkey "$PUBKEY" '.updater.pubkey = $pubkey' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
        mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
        
        print_success "tauri.conf.json updated with public key"
    else
        print_error "Failed to generate Tauri updater keys"
        exit 1
    fi
else
    print_warning "Tauri updater keys already exist"
    PUBKEY=$(cat "$KEYS_DIR/tauri.key.pub")
    print_status "Existing public key: $PUBKEY"
fi

# Create environment file template
print_status "Creating environment file template..."

cat > .env.signing << 'EOF'
# Tauri Signing Configuration
# Copy this file to .env.local and fill in the values

# Tauri updater private key (generated automatically)
TAURI_PRIVATE_KEY_PATH=.keys/tauri.key
TAURI_KEY_PASSWORD=

# GitHub token for releases
GITHUB_TOKEN=

# macOS Code Signing (for macOS builds)
APPLE_CERTIFICATE=
APPLE_CERTIFICATE_PASSWORD=
APPLE_SIGNING_IDENTITY=
APPLE_ID=
APPLE_PASSWORD=
APPLE_TEAM_ID=
KEYCHAIN_PASSWORD=

# Windows Code Signing (for Windows builds)
WINDOWS_CERTIFICATE_THUMBPRINT=
WINDOWS_CERTIFICATE_PASSWORD=
WINDOWS_TIMESTAMP_URL=http://timestamp.digicert.com

# Linux Code Signing (optional)
GPG_PRIVATE_KEY=
GPG_PASSPHRASE=
EOF

print_success "Environment template created: .env.signing"

# Create GitHub Actions secrets documentation
print_status "Creating GitHub Actions secrets documentation..."

cat > SIGNING_SETUP.md << EOF
# Code Signing Setup for PA eDocket Desktop

This document explains how to set up code signing for the PA eDocket Desktop application.

## Generated Files

- \`.keys/tauri.key\` - Private key for Tauri updater (keep secret!)
- \`.keys/tauri.key.pub\` - Public key for Tauri updater (embedded in app)
- \`.env.signing\` - Template for environment variables

## GitHub Actions Secrets

Add the following secrets to your GitHub repository:

### Required for All Platforms

\`\`\`
TAURI_PRIVATE_KEY=$(cat .keys/tauri.key | base64 -w 0)
TAURI_KEY_PASSWORD=your_key_password_if_set
GITHUB_TOKEN=your_github_token_with_repo_access
\`\`\`

### macOS Code Signing

\`\`\`
APPLE_CERTIFICATE=base64_encoded_p12_certificate
APPLE_CERTIFICATE_PASSWORD=certificate_password
APPLE_SIGNING_IDENTITY=Developer ID Application: Your Name (TEAM_ID)
APPLE_ID=your_apple_id_email
APPLE_PASSWORD=app_specific_password
APPLE_TEAM_ID=your_team_id
KEYCHAIN_PASSWORD=temporary_keychain_password
\`\`\`

### Windows Code Signing

\`\`\`
WINDOWS_CERTIFICATE_THUMBPRINT=certificate_thumbprint
WINDOWS_CERTIFICATE_PASSWORD=certificate_password
\`\`\`

## Setting Up Certificates

### macOS

1. **Get a Developer ID Certificate:**
   - Join the Apple Developer Program
   - Create a Developer ID Application certificate
   - Download the certificate as a .p12 file

2. **Prepare the certificate:**
   \`\`\`bash
   # Convert to base64
   base64 -i certificate.p12 -o certificate.base64
   
   # Copy the content to APPLE_CERTIFICATE secret
   cat certificate.base64
   \`\`\`

3. **Get App-Specific Password:**
   - Go to appleid.apple.com
   - Sign in and go to Security section
   - Generate an app-specific password
   - Use this as APPLE_PASSWORD

### Windows

1. **Get a Code Signing Certificate:**
   - Purchase from a CA (DigiCert, Sectigo, etc.)
   - Or use a self-signed certificate for testing

2. **Install the certificate:**
   \`\`\`powershell
   # Import certificate to Windows certificate store
   Import-PfxCertificate -FilePath certificate.pfx -CertStoreLocation Cert:\\CurrentUser\\My
   
   # Get thumbprint
   Get-ChildItem -Path Cert:\\CurrentUser\\My | Where-Object {$_.Subject -like "*Your Name*"}
   \`\`\`

### Linux (Optional)

1. **Create GPG key:**
   \`\`\`bash
   gpg --full-generate-key
   gpg --export-secret-keys --armor your_email > private.key
   \`\`\`

## Local Development

1. **Copy environment template:**
   \`\`\`bash
   cp .env.signing .env.local
   \`\`\`

2. **Fill in the values in .env.local**

3. **Source the environment:**
   \`\`\`bash
   source .env.local
   \`\`\`

4. **Build with signing:**
   \`\`\`bash
   ./scripts/package.sh --sign
   \`\`\`

## Security Notes

- **Never commit private keys to version control**
- **Use strong passwords for certificates**
- **Rotate keys regularly**
- **Use GitHub's encrypted secrets for CI/CD**
- **Limit access to signing certificates**

## Troubleshooting

### macOS Notarization Issues

1. **Check notarization status:**
   \`\`\`bash
   xcrun notarytool history --apple-id your@email.com --password app_password --team-id TEAM_ID
   \`\`\`

2. **Get detailed logs:**
   \`\`\`bash
   xcrun notarytool log submission_id --apple-id your@email.com --password app_password --team-id TEAM_ID
   \`\`\`

### Windows Signing Issues

1. **Verify certificate:**
   \`\`\`powershell
   Get-AuthenticodeSignature path\\to\\file.exe
   \`\`\`

2. **Check timestamp server:**
   \`\`\`bash
   curl -I http://timestamp.digicert.com
   \`\`\`

### Tauri Updater Issues

1. **Verify public key in config:**
   \`\`\`bash
   jq '.updater.pubkey' src-tauri/tauri.conf.json
   \`\`\`

2. **Test key pair:**
   \`\`\`bash
   echo "test" | tauri signer sign --private-key .keys/tauri.key
   \`\`\`

## References

- [Tauri Code Signing Guide](https://tauri.app/v1/guides/distribution/sign-your-application)
- [Apple Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Windows Code Signing](https://docs.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)
EOF

print_success "Signing setup documentation created: SIGNING_SETUP.md"

# Create .gitignore entries for sensitive files
print_status "Updating .gitignore for signing files..."

if [ ! -f .gitignore ]; then
    touch .gitignore
fi

# Add signing-related entries to .gitignore if not already present
grep -q "^\.keys/" .gitignore 2>/dev/null || echo ".keys/" >> .gitignore
grep -q "^\.env\.local" .gitignore 2>/dev/null || echo ".env.local" >> .gitignore
grep -q "^\.env\.signing" .gitignore 2>/dev/null || echo ".env.signing" >> .gitignore
grep -q "^certificate\." .gitignore 2>/dev/null || echo "certificate.*" >> .gitignore
grep -q "^\*\.p12" .gitignore 2>/dev/null || echo "*.p12" >> .gitignore
grep -q "^\*\.pfx" .gitignore 2>/dev/null || echo "*.pfx" >> .gitignore

print_success ".gitignore updated with signing file patterns"

# Create a verification script
print_status "Creating verification script..."

cat > scripts/verify-signing.sh << 'EOF'
#!/bin/bash

# Verification script for signed applications

set -e

print_status() {
    echo -e "\033[0;34m[INFO]\033[0m $1"
}

print_success() {
    echo -e "\033[0;32m[SUCCESS]\033[0m $1"
}

print_error() {
    echo -e "\033[0;31m[ERROR]\033[0m $1"
}

if [ $# -eq 0 ]; then
    print_error "Usage: $0 <path_to_signed_file>"
    exit 1
fi

FILE_PATH="$1"

if [ ! -f "$FILE_PATH" ]; then
    print_error "File not found: $FILE_PATH"
    exit 1
fi

print_status "Verifying signature for: $(basename "$FILE_PATH")"

# Detect file type and verify accordingly
case "$FILE_PATH" in
    *.dmg|*.app)
        print_status "Verifying macOS signature..."
        codesign -v -v "$FILE_PATH"
        if [ $? -eq 0 ]; then
            print_success "macOS signature is valid"
            
            # Check notarization
            spctl -a -v "$FILE_PATH"
            if [ $? -eq 0 ]; then
                print_success "macOS notarization is valid"
            else
                print_error "macOS notarization check failed"
            fi
        else
            print_error "macOS signature verification failed"
        fi
        ;;
    *.exe|*.msi)
        print_status "Verifying Windows signature..."
        if command -v osslsigncode &> /dev/null; then
            osslsigncode verify "$FILE_PATH"
            if [ $? -eq 0 ]; then
                print_success "Windows signature is valid"
            else
                print_error "Windows signature verification failed"
            fi
        else
            print_error "osslsigncode not found. Install it to verify Windows signatures."
        fi
        ;;
    *.deb)
        print_status "Verifying Debian package..."
        dpkg-sig --verify "$FILE_PATH"
        if [ $? -eq 0 ]; then
            print_success "Debian package signature is valid"
        else
            print_error "Debian package signature verification failed"
        fi
        ;;
    *.AppImage)
        print_status "AppImage files are not typically signed"
        print_status "Checking file integrity..."
        if [ -f "$FILE_PATH.sha256" ]; then
            sha256sum -c "$FILE_PATH.sha256"
            if [ $? -eq 0 ]; then
                print_success "AppImage checksum is valid"
            else
                print_error "AppImage checksum verification failed"
            fi
        else
            print_error "No checksum file found for AppImage"
        fi
        ;;
    *)
        print_error "Unknown file type: $FILE_PATH"
        exit 1
        ;;
esac

print_status "Verification completed"
EOF

chmod +x scripts/verify-signing.sh
print_success "Verification script created: scripts/verify-signing.sh"

# Summary
print_status "Setup Summary:"
echo "=================="
print_success "✓ Tauri updater keys generated"
print_success "✓ Environment template created (.env.signing)"
print_success "✓ GitHub Actions documentation created (SIGNING_SETUP.md)"
print_success "✓ .gitignore updated with signing patterns"
print_success "✓ Verification script created (scripts/verify-signing.sh)"

echo ""
print_status "Next Steps:"
echo "1. Review SIGNING_SETUP.md for detailed instructions"
echo "2. Set up certificates for your target platforms"
echo "3. Configure GitHub Actions secrets"
echo "4. Copy .env.signing to .env.local and fill in values for local development"
echo "5. Test signing with: ./scripts/package.sh --sign"

print_warning "Important: Keep your private keys secure and never commit them to version control!"

print_success "Signing setup completed successfully!"

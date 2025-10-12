#!/bin/bash

# Deployment script for PA eDocket Desktop
# Handles version bumping, tagging, and release creation

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

print_status "Starting PA eDocket Desktop deployment process..."

# Parse command line arguments
VERSION=""
RELEASE_TYPE=""
DRY_RUN=false
SKIP_TESTS=false
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --major)
            RELEASE_TYPE="major"
            shift
            ;;
        --minor)
            RELEASE_TYPE="minor"
            shift
            ;;
        --patch)
            RELEASE_TYPE="patch"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version VERSION    Specify exact version (e.g., 1.2.3)"
            echo "  --major              Bump major version (X.0.0)"
            echo "  --minor              Bump minor version (x.Y.0)"
            echo "  --patch              Bump patch version (x.y.Z)"
            echo "  --dry-run            Show what would be done without making changes"
            echo "  --skip-tests         Skip running tests before deployment"
            echo "  --force              Force deployment even if working directory is dirty"
            echo "  --help               Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --patch                    # Bump patch version"
            echo "  $0 --version 1.2.3           # Set specific version"
            echo "  $0 --minor --dry-run         # Preview minor version bump"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate arguments
if [ -z "$VERSION" ] && [ -z "$RELEASE_TYPE" ]; then
    print_error "Must specify either --version or release type (--major, --minor, --patch)"
    exit 1
fi

if [ -n "$VERSION" ] && [ -n "$RELEASE_TYPE" ]; then
    print_error "Cannot specify both --version and release type"
    exit 1
fi

# Check if we're on the main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$FORCE" != true ]; then
    print_error "Must be on main branch for deployment (current: $CURRENT_BRANCH)"
    print_error "Use --force to override this check"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ] && [ "$FORCE" != true ]; then
    print_error "Working directory is not clean. Commit or stash changes first."
    print_error "Use --force to override this check"
    exit 1
fi

# Pull latest changes
print_status "Pulling latest changes from origin..."
if [ "$DRY_RUN" != true ]; then
    git pull origin main
fi

# Get current version
CURRENT_VERSION=$(jq -r '.version' package.json)
print_status "Current version: $CURRENT_VERSION"

# Calculate new version
if [ -n "$VERSION" ]; then
    NEW_VERSION="$VERSION"
else
    # Parse current version
    IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
    MAJOR=${VERSION_PARTS[0]}
    MINOR=${VERSION_PARTS[1]}
    PATCH=${VERSION_PARTS[2]}
    
    case $RELEASE_TYPE in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
    esac
    
    NEW_VERSION="$MAJOR.$MINOR.$PATCH"
fi

print_status "New version: $NEW_VERSION"

# Validate version format
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format: $NEW_VERSION (expected: X.Y.Z)"
    exit 1
fi

# Check if version already exists
if git tag | grep -q "^v$NEW_VERSION$"; then
    print_error "Version v$NEW_VERSION already exists"
    exit 1
fi

# Run tests unless skipped
if [ "$SKIP_TESTS" != true ]; then
    print_status "Running tests..."
    if [ "$DRY_RUN" != true ]; then
        ./scripts/run_tests.sh --unit --integration
        if [ $? -ne 0 ]; then
            print_error "Tests failed. Fix issues before deploying."
            exit 1
        fi
        print_success "All tests passed"
    else
        print_status "Would run tests (skipped in dry-run mode)"
    fi
else
    print_warning "Skipping tests as requested"
fi

# Update version in files
print_status "Updating version in project files..."

if [ "$DRY_RUN" != true ]; then
    # Update package.json
    jq --arg version "$NEW_VERSION" '.version = $version' package.json > package.json.tmp
    mv package.json.tmp package.json
    
    # Update Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
    rm src-tauri/Cargo.toml.bak
    
    # Update tauri.conf.json
    jq --arg version "$NEW_VERSION" '.version = $version' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
    mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
    
    print_success "Version updated in project files"
else
    print_status "Would update version in:"
    print_status "  - package.json"
    print_status "  - src-tauri/Cargo.toml"
    print_status "  - src-tauri/tauri.conf.json"
fi

# Update changelog
print_status "Updating changelog..."

if [ "$DRY_RUN" != true ]; then
    # Get current date
    RELEASE_DATE=$(date +%Y-%m-%d)
    
    # Update changelog
    sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $RELEASE_DATE/" CHANGELOG.md
    rm CHANGELOG.md.bak
    
    print_success "Changelog updated"
else
    print_status "Would update CHANGELOG.md with release date"
fi

# Build the application
print_status "Building application..."

if [ "$DRY_RUN" != true ]; then
    npm run build
    if [ $? -ne 0 ]; then
        print_error "Frontend build failed"
        exit 1
    fi
    
    # Test Tauri build
    npm run tauri build
    if [ $? -ne 0 ]; then
        print_error "Tauri build failed"
        exit 1
    fi
    
    print_success "Application built successfully"
else
    print_status "Would build frontend and Tauri application"
fi

# Commit changes
print_status "Committing version changes..."

if [ "$DRY_RUN" != true ]; then
    git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
    git commit -m "chore: bump version to $NEW_VERSION"
    print_success "Changes committed"
else
    print_status "Would commit version changes"
fi

# Create and push tag
print_status "Creating and pushing tag..."

if [ "$DRY_RUN" != true ]; then
    git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"
    git push origin main
    git push origin "v$NEW_VERSION"
    print_success "Tag v$NEW_VERSION created and pushed"
else
    print_status "Would create and push tag v$NEW_VERSION"
fi

# Wait for GitHub Actions
if [ "$DRY_RUN" != true ]; then
    print_status "GitHub Actions will now build and create the release..."
    print_status "Monitor progress at: https://github.com/quartzxmoon/drafter/actions"
    
    # Wait a moment for the workflow to start
    sleep 5
    
    # Check if we can query the GitHub API
    if command -v gh &> /dev/null; then
        print_status "Checking GitHub Actions status..."
        gh run list --limit 1 --json status,conclusion,url
    else
        print_status "Install GitHub CLI (gh) to monitor workflow status"
    fi
else
    print_status "Would trigger GitHub Actions workflow for release"
fi

# Generate release notes
print_status "Generating release notes..."

if [ "$DRY_RUN" != true ]; then
    cat > "release-notes-v$NEW_VERSION.md" << EOF
# PA eDocket Desktop v$NEW_VERSION

Released on $(date +"%B %d, %Y")

## What's New

$(awk '/^## \['"$NEW_VERSION"'\]/{flag=1; next} /^## \[/{flag=0} flag' CHANGELOG.md | sed '/^$/d' | head -20)

## Download

Choose the appropriate installer for your platform:

- **macOS**: \`pa-edocket-desktop-macos-x64.dmg\` or \`pa-edocket-desktop-macos-aarch64.dmg\`
- **Windows**: \`pa-edocket-desktop-windows-x64.msi\` or \`pa-edocket-desktop-windows-x64-setup.exe\`
- **Linux**: \`pa-edocket-desktop-linux-amd64.deb\` or \`pa-edocket-desktop-linux-amd64.AppImage\`

## Verification

All releases are signed and can be verified:

\`\`\`bash
# Verify checksums
sha256sum -c filename.sha256

# Verify signatures (platform-specific)
./scripts/verify-signing.sh path/to/downloaded/file
\`\`\`

## Installation

See the [Installation Guide](https://github.com/quartzxmoon/drafter#installation) for detailed instructions.

## Support

- **Documentation**: [User Guide](https://github.com/quartzxmoon/drafter/blob/main/docs/user-guide.md)
- **Issues**: [GitHub Issues](https://github.com/quartzxmoon/drafter/issues)
- **Discussions**: [GitHub Discussions](https://github.com/quartzxmoon/drafter/discussions)

---

**Full Changelog**: [v$CURRENT_VERSION...v$NEW_VERSION](https://github.com/quartzxmoon/drafter/compare/v$CURRENT_VERSION...v$NEW_VERSION)
EOF
    
    print_success "Release notes generated: release-notes-v$NEW_VERSION.md"
else
    print_status "Would generate release notes file"
fi

# Summary
print_status "Deployment Summary:"
echo "===================="
if [ "$DRY_RUN" = true ]; then
    print_warning "DRY RUN MODE - No changes were made"
    echo ""
fi

print_status "Version: $CURRENT_VERSION → $NEW_VERSION"
print_status "Tag: v$NEW_VERSION"
print_status "Branch: $CURRENT_BRANCH"

if [ "$DRY_RUN" != true ]; then
    print_success "✓ Version updated in project files"
    print_success "✓ Changelog updated"
    print_success "✓ Application built successfully"
    print_success "✓ Changes committed and tagged"
    print_success "✓ Tag pushed to GitHub"
    print_success "✓ Release notes generated"
    
    echo ""
    print_status "Next Steps:"
    echo "1. Monitor GitHub Actions workflow"
    echo "2. Review and publish the GitHub release"
    echo "3. Update documentation if needed"
    echo "4. Announce the release"
    
    print_success "Deployment completed successfully!"
else
    echo ""
    print_status "To execute this deployment, run:"
    echo "  $0 $(echo "$@" | sed 's/--dry-run//')"
fi

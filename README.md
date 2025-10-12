# PA eDocket Desktop

A production-grade, cross-platform desktop application for Pennsylvania court docket search, document management, and e-filing workflows. Built with Tauri v2 (Rust backend) and React/TypeScript frontend.

## 🚀 Features

### Core Functionality

- **Live Docket Search**: Real-time search across Pennsylvania court systems
- **Document Drafting**: Automated legal document generation with templates
- **Bluebook Citations**: Compliant legal citation formatting
- **Court-Specific Formatting**: Automatic formatting for all 67 PA counties
- **E-Filing Integration**: Direct filing to supported court systems
- **Export Capabilities**: JSON, CSV, PDF, and ZIP export formats

### Advanced Features

- **Offline Caching**: SQLite-based local storage for dockets and searches
- **Watchlists**: Track cases and receive updates
- **Batch Processing**: Automated background tasks and workflows
- **Security**: OS keychain integration for credential storage
- **Auto-Updates**: Seamless application updates with digital signatures

### Data Sources

- PA UJS Web Portal (public search and docket retrieval)
- PACFile (authenticated e-filing)
- County e-filing systems (Philadelphia, Allegheny)
- C-Track (civil case management)
- CourtListener API (legal research)
- GovInfo API (government documents)

## 🏗️ Architecture

### Technology Stack

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust + Tauri v2 + SQLite + Tokio
- **State Management**: Zustand + React Query
- **Forms**: React Hook Form + Zod validation
- **Routing**: React Router v6

### Key Components

- **Provider Layer**: Abstracted integrations for court systems
- **Service Layer**: Business logic for citations, drafting, exports
- **Domain Models**: Shared types between Rust and TypeScript
- **Configuration System**: YAML-based court and provider configs
- **Security Layer**: Credential management and session handling

## 📋 Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+ with Cargo
- **System Dependencies**:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf`
  - **Windows**: Microsoft C++ Build Tools

## 🛠️ Installation

### 1. Clone and Install Dependencies

```bash
git clone https://github.com/quartzxmoon/drafter.git
cd drafter
npm install
```

### 2. Set Up Environment

```bash
# Copy environment template
cp .env.example .env.local

# Add your API keys
echo "COURTLISTENER_API_TOKEN=your_token_here" >> .env.local
echo "GOVINFO_API_KEY=your_key_here" >> .env.local
```

### 3. Initialize Database

```bash
# Run database migrations
cd src-tauri
cargo run --bin migrate
cd ..
```

## 🚀 Development

### Start Development Server

```bash
npm run tauri dev
```

This starts both the frontend dev server and Tauri backend in development mode.

### Available Scripts

```bash
npm run dev          # Start frontend dev server
npm run build        # Build frontend for production
npm run tauri dev    # Start Tauri development
npm run tauri build  # Build Tauri application
npm test             # Run frontend tests
npm run lint         # Run ESLint
npm run format       # Format code with Prettier
```

### Rust Development

```bash
cd src-tauri

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run Clippy
cargo clippy
```

## 📦 Building and Packaging

### Quick Build

```bash
npm run tauri build
```

### Advanced Packaging

```bash
# Basic release build
./scripts/package.sh

# Debug build
./scripts/package.sh --debug

# Signed build (requires certificates)
./scripts/package.sh --sign

# Signed and notarized (macOS)
./scripts/package.sh --sign --notarize

# Clean build
./scripts/package.sh --clean

# Specific target
./scripts/package.sh --target x86_64-apple-darwin
```

## 🔐 Code Signing Setup

### 1. Generate Signing Keys

```bash
./scripts/setup-signing.sh
```

### 2. Configure Certificates

See `SIGNING_SETUP.md` for detailed instructions on:

- macOS Developer ID certificates
- Windows code signing certificates
- GitHub Actions secrets configuration

### 3. Verify Signatures

```bash
./scripts/verify-signing.sh path/to/signed/app
```

## 🧪 Testing

### Run All Tests

```bash
./scripts/run_tests.sh
```

### Specific Test Types

```bash
# Unit tests only
./scripts/run_tests.sh --unit

# Integration tests only
./scripts/run_tests.sh --integration

# With coverage
./scripts/run_tests.sh --coverage

# Performance benchmarks
./scripts/run_tests.sh --bench
```

### Frontend Tests

```bash
npm test                    # Run once
npm run test:watch         # Watch mode
npm run test:coverage      # With coverage
```

### Rust Tests

```bash
cd src-tauri
cargo test                 # All tests
cargo test --lib           # Unit tests only
cargo test --test integration_tests  # Integration tests
```

## 📊 Performance Monitoring

### Benchmarks

```bash
cd src-tauri
cargo bench
```

View results in `target/criterion/report/index.html`

### Profiling

```bash
# CPU profiling
cargo run --release --bin profile

# Memory profiling
valgrind --tool=massif cargo run --release
```

## 🔧 Configuration

### Court Configuration

Edit `config/courts.yaml` to modify court-specific settings:

```yaml
courts:
  philadelphia:
    name: "Philadelphia County"
    jurisdiction: "municipal"
    formatting:
      date_format: "MM/dd/yyyy"
      case_number_format: "CP-51-CR-{year}-{sequence}"
```

### Provider Configuration

Edit `config/providers.yaml` for API settings:

```yaml
providers:
  ujs_portal:
    base_url: "https://ujsportal.pacourts.us"
    rate_limit:
      requests_per_minute: 30
      burst_size: 5
```

### Automation Jobs

Configure background tasks in `config/jobs.yaml`:

```yaml
jobs:
  docket_refresh:
    schedule: "0 */6 * * *"  # Every 6 hours
    enabled: true
    retry_policy:
      max_attempts: 3
      backoff_multiplier: 2
```

## 🚀 Deployment

### GitHub Actions CI/CD

The project includes comprehensive CI/CD pipelines:

- **Continuous Integration**: Tests, linting, security audits
- **Release Builds**: Multi-platform signed releases
- **Auto-Updates**: Automatic update distribution
- **Documentation**: Auto-generated API docs

### Manual Release

1. **Tag a release**:

   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **GitHub Actions will automatically**:
   - Build for all platforms
   - Sign applications
   - Create release with artifacts
   - Generate update manifests

### Distribution

Built applications are available as:

- **macOS**: `.dmg` (signed and notarized)
- **Windows**: `.msi` and `.exe` (signed)
- **Linux**: `.deb` and `.AppImage`

## 📚 Documentation

### API Documentation

```bash
cd src-tauri
cargo doc --open
```

### Architecture Diagrams

View system architecture in `docs/architecture.md`

### User Guide

Complete user documentation in `docs/user-guide.md`

## 🤝 Contributing

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make changes and test**: `./scripts/run_tests.sh`
4. **Commit changes**: `git commit -m 'Add amazing feature'`
5. **Push to branch**: `git push origin feature/amazing-feature`
6. **Open a Pull Request**

### Code Standards

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **TypeScript**: Use ESLint and Prettier configurations
- **Commits**: Use conventional commit format
- **Tests**: Maintain >90% code coverage

### Issue Reporting

Please use GitHub Issues with:

- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- System information (OS, version)
- Relevant logs or screenshots

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Tauri Team** for the excellent framework
- **Pennsylvania Courts** for providing public access to docket information
- **CourtListener** and **GovInfo** for legal data APIs
- **Rust and React Communities** for amazing tools and libraries

## 📞 Support

- **Documentation**: Check `docs/` directory
- **Issues**: GitHub Issues for bugs and feature requests
- **Discussions**: GitHub Discussions for questions
- **Security**: Email <security@paedocket.com> for security issues

---

**PA eDocket Desktop** - Streamlining Pennsylvania legal workflows with modern technology.

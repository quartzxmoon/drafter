# Quick Start Guide - PA eDocket Desktop

## Initial Setup (5 minutes)

### 1. Install Dependencies

```bash
npm install
cd src-tauri && cargo build && cd ..
```

### 2. Configure Environment

Your API keys are already configured in `.env.local`:
- ✅ CourtListener API Token
- ✅ GovInfo API Key
- ✅ Qdrant API Key (vector search)

### 3. Start Development

```bash
npm run tauri:dev
```

This starts both the Vite dev server and Tauri application.

## GitHub Secrets Setup

Add these secrets to your GitHub repository for CI/CD:

1. Go to: https://github.com/quartzxmoon/drafter/settings/secrets/actions
2. Add the following secrets:

| Secret Name | Value | Purpose |
|-------------|-------|---------|
| `COURTLISTENER_API_TOKEN` | `b3ae1e53785d7eeca5c4d7ceed968fd594bdd8f3` | Court data access |
| `GOVINFO_API_KEY` | `ZidzVKpwkyLQdNP3Ux2IQwz6Y1Qjohrmg12P3fDc` | Gov docs access |
| `QDRANT_API_KEY` | `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.ZZi9F9ygSXvJtz5-w4BShHdDvBj1cZd2c82aKA-eJRg` | Vector search |
| `TAURI_SIGNING_PRIVATE_KEY` | (see below) | App signing |

**Tauri Private Key:**
```
dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5MGVCOXAxZW5LM0Mwd213RHdqUzBlZUxyZURUMFhyWkF1L1B4aXJtUHp4NEFBQkFBQUFBQUFBQUFBQUlBQUFBQW5EUUNUMGlsWEtrVmg4ckhCWTgvNkJTd04zNFBiZ3lvUEhtTHVhaHgzZWpKMmJSc1BsOVcrOHF0UjZPSTJVN0ZwZGNrYkZ5bGtEMjNFcFdUS01nbEJtUUV4ckc3ZVk1SDRreUh3UTU4TFFBMllSNktpWm9BdVhqTGtydmJzeXVPWFVRaDlFdm41Rkk9Cg==
```

## Common Commands

### Development
```bash
npm run tauri:dev        # Start dev server + Tauri
npm run dev              # Frontend only
npm run lint             # Check code quality
npm run format           # Format code
```

### Building
```bash
npm run tauri:build      # Production build
npm run build            # Frontend build only
```

### Testing
```bash
./scripts/run_tests.sh   # All tests
cargo test               # Rust tests only (in src-tauri/)
```

### Database
```bash
npm run db:migrate       # Run migrations
npm run db:seed          # Seed test data
```

## Project Structure

```
drafter/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Route pages
│   ├── lib/citations/      # Citation engine
│   └── types/              # TypeScript types
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── services/       # Business logic
│   │   ├── providers/      # External APIs
│   │   └── domain/         # Data models
│   └── migrations/         # Database migrations
├── config/                 # YAML configurations
└── scripts/                # Build & deploy scripts
```

## Key Files

- `CLAUDE.md` - Architecture guide for AI assistants
- `SECRETS_SETUP.md` - Detailed secrets documentation
- `README.md` - Comprehensive project documentation
- `.env.local` - Your local API keys (git-ignored)
- `src-tauri/tauri.conf.json` - Tauri configuration

## Next Steps

1. ✅ Install dependencies
2. ✅ Configure API keys (already done)
3. 🔲 Add GitHub secrets for CI/CD
4. 🔲 Run `npm run tauri:dev` to test
5. 🔲 Create your first release: `git tag v0.1.0 && git push --tags`

## Troubleshooting

**Dev server won't start:**
```bash
# Clear caches
rm -rf node_modules dist src-tauri/target
npm install
```

**API key errors:**
```bash
# Verify keys are loaded
npm run env:check
```

**Build fails:**
```bash
# Ensure Rust toolchain is installed
rustc --version
cargo --version

# Update Tauri CLI
npm install -g @tauri-apps/cli@latest
```

## Resources

- 📖 [Full Documentation](README.md)
- 🔐 [Secrets Setup Guide](SECRETS_SETUP.md)
- 🤖 [AI Assistant Guide](CLAUDE.md)
- 🌐 [Tauri Docs](https://tauri.app/v2/)
- 🌐 [React Router Docs](https://reactrouter.com/)

## Quick Reference

**Frontend → Backend Communication:**
```typescript
import { invoke } from '@tauri-apps/api/core';

// Call Rust command
const results = await invoke('cmd_search', { params: searchParams });
```

**Adding a new Rust command:**
1. Add handler in `src-tauri/src/services/commands.rs`
2. Register in `src-tauri/src/lib.rs` invoke_handler!
3. Call from frontend using `invoke()`

**Citation processing:**
```typescript
import { CitationEngine } from '@/lib/citations';

const engine = new CitationEngine();
const citations = engine.parse(documentText);
```

---

**Ready to code?** Run `npm run tauri:dev` and start building!

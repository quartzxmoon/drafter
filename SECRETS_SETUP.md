# Secrets and Environment Variables Setup

This guide explains how to configure API keys, signing keys, and environment variables for PA eDocket Desktop.

## Overview

This application requires several API keys and secrets to function:

- **CourtListener API**: For legal research data
- **GovInfo API**: For government documents
- **Tauri Signing Keys**: For application updates and code signing
- **Database Credentials**: For PostgreSQL (production only)
- **Security Keys**: JWT and encryption keys (production only)

## Local Development Setup

### 1. Create Local Environment File

Copy the example environment file:

```bash
cp .env.example .env.local
```

### 2. Add Your API Keys

Edit `.env.local` and add your actual API keys:

```bash
# API Keys for Data Sources
COURTLISTENER_API_TOKEN=b3ae1e53785d7eeca5c4d7ceed968fd594bdd8f3
GOVINFO_API_KEY=ZidzVKpwkyLQdNP3Ux2IQwz6Y1Qjohrmg12P3fDc

# Vector Search
QDRANT_API_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.ZZi9F9ygSXvJtz5-w4BShHdDvBj1cZd2c82aKA-eJRg
```

**Note:** `.env.local` is already in `.gitignore` and will NOT be committed to version control.

### 3. Verify Configuration

Run the environment check script:

```bash
npm run env:check
```

This will verify all required environment variables are set and API keys are valid.

## GitHub Actions Secrets

For CI/CD workflows, add the following secrets to your GitHub repository:

### Navigate to GitHub Secrets

1. Go to: https://github.com/quartzxmoon/drafter/settings/secrets/actions
2. Click **New repository secret** for each secret below

### Required Secrets

#### 1. CourtListener API Token

- **Name:** `COURTLISTENER_API_TOKEN`
- **Value:** `b3ae1e53785d7eeca5c4d7ceed968fd594bdd8f3`
- **Used in:** Data ingestion workflows, search functionality

#### 2. GovInfo API Key

- **Name:** `GOVINFO_API_KEY`
- **Value:** `ZidzVKpwkyLQdNP3Ux2IQwz6Y1Qjohrmg12P3fDc`
- **Used in:** Government document ingestion workflows

#### 3. Qdrant API Key

- **Name:** `QDRANT_API_KEY`
- **Value:** `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.ZZi9F9ygSXvJtz5-w4BShHdDvBj1cZd2c82aKA-eJRg`
- **Used in:** Vector similarity search for legal documents

#### 4. Tauri Signing Private Key

- **Name:** `TAURI_SIGNING_PRIVATE_KEY`
- **Value:**
  ```
  dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5MGVCOXAxZW5LM0Mwd213RHdqUzBlZUxyZURUMFhyWkF1L1B4aXJtUHp4NEFBQkFBQUFBQUFBQUFBQUlBQUFBQW5EUUNUMGlsWEtrVmg4ckhCWTgvNkJTd04zNFBiZ3lvUEhtTHVhaHgzZWpKMmJSc1BsOVcrOHF0UjZPSTJVN0ZwZGNrYkZ5bGtEMjNFcFdUS01nbEJtUUV4ckc3ZVk1SDRreUh3UTU4TFFBMllSNktpWm9BdVhqTGtydmJzeXVPWFVRaDlFdm41Rkk9Cg==
  ```
- **Used in:** Release builds, auto-update signatures
- **Critical:** Keep this secret! If lost, users will need to reinstall the app.

#### 5. Tauri Signing Public Key (Optional)

- **Name:** `TAURI_SIGNING_PUBLIC_KEY`
- **Value:**
  ```
  dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDMxNDdCMTlBNERCN0Q0OUYKUldTZjFMZE5tckZITVJtOFhPc1ZSdmdwVDJVM2xXeTd1YUFZUDVWTkNaVE1RMm1zSWhCOHdhSTAK
  ```
- **Used in:** Already in `tauri.conf.json`, this is optional for CI
- **Note:** This is public and safe to commit to the repository

### Optional macOS Code Signing Secrets

For macOS notarization and code signing (required for distribution):

#### 6. Apple Certificate

- **Name:** `APPLE_CERTIFICATE`
- **Value:** Base64-encoded P12 certificate
- **How to get:**
  ```bash
  # Export certificate from Keychain Access
  # Then encode it
  base64 -i certificate.p12 | pbcopy
  ```

#### 7. Apple Certificate Password

- **Name:** `APPLE_CERTIFICATE_PASSWORD`
- **Value:** Password for your P12 certificate

#### 8. Apple Team ID

- **Name:** `APPLE_TEAM_ID`
- **Value:** Your 10-character Apple Team ID
- **Where to find:** https://developer.apple.com/account

#### 9. Notarization API Key

- **Name:** `NOTARY_API_KEY_ID`
- **Name:** `NOTARY_API_ISSUER_ID`
- **Name:** `NOTARY_API_KEY`
- **Where to get:** https://developer.apple.com/account/resources/authkeys/list

### Optional Windows Code Signing Secrets

For Windows code signing (optional but recommended):

#### 10. Windows Certificate

- **Name:** `WINDOWS_CERTIFICATE`
- **Value:** Base64-encoded PFX certificate
- **Name:** `WINDOWS_CERTIFICATE_PASSWORD`

## Production Environment Variables

For production deployments, you'll also need:

### Database Configuration

```bash
DATABASE_URL=postgresql://user:pass@host:5432/pa_edocket_production
QDRANT_URL=https://your-qdrant-instance.com
QDRANT_API_KEY=your_production_qdrant_key
OPENSEARCH_URL=https://your-opensearch-instance.com
OPENSEARCH_USERNAME=admin
OPENSEARCH_PASSWORD=strong_password_here
```

### Security Configuration

```bash
JWT_SECRET=generate_strong_random_32_char_secret
ENCRYPTION_KEY=generate_strong_random_32_char_key
```

**Generate secure secrets:**

```bash
# On macOS/Linux
openssl rand -base64 32

# On Windows (PowerShell)
[Convert]::ToBase64String([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
```

## API Key Management

### CourtListener API

- **Sign up:** https://www.courtlistener.com/api/
- **Free tier:** 5 requests/second
- **Documentation:** https://www.courtlistener.com/api/rest-info/

### GovInfo API

- **Sign up:** https://api.data.gov/signup/
- **Free tier:** 1,000 requests/hour
- **Documentation:** https://api.govinfo.gov/docs/

### Qdrant Vector Database

- **Sign up:** https://cloud.qdrant.io/
- **Free tier:** 1GB cluster
- **Documentation:** https://qdrant.tech/documentation/

## Security Best Practices

1. **Never commit secrets to version control**
   - `.env.local` is in `.gitignore`
   - Double-check before committing

2. **Use different keys for different environments**
   - Development keys in `.env.local`
   - Production keys in GitHub Secrets or environment variables

3. **Rotate keys periodically**
   - API keys: Every 6-12 months
   - Signing keys: Only if compromised (requires app reinstall)

4. **Backup your signing keys**
   - Store in a password manager (1Password, LastPass, etc.)
   - If lost, users must reinstall the application

5. **Limit key permissions**
   - Use read-only API keys when possible
   - Restrict GitHub Actions to necessary secrets only

## Troubleshooting

### "Invalid API key" errors

1. Verify the key is correct in `.env.local`
2. Check API usage limits haven't been exceeded
3. Ensure no extra whitespace in the key
4. Test the key directly with curl:

```bash
# CourtListener
curl -H "Authorization: Token YOUR_KEY" https://www.courtlistener.com/api/rest/v3/

# GovInfo
curl "https://api.govinfo.gov/collections?api_key=YOUR_KEY"
```

### "Signing key not found" errors

1. Verify `TAURI_SIGNING_PRIVATE_KEY` is set in GitHub Secrets
2. Check the key format (should be base64 encoded)
3. Ensure no extra newlines or spaces
4. Regenerate keys if necessary:
   ```bash
   npm run tauri signer generate --ci
   ```

### Environment variables not loading

1. Restart development server after changing `.env.local`
2. Check file is named exactly `.env.local` (not `.env.local.txt`)
3. Verify file is in project root directory
4. Run `npm run env:check` to diagnose issues

## Reference

### Current Configuration

- **Public key location:** `src-tauri/tauri.conf.json` (line 43)
- **Workflow configuration:** `.github/workflows/release.yml`
- **Environment example:** `.env.example`
- **Local environment:** `.env.local` (git-ignored)

### Useful Commands

```bash
# Check environment configuration
npm run env:check

# Test API connections
npm run tauri dev

# Generate new signing keys
npm run tauri signer generate --ci

# Verify signing key format
echo "YOUR_KEY" | base64 -d
```

## Support

If you encounter issues with secrets or environment configuration:

1. Check this document first
2. Review GitHub Actions logs for specific error messages
3. Verify secrets are correctly set in GitHub repository settings
4. Contact the development team for assistance

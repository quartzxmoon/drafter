# PA eDocket Desktop - Deployment Guide

## Overview

This guide covers the complete deployment of the PA eDocket Desktop application, including the backend API, worker services, and desktop client distribution.

## Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+), macOS (10.15+), Windows (10+)
- **Node.js**: 18.x or later
- **Docker**: 20.10+ with Docker Compose
- **PostgreSQL**: 15+ (if not using Docker)
- **OpenSearch**: 2.11+ (if not using Docker)
- **Qdrant**: 1.7+ (if not using Docker)
- **Redis**: 7+ (if not using Docker)

### Required API Keys

- **CourtListener API Token**: Register at https://www.courtlistener.com/api/
- **GovInfo API Key**: Register at https://api.govinfo.gov/

## Quick Start with Docker

### 1. Environment Setup

```bash
# Clone the repository
git clone https://github.com/quartzxmoon/drafter.git
cd drafter

# Copy environment template
cp .env.example .env

# Edit environment variables
nano .env
```

Required environment variables:
```bash
# Database
DATABASE_URL=postgresql://pa_edocket:secure_password_change_me@localhost:5432/pa_edocket_production

# Search Services
OPENSEARCH_URL=http://localhost:9200
QDRANT_URL=http://localhost:6333

# API Keys
COURTLISTENER_API_TOKEN=your_courtlistener_token_here
GOVINFO_API_KEY=your_govinfo_api_key_here

# Security
JWT_SECRET=your_jwt_secret_32_characters_min
ENCRYPTION_KEY=your_32_character_encryption_key

# Optional
POSTGRES_PASSWORD=secure_password_change_me
GRAFANA_PASSWORD=admin
```

### 2. Deploy with Docker Compose

```bash
# Start all services
docker-compose up -d

# Check service health
docker-compose ps

# View logs
docker-compose logs -f api
docker-compose logs -f worker
```

### 3. Initialize the System

```bash
# Check environment
npm run env:check

# Run database migrations
npm run db:migrate

# Create search indexes
npm run search:create-indexes

# Start initial data ingestion (optional)
npm run ingest:courtlistener:backfill
npm run ingest:govinfo:backfill
```

### 4. Verify Deployment

```bash
# Check API health
curl http://localhost:3000/health

# Check search functionality
curl "http://localhost:3000/api/search?q=contract"

# Check metrics
curl http://localhost:9090/metrics
```

## Manual Installation

### 1. Database Setup

#### PostgreSQL Installation

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql
brew services start postgresql

# Create database and user
sudo -u postgres psql
CREATE DATABASE pa_edocket_production;
CREATE USER pa_edocket WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE pa_edocket_production TO pa_edocket;
\q
```

#### Run Migrations

```bash
# Install dependencies
npm install

# Run migrations
npm run db:migrate
```

### 2. Search Services Setup

#### OpenSearch Installation

```bash
# Download and install OpenSearch
wget https://artifacts.opensearch.org/releases/bundle/opensearch/2.11.0/opensearch-2.11.0-linux-x64.tar.gz
tar -xzf opensearch-2.11.0-linux-x64.tar.gz
cd opensearch-2.11.0

# Configure
echo "discovery.type: single-node" >> config/opensearch.yml
echo "plugins.security.disabled: true" >> config/opensearch.yml

# Start
./bin/opensearch
```

#### Qdrant Installation

```bash
# Download and install Qdrant
wget https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-x86_64-unknown-linux-gnu.tar.gz
tar -xzf qdrant-x86_64-unknown-linux-gnu.tar.gz

# Start
./qdrant
```

#### Redis Installation

```bash
# Ubuntu/Debian
sudo apt install redis-server

# macOS
brew install redis
brew services start redis

# Start Redis
redis-server
```

### 3. Application Services

#### API Server

```bash
# Start API server
npm run api:start

# Or with PM2 for production
npm install -g pm2
pm2 start scripts/api-server.js --name "pa-edocket-api"
```

#### Background Worker

```bash
# Start worker
npm run worker:start

# Or with PM2
pm2 start scripts/worker.js --name "pa-edocket-worker"
```

### 4. Create Search Indexes

```bash
npm run search:create-indexes
```

## Data Ingestion

### Initial Bulk Import

```bash
# CourtListener backfill (5,000+ documents)
npm run ingest:courtlistener:backfill

# GovInfo backfill (1,000+ documents)
npm run ingest:govinfo:backfill
```

### Incremental Sync

```bash
# CourtListener incremental
npm run ingest:courtlistener:since

# GovInfo incremental
npm run ingest:govinfo:since
```

### Monitoring Ingestion

```bash
# Check ingestion status
curl http://localhost:3000/api/stats

# View worker logs
docker-compose logs -f worker
```

## Desktop Client Distribution

### Building Desktop Apps

```bash
# Install Tauri CLI
cargo install tauri-cli

# Build for current platform
cd src-tauri
cargo tauri build

# Build for all platforms (requires setup)
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-apple-darwin
cargo tauri build --target aarch64-apple-darwin
```

### Code Signing

#### macOS

```bash
# Install certificates
security import developer-id.p12 -k ~/Library/Keychains/login.keychain

# Sign and notarize
cargo tauri build --target x86_64-apple-darwin
xcrun notarytool submit target/release/bundle/dmg/PA\ eDocket\ Desktop.dmg \
  --keychain-profile "notarytool-profile" --wait
```

#### Windows

```bash
# Sign with certificate
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com \
  target/release/bundle/msi/PA\ eDocket\ Desktop.msi
```

### Auto-Update Setup

1. Upload signed binaries to release server
2. Generate update manifest
3. Configure auto-updater in Tauri config

## Monitoring and Maintenance

### Health Checks

```bash
# API health
curl http://localhost:3000/health

# Database connection
npm run db:check

# Search services
curl http://localhost:9200/_cluster/health
curl http://localhost:6333/health
```

### Metrics and Monitoring

- **Prometheus**: http://localhost:9091
- **Grafana**: http://localhost:3001 (admin/admin)
- **API Metrics**: http://localhost:9090/metrics

### Log Management

```bash
# View application logs
tail -f logs/api-combined.log
tail -f logs/worker-combined.log

# Docker logs
docker-compose logs -f api
docker-compose logs -f worker
```

### Backup Procedures

#### Database Backup

```bash
# Create backup
pg_dump -h localhost -U pa_edocket pa_edocket_production > backup.sql

# Restore backup
psql -h localhost -U pa_edocket pa_edocket_production < backup.sql
```

#### File Storage Backup

```bash
# Backup uploaded files and data
tar -czf data-backup.tar.gz data/ uploads/
```

## Scaling and Performance

### Horizontal Scaling

1. **API Server**: Deploy multiple instances behind load balancer
2. **Worker**: Increase worker concurrency or deploy multiple workers
3. **Database**: Use read replicas for search queries
4. **Search**: Use OpenSearch cluster with multiple nodes

### Performance Tuning

#### PostgreSQL

```sql
-- Optimize for search workloads
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET random_page_cost = 1.1;
SELECT pg_reload_conf();
```

#### OpenSearch

```yaml
# opensearch.yml
indices.memory.index_buffer_size: 20%
indices.queries.cache.size: 20%
```

### Resource Requirements

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| API Server | 2 cores | 4GB | 10GB |
| Worker | 4 cores | 8GB | 100GB |
| PostgreSQL | 4 cores | 8GB | 500GB |
| OpenSearch | 4 cores | 8GB | 200GB |
| Qdrant | 2 cores | 4GB | 100GB |

## Security Considerations

### Network Security

- Use HTTPS/TLS for all external connections
- Implement firewall rules to restrict access
- Use VPN for administrative access

### Application Security

- Rotate API keys regularly
- Use strong passwords and encryption keys
- Enable audit logging
- Regular security updates

### Data Protection

- Encrypt sensitive data at rest
- Implement proper access controls
- Regular security audits
- Compliance with legal requirements

## Troubleshooting

### Common Issues

#### API Connection Errors

```bash
# Check API server status
curl http://localhost:3000/health

# Check logs
docker-compose logs api
```

#### Database Connection Issues

```bash
# Test database connection
psql -h localhost -U pa_edocket pa_edocket_production -c "SELECT 1;"

# Check PostgreSQL logs
docker-compose logs postgres
```

#### Search Service Issues

```bash
# Check OpenSearch
curl http://localhost:9200/_cluster/health

# Check Qdrant
curl http://localhost:6333/health
```

#### Ingestion Problems

```bash
# Check worker status
docker-compose logs worker

# Verify API keys
npm run env:check

# Check rate limiting
curl -I https://www.courtlistener.com/api/rest/v3/
```

### Performance Issues

1. **Slow Search**: Check OpenSearch cluster health and index optimization
2. **High Memory Usage**: Monitor and adjust JVM heap sizes
3. **Database Locks**: Check for long-running queries and optimize indexes
4. **Worker Backlog**: Increase worker concurrency or add more workers

## Support and Maintenance

### Regular Maintenance Tasks

- Weekly database maintenance and optimization
- Monthly security updates
- Quarterly performance reviews
- Annual security audits

### Monitoring Alerts

Set up alerts for:
- API response time > 5 seconds
- Database connection failures
- Search service downtime
- Worker queue backlog > 1000 jobs
- Disk usage > 80%

### Contact Information

For deployment support and issues:
- Technical Support: support@pa-edocket.com
- Documentation: https://docs.pa-edocket.com
- GitHub Issues: https://github.com/quartzxmoon/drafter/issues

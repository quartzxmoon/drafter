#!/usr/bin/env node

/**
 * Database Migration Script
 * Applies SQL migrations to the PostgreSQL database
 */

import { config } from 'dotenv';
import { Client } from 'pg';
import { readFileSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load environment variables
config();

class DatabaseMigrator {
  constructor() {
    if (!process.env.DATABASE_URL) {
      throw new Error('DATABASE_URL environment variable is required');
    }
    
    this.client = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    this.migrationsDir = join(__dirname, '..', 'migrations');
  }

  async connect() {
    await this.client.connect();
    console.log('✅ Connected to PostgreSQL database');
  }

  async disconnect() {
    await this.client.end();
    console.log('✅ Disconnected from database');
  }

  async createMigrationsTable() {
    const query = `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        id SERIAL PRIMARY KEY,
        filename TEXT NOT NULL UNIQUE,
        applied_at TIMESTAMPTZ DEFAULT NOW(),
        checksum TEXT NOT NULL
      );
    `;
    
    await this.client.query(query);
    console.log('✅ Migrations table ready');
  }

  async getAppliedMigrations() {
    const result = await this.client.query(
      'SELECT filename, checksum FROM schema_migrations ORDER BY filename'
    );
    return new Map(result.rows.map(row => [row.filename, row.checksum]));
  }

  async getMigrationFiles() {
    try {
      const files = readdirSync(this.migrationsDir)
        .filter(file => file.endsWith('.sql'))
        .sort();
      
      return files.map(filename => {
        const filepath = join(this.migrationsDir, filename);
        const content = readFileSync(filepath, 'utf8');
        const checksum = this.calculateChecksum(content);
        
        return { filename, filepath, content, checksum };
      });
    } catch (error) {
      if (error.code === 'ENOENT') {
        console.log('📁 No migrations directory found, creating...');
        return [];
      }
      throw error;
    }
  }

  calculateChecksum(content) {
    const crypto = await import('crypto');
    return crypto.createHash('sha256').update(content).digest('hex');
  }

  async applyMigration(migration) {
    console.log(`🔄 Applying migration: ${migration.filename}`);
    
    try {
      await this.client.query('BEGIN');
      
      // Execute the migration
      await this.client.query(migration.content);
      
      // Record the migration
      await this.client.query(
        'INSERT INTO schema_migrations (filename, checksum) VALUES ($1, $2)',
        [migration.filename, migration.checksum]
      );
      
      await this.client.query('COMMIT');
      console.log(`✅ Applied migration: ${migration.filename}`);
      
    } catch (error) {
      await this.client.query('ROLLBACK');
      console.error(`❌ Failed to apply migration ${migration.filename}:`, error.message);
      throw error;
    }
  }

  async validateMigration(migration, appliedChecksum) {
    if (migration.checksum !== appliedChecksum) {
      throw new Error(
        `Migration ${migration.filename} has been modified after being applied. ` +
        `Expected checksum: ${appliedChecksum}, actual: ${migration.checksum}`
      );
    }
  }

  async run() {
    console.log('🚀 Starting database migration...\n');
    
    try {
      await this.connect();
      await this.createMigrationsTable();
      
      const appliedMigrations = await this.getAppliedMigrations();
      const migrationFiles = await this.getMigrationFiles();
      
      console.log(`📋 Found ${migrationFiles.length} migration files`);
      console.log(`📋 ${appliedMigrations.size} migrations already applied\n`);
      
      let appliedCount = 0;
      
      for (const migration of migrationFiles) {
        if (appliedMigrations.has(migration.filename)) {
          // Validate that the migration hasn't been modified
          const appliedChecksum = appliedMigrations.get(migration.filename);
          await this.validateMigration(migration, appliedChecksum);
          console.log(`⏭️  Skipping already applied: ${migration.filename}`);
        } else {
          await this.applyMigration(migration);
          appliedCount++;
        }
      }
      
      console.log(`\n🎉 Migration completed successfully!`);
      console.log(`📊 Applied ${appliedCount} new migrations`);
      
      // Show database statistics
      await this.showDatabaseStats();
      
    } catch (error) {
      console.error('\n❌ Migration failed:', error.message);
      process.exit(1);
    } finally {
      await this.disconnect();
    }
  }

  async showDatabaseStats() {
    try {
      console.log('\n📊 Database Statistics:');
      
      // Table counts
      const tableStats = await this.client.query(`
        SELECT 
          schemaname,
          tablename,
          n_tup_ins as inserts,
          n_tup_upd as updates,
          n_tup_del as deletes,
          n_live_tup as live_rows
        FROM pg_stat_user_tables 
        ORDER BY tablename
      `);
      
      if (tableStats.rows.length > 0) {
        console.log('\nTable Statistics:');
        tableStats.rows.forEach(row => {
          console.log(`  📋 ${row.tablename}: ${row.live_rows} rows`);
        });
      }
      
      // Index usage
      const indexStats = await this.client.query(`
        SELECT 
          schemaname,
          tablename,
          indexname,
          idx_scan as scans
        FROM pg_stat_user_indexes 
        WHERE idx_scan > 0
        ORDER BY idx_scan DESC
        LIMIT 10
      `);
      
      if (indexStats.rows.length > 0) {
        console.log('\nTop Index Usage:');
        indexStats.rows.forEach(row => {
          console.log(`  🔍 ${row.indexname}: ${row.scans} scans`);
        });
      }
      
      // Database size
      const sizeResult = await this.client.query(`
        SELECT pg_size_pretty(pg_database_size(current_database())) as size
      `);
      
      console.log(`\n💾 Database size: ${sizeResult.rows[0].size}`);
      
    } catch (error) {
      console.warn('⚠️  Could not retrieve database statistics:', error.message);
    }
  }
}

// Command line interface
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
Database Migration Tool

Usage: node migrate.js [options]

Options:
  --help, -h     Show this help message
  --dry-run      Show what would be migrated without applying
  --force        Force migration even if checksums don't match

Environment Variables:
  DATABASE_URL   PostgreSQL connection string (required)

Examples:
  node migrate.js                    # Apply all pending migrations
  node migrate.js --dry-run          # Preview migrations
  node migrate.js --force            # Force apply (dangerous)
`);
    process.exit(0);
  }
  
  if (args.includes('--dry-run')) {
    console.log('🔍 DRY RUN MODE - No changes will be made\n');
    // TODO: Implement dry-run logic
    console.log('Dry-run mode not yet implemented');
    process.exit(0);
  }
  
  const migrator = new DatabaseMigrator();
  await migrator.run();
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(error => {
    console.error('Migration failed:', error);
    process.exit(1);
  });
}

export { DatabaseMigrator };

#!/usr/bin/env node

/**
 * Environment Check Script
 * Verifies all required environment variables and connectivity to external services
 */

import { config } from 'dotenv';
import { Client } from 'pg';
import fetch from 'node-fetch';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

// Load environment variables
config();

const REQUIRED_VARS = [
  'DATABASE_URL',
  'QDRANT_URL', 
  'OPENSEARCH_URL',
  'COURTLISTENER_API_TOKEN',
  'GOVINFO_API_KEY'
];

const OPTIONAL_VARS = [
  'QDRANT_API_KEY',
  'OPENSEARCH_USERNAME',
  'OPENSEARCH_PASSWORD',
  'JWT_SECRET',
  'ENCRYPTION_KEY'
];

class EnvironmentChecker {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.success = [];
  }

  log(level, message) {
    const timestamp = new Date().toISOString();
    const prefix = `[${timestamp}] [${level.toUpperCase()}]`;
    
    switch (level) {
      case 'error':
        console.error(`\x1b[31m${prefix} ${message}\x1b[0m`);
        this.errors.push(message);
        break;
      case 'warn':
        console.warn(`\x1b[33m${prefix} ${message}\x1b[0m`);
        this.warnings.push(message);
        break;
      case 'success':
        console.log(`\x1b[32m${prefix} ${message}\x1b[0m`);
        this.success.push(message);
        break;
      default:
        console.log(`${prefix} ${message}`);
    }
  }

  checkEnvironmentVariables() {
    this.log('info', 'Checking required environment variables...');
    
    for (const varName of REQUIRED_VARS) {
      if (!process.env[varName]) {
        this.log('error', `Missing required environment variable: ${varName}`);
      } else {
        this.log('success', `Found required variable: ${varName}`);
      }
    }

    for (const varName of OPTIONAL_VARS) {
      if (!process.env[varName]) {
        this.log('warn', `Optional environment variable not set: ${varName}`);
      } else {
        this.log('success', `Found optional variable: ${varName}`);
      }
    }
  }

  async checkPostgresConnection() {
    this.log('info', 'Testing PostgreSQL connection...');
    
    if (!process.env.DATABASE_URL) {
      this.log('error', 'DATABASE_URL not set, skipping PostgreSQL check');
      return;
    }

    const client = new Client({
      connectionString: process.env.DATABASE_URL,
      connectionTimeoutMillis: 5000,
    });

    try {
      await client.connect();
      const result = await client.query('SELECT version()');
      this.log('success', `PostgreSQL connected: ${result.rows[0].version.split(' ')[0]} ${result.rows[0].version.split(' ')[1]}`);
      
      // Check if our tables exist
      const tablesResult = await client.query(`
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name IN ('sources', 'documents', 'ingest_state')
      `);
      
      if (tablesResult.rows.length > 0) {
        this.log('success', `Found ${tablesResult.rows.length} application tables`);
      } else {
        this.log('warn', 'No application tables found - run migrations');
      }
      
    } catch (error) {
      this.log('error', `PostgreSQL connection failed: ${error.message}`);
    } finally {
      await client.end();
    }
  }

  async checkQdrantConnection() {
    this.log('info', 'Testing Qdrant connection...');
    
    if (!process.env.QDRANT_URL) {
      this.log('error', 'QDRANT_URL not set, skipping Qdrant check');
      return;
    }

    try {
      const headers = {};
      if (process.env.QDRANT_API_KEY) {
        headers['api-key'] = process.env.QDRANT_API_KEY;
      }

      const response = await fetch(`${process.env.QDRANT_URL}/collections`, {
        method: 'GET',
        headers,
        timeout: 5000
      });

      if (response.ok) {
        const data = await response.json();
        this.log('success', `Qdrant connected: ${data.result?.collections?.length || 0} collections`);
      } else {
        this.log('error', `Qdrant connection failed: HTTP ${response.status}`);
      }
    } catch (error) {
      this.log('error', `Qdrant connection failed: ${error.message}`);
    }
  }

  async checkOpenSearchConnection() {
    this.log('info', 'Testing OpenSearch connection...');
    
    if (!process.env.OPENSEARCH_URL) {
      this.log('error', 'OPENSEARCH_URL not set, skipping OpenSearch check');
      return;
    }

    try {
      const auth = process.env.OPENSEARCH_USERNAME && process.env.OPENSEARCH_PASSWORD
        ? Buffer.from(`${process.env.OPENSEARCH_USERNAME}:${process.env.OPENSEARCH_PASSWORD}`).toString('base64')
        : null;

      const headers = {
        'Content-Type': 'application/json'
      };
      
      if (auth) {
        headers['Authorization'] = `Basic ${auth}`;
      }

      const response = await fetch(`${process.env.OPENSEARCH_URL}/_cluster/health`, {
        method: 'GET',
        headers,
        timeout: 5000,
        // Allow self-signed certificates in development
        agent: process.env.NODE_ENV === 'development' ? 
          new (require('https').Agent)({ rejectUnauthorized: false }) : undefined
      });

      if (response.ok) {
        const data = await response.json();
        this.log('success', `OpenSearch connected: ${data.status} cluster with ${data.number_of_nodes} nodes`);
      } else {
        this.log('error', `OpenSearch connection failed: HTTP ${response.status}`);
      }
    } catch (error) {
      this.log('error', `OpenSearch connection failed: ${error.message}`);
    }
  }

  async checkAPIConnectivity() {
    this.log('info', 'Testing external API connectivity...');

    // Test CourtListener API
    if (process.env.COURTLISTENER_API_TOKEN) {
      try {
        const response = await fetch('https://www.courtlistener.com/api/rest/v3/jurisdictions/?format=json', {
          headers: {
            'Authorization': `Token ${process.env.COURTLISTENER_API_TOKEN}`,
            'User-Agent': 'PA-eDocket-Desktop/1.0'
          },
          timeout: 10000
        });

        if (response.ok) {
          const data = await response.json();
          this.log('success', `CourtListener API connected: ${data.count} jurisdictions available`);
        } else {
          this.log('error', `CourtListener API failed: HTTP ${response.status}`);
        }
      } catch (error) {
        this.log('error', `CourtListener API failed: ${error.message}`);
      }
    }

    // Test GovInfo API
    if (process.env.GOVINFO_API_KEY) {
      try {
        const response = await fetch(`https://api.govinfo.gov/collections?api_key=${process.env.GOVINFO_API_KEY}`, {
          timeout: 10000
        });

        if (response.ok) {
          const data = await response.json();
          this.log('success', `GovInfo API connected: ${data.collections?.length || 0} collections available`);
        } else {
          this.log('error', `GovInfo API failed: HTTP ${response.status}`);
        }
      } catch (error) {
        this.log('error', `GovInfo API failed: ${error.message}`);
      }
    }
  }

  async run() {
    console.log('\n🔍 PA eDocket Desktop - Environment Check\n');
    
    this.checkEnvironmentVariables();
    await this.checkPostgresConnection();
    await this.checkQdrantConnection();
    await this.checkOpenSearchConnection();
    await this.checkAPIConnectivity();

    console.log('\n📊 Summary:');
    console.log(`✅ Success: ${this.success.length}`);
    console.log(`⚠️  Warnings: ${this.warnings.length}`);
    console.log(`❌ Errors: ${this.errors.length}`);

    if (this.errors.length > 0) {
      console.log('\n❌ Critical Issues:');
      this.errors.forEach(error => console.log(`  • ${error}`));
      process.exit(1);
    }

    if (this.warnings.length > 0) {
      console.log('\n⚠️  Warnings:');
      this.warnings.forEach(warning => console.log(`  • ${warning}`));
    }

    console.log('\n✅ Environment check completed successfully!\n');
    process.exit(0);
  }
}

// Run the checker
const checker = new EnvironmentChecker();
checker.run().catch(error => {
  console.error('Environment check failed:', error);
  process.exit(1);
});

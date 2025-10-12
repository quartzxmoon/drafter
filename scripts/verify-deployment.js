#!/usr/bin/env node

/**
 * PA eDocket Desktop Deployment Verification
 * Comprehensive verification suite for production deployment
 */

import { config } from 'dotenv';
import { Client } from 'pg';
import fetch from 'node-fetch';
import https from 'https';
import { promises as fs } from 'fs';

// Load environment variables
config();

class DeploymentVerifier {
  constructor() {
    this.results = {
      environment: { passed: 0, failed: 0, tests: [] },
      database: { passed: 0, failed: 0, tests: [] },
      api: { passed: 0, failed: 0, tests: [] },
      search: { passed: 0, failed: 0, tests: [] },
      ingestion: { passed: 0, failed: 0, tests: [] },
      integration: { passed: 0, failed: 0, tests: [] }
    };
    
    this.httpsAgent = new https.Agent({ rejectUnauthorized: false });
  }

  async runAllTests() {
    console.log('🚀 Starting PA eDocket Desktop Deployment Verification\n');
    
    try {
      await this.testEnvironment();
      await this.testDatabase();
      await this.testAPI();
      await this.testSearchServices();
      await this.testIngestion();
      await this.testIntegration();
      
      this.printSummary();
      
    } catch (error) {
      console.error('❌ Verification failed:', error.message);
      process.exit(1);
    }
  }

  async testEnvironment() {
    console.log('📋 Testing Environment Configuration...');
    
    const requiredVars = [
      'DATABASE_URL',
      'OPENSEARCH_URL',
      'QDRANT_URL',
      'COURTLISTENER_API_TOKEN',
      'GOVINFO_API_KEY'
    ];
    
    for (const varName of requiredVars) {
      const result = await this.test(
        `Environment variable ${varName}`,
        () => {
          const value = process.env[varName];
          if (!value) throw new Error(`${varName} is not set`);
          if (value.includes('change_me') || value.includes('your_')) {
            throw new Error(`${varName} contains placeholder value`);
          }
          return true;
        },
        'environment'
      );
    }
    
    // Test file permissions
    await this.test(
      'Data directory writable',
      async () => {
        await fs.access('./data', fs.constants.W_OK);
        return true;
      },
      'environment'
    );
    
    await this.test(
      'Logs directory writable',
      async () => {
        await fs.access('./logs', fs.constants.W_OK);
        return true;
      },
      'environment'
    );
  }

  async testDatabase() {
    console.log('🗄️  Testing Database Connection...');
    
    const client = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    await this.test(
      'PostgreSQL connection',
      async () => {
        await client.connect();
        const result = await client.query('SELECT version()');
        return result.rows.length > 0;
      },
      'database'
    );
    
    await this.test(
      'Required tables exist',
      async () => {
        const tables = ['sources', 'documents', 'ingest_state', 'watchlists', 'export_history'];
        for (const table of tables) {
          const result = await client.query(
            `SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)`,
            [table]
          );
          if (!result.rows[0].exists) {
            throw new Error(`Table ${table} does not exist`);
          }
        }
        return true;
      },
      'database'
    );
    
    await this.test(
      'Database indexes exist',
      async () => {
        const result = await client.query(`
          SELECT COUNT(*) as index_count 
          FROM pg_indexes 
          WHERE tablename IN ('documents', 'sources', 'ingest_state')
        `);
        const indexCount = parseInt(result.rows[0].index_count);
        if (indexCount < 5) {
          throw new Error(`Expected at least 5 indexes, found ${indexCount}`);
        }
        return true;
      },
      'database'
    );
    
    await client.end();
  }

  async testAPI() {
    console.log('🌐 Testing API Server...');
    
    const apiBase = process.env.VITE_API_BASE || 'http://localhost:3000';
    
    await this.test(
      'API health check',
      async () => {
        const response = await fetch(`${apiBase}/health`, { agent: this.httpsAgent });
        if (!response.ok) throw new Error(`Health check failed: ${response.status}`);
        const data = await response.json();
        return data.status === 'healthy';
      },
      'api'
    );
    
    await this.test(
      'API metrics endpoint',
      async () => {
        const response = await fetch(`${apiBase}/metrics`, { agent: this.httpsAgent });
        if (!response.ok) throw new Error(`Metrics failed: ${response.status}`);
        const text = await response.text();
        return text.includes('http_requests_total');
      },
      'api'
    );
    
    await this.test(
      'API search endpoint',
      async () => {
        const response = await fetch(`${apiBase}/api/search?q=test`, { agent: this.httpsAgent });
        if (!response.ok) throw new Error(`Search failed: ${response.status}`);
        const data = await response.json();
        return data.hasOwnProperty('results') && data.hasOwnProperty('pagination');
      },
      'api'
    );
    
    await this.test(
      'API stats endpoint',
      async () => {
        const response = await fetch(`${apiBase}/api/stats`, { agent: this.httpsAgent });
        if (!response.ok) throw new Error(`Stats failed: ${response.status}`);
        const data = await response.json();
        return data.hasOwnProperty('overview');
      },
      'api'
    );
  }

  async testSearchServices() {
    console.log('🔍 Testing Search Services...');
    
    // Test OpenSearch
    await this.test(
      'OpenSearch cluster health',
      async () => {
        const response = await fetch(`${process.env.OPENSEARCH_URL}/_cluster/health`, {
          agent: this.httpsAgent
        });
        if (!response.ok) throw new Error(`OpenSearch health failed: ${response.status}`);
        const data = await response.json();
        return data.status === 'green' || data.status === 'yellow';
      },
      'search'
    );
    
    await this.test(
      'OpenSearch documents index exists',
      async () => {
        const response = await fetch(`${process.env.OPENSEARCH_URL}/documents_v1`, {
          agent: this.httpsAgent
        });
        return response.ok;
      },
      'search'
    );
    
    // Test Qdrant
    await this.test(
      'Qdrant health check',
      async () => {
        const response = await fetch(`${process.env.QDRANT_URL}/health`, {
          agent: this.httpsAgent
        });
        if (!response.ok) throw new Error(`Qdrant health failed: ${response.status}`);
        return true;
      },
      'search'
    );
    
    await this.test(
      'Qdrant collections exist',
      async () => {
        const response = await fetch(`${process.env.QDRANT_URL}/collections`, {
          agent: this.httpsAgent
        });
        if (!response.ok) throw new Error(`Qdrant collections failed: ${response.status}`);
        const data = await response.json();
        const collections = data.result.collections.map(c => c.name);
        return collections.includes('docs_v1');
      },
      'search'
    );
  }

  async testIngestion() {
    console.log('📥 Testing Data Ingestion...');
    
    // Test CourtListener API
    await this.test(
      'CourtListener API access',
      async () => {
        const response = await fetch('https://www.courtlistener.com/api/rest/v3/jurisdictions/', {
          headers: {
            'Authorization': `Token ${process.env.COURTLISTENER_API_TOKEN}`,
            'User-Agent': 'PA-eDocket-Desktop/1.0'
          }
        });
        if (!response.ok) throw new Error(`CourtListener API failed: ${response.status}`);
        const data = await response.json();
        return data.count > 0;
      },
      'ingestion'
    );
    
    // Test GovInfo API
    await this.test(
      'GovInfo API access',
      async () => {
        const response = await fetch(`https://api.govinfo.gov/collections?api_key=${process.env.GOVINFO_API_KEY}`);
        if (!response.ok) throw new Error(`GovInfo API failed: ${response.status}`);
        const data = await response.json();
        return data.collections && data.collections.length > 0;
      },
      'ingestion'
    );
    
    // Test document count
    const client = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    await client.connect();
    
    await this.test(
      'Documents ingested (minimum 1000)',
      async () => {
        const result = await client.query('SELECT COUNT(*) as count FROM documents');
        const count = parseInt(result.rows[0].count);
        console.log(`    Found ${count} documents`);
        return count >= 1000;
      },
      'ingestion'
    );
    
    await this.test(
      'Multiple sources active',
      async () => {
        const result = await client.query(`
          SELECT s.name, COUNT(d.id) as doc_count 
          FROM sources s 
          LEFT JOIN documents d ON s.id = d.source_id 
          GROUP BY s.id, s.name
        `);
        console.log('    Source breakdown:');
        result.rows.forEach(row => {
          console.log(`      ${row.name}: ${row.doc_count} documents`);
        });
        return result.rows.length >= 2;
      },
      'ingestion'
    );
    
    await client.end();
  }

  async testIntegration() {
    console.log('🔗 Testing End-to-End Integration...');
    
    const apiBase = process.env.VITE_API_BASE || 'http://localhost:3000';
    
    await this.test(
      'Search integration test',
      async () => {
        const response = await fetch(`${apiBase}/api/search?q=contract&pageSize=5`, {
          agent: this.httpsAgent
        });
        if (!response.ok) throw new Error(`Search integration failed: ${response.status}`);
        const data = await response.json();
        console.log(`    Found ${data.results.length} results for "contract"`);
        return data.results.length > 0;
      },
      'integration'
    );
    
    await this.test(
      'Document retrieval test',
      async () => {
        // First get a document ID from search
        const searchResponse = await fetch(`${apiBase}/api/search?q=court&pageSize=1`, {
          agent: this.httpsAgent
        });
        const searchData = await searchResponse.json();
        
        if (searchData.results.length === 0) {
          throw new Error('No documents found for retrieval test');
        }
        
        const docId = searchData.results[0].id;
        const docResponse = await fetch(`${apiBase}/api/documents/${docId}`, {
          agent: this.httpsAgent
        });
        
        if (!docResponse.ok) throw new Error(`Document retrieval failed: ${docResponse.status}`);
        const docData = await docResponse.json();
        return docData.id === docId;
      },
      'integration'
    );
    
    await this.test(
      'Export functionality test',
      async () => {
        const exportRequest = {
          format: 'json',
          query: 'test',
          filters: {}
        };
        
        const response = await fetch(`${apiBase}/api/export`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(exportRequest),
          agent: this.httpsAgent
        });
        
        if (!response.ok) throw new Error(`Export failed: ${response.status}`);
        const data = await response.json();
        return data.hasOwnProperty('exportId');
      },
      'integration'
    );
  }

  async test(name, testFn, category) {
    try {
      const result = await testFn();
      if (result) {
        console.log(`  ✅ ${name}`);
        this.results[category].passed++;
        this.results[category].tests.push({ name, status: 'PASS' });
        return true;
      } else {
        console.log(`  ❌ ${name}: Test returned false`);
        this.results[category].failed++;
        this.results[category].tests.push({ name, status: 'FAIL', error: 'Test returned false' });
        return false;
      }
    } catch (error) {
      console.log(`  ❌ ${name}: ${error.message}`);
      this.results[category].failed++;
      this.results[category].tests.push({ name, status: 'FAIL', error: error.message });
      return false;
    }
  }

  printSummary() {
    console.log('\n📊 Verification Summary');
    console.log('========================');
    
    let totalPassed = 0;
    let totalFailed = 0;
    
    for (const [category, results] of Object.entries(this.results)) {
      const total = results.passed + results.failed;
      const percentage = total > 0 ? Math.round((results.passed / total) * 100) : 0;
      
      console.log(`${category.toUpperCase()}: ${results.passed}/${total} (${percentage}%)`);
      
      totalPassed += results.passed;
      totalFailed += results.failed;
    }
    
    const grandTotal = totalPassed + totalFailed;
    const overallPercentage = grandTotal > 0 ? Math.round((totalPassed / grandTotal) * 100) : 0;
    
    console.log('------------------------');
    console.log(`OVERALL: ${totalPassed}/${grandTotal} (${overallPercentage}%)`);
    
    if (totalFailed === 0) {
      console.log('\n🎉 All tests passed! Deployment is ready for production.');
      process.exit(0);
    } else {
      console.log(`\n⚠️  ${totalFailed} test(s) failed. Please review and fix issues before production deployment.`);
      process.exit(1);
    }
  }
}

// Run verification if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const verifier = new DeploymentVerifier();
  verifier.runAllTests().catch(error => {
    console.error('Verification failed:', error);
    process.exit(1);
  });
}

export { DeploymentVerifier };

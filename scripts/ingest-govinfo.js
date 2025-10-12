#!/usr/bin/env node

/**
 * GovInfo Bulk Ingest Script
 * Fetches government documents from GovInfo API and processes them for search
 */

import { config } from 'dotenv';
import { Client } from 'pg';
import fetch from 'node-fetch';
import { createWriteStream, mkdirSync, existsSync } from 'fs';
import { join } from 'path';
import { pipeline } from 'stream/promises';
import crypto from 'crypto';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Load environment variables
config();

class GovInfoIngestor {
  constructor() {
    this.apiKey = process.env.GOVINFO_API_KEY;
    this.baseUrl = 'https://api.govinfo.gov';
    this.dataDir = process.env.DATA_DIR || './data';
    this.rateLimit = parseInt(process.env.GOVINFO_RATE_LIMIT) || 10;
    this.batchSize = 100;
    
    if (!this.apiKey) {
      throw new Error('GOVINFO_API_KEY environment variable is required');
    }
    
    this.client = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    // Rate limiting with dynamic adjustment
    this.lastRequest = 0;
    this.requestInterval = 1000 / this.rateLimit;
    this.rateLimitRemaining = this.rateLimit;
    
    // Statistics
    this.stats = {
      processed: 0,
      failed: 0,
      downloaded: 0,
      bytes: 0,
      startTime: Date.now()
    };
    
    // Collections to ingest
    this.collections = [
      'USCOURTS', // US Court opinions and orders
      'CFR',      // Code of Federal Regulations
      'FR',       // Federal Register
      'STATUTE',  // US Statutes at Large
      'BILLS',    // Congressional Bills
      'CRPT'      // Congressional Reports
    ];
    
    // Ensure data directory exists
    mkdirSync(this.dataDir, { recursive: true });
    mkdirSync(join(this.dataDir, 'govinfo', 'pdfs'), { recursive: true });
    mkdirSync(join(this.dataDir, 'govinfo', 'text'), { recursive: true });
  }

  async connect() {
    await this.client.connect();
    console.log('✅ Connected to PostgreSQL database');
  }

  async disconnect() {
    await this.client.end();
    console.log('✅ Disconnected from database');
  }

  async rateLimit() {
    const now = Date.now();
    const timeSinceLastRequest = now - this.lastRequest;
    
    if (timeSinceLastRequest < this.requestInterval) {
      const delay = this.requestInterval - timeSinceLastRequest;
      await new Promise(resolve => setTimeout(resolve, delay));
    }
    
    this.lastRequest = Date.now();
  }

  async makeApiRequest(endpoint, params = {}) {
    await this.rateLimit();
    
    const url = new URL(`${this.baseUrl}${endpoint}`);
    url.searchParams.append('api_key', this.apiKey);
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        url.searchParams.append(key, value.toString());
      }
    });

    const response = await fetch(url.toString(), {
      headers: {
        'User-Agent': 'PA-eDocket-Desktop/1.0'
      }
    });

    // Handle rate limiting
    const rateLimitRemaining = response.headers.get('X-RateLimit-Remaining');
    if (rateLimitRemaining) {
      this.rateLimitRemaining = parseInt(rateLimitRemaining);
      
      // Adjust rate if we're hitting limits
      if (this.rateLimitRemaining < 5) {
        this.requestInterval = Math.min(this.requestInterval * 1.5, 2000);
        console.log(`⚠️  Rate limit low (${this.rateLimitRemaining}), slowing down`);
      }
    }

    if (!response.ok) {
      if (response.status === 429) {
        // Rate limited, wait and retry
        console.log('⏳ Rate limited, waiting 60 seconds...');
        await new Promise(resolve => setTimeout(resolve, 60000));
        return this.makeApiRequest(endpoint, params);
      }
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  async downloadFile(url, filepath) {
    await this.rateLimit();
    
    const response = await fetch(url, {
      headers: {
        'User-Agent': 'PA-eDocket-Desktop/1.0'
      }
    });

    if (!response.ok) {
      throw new Error(`Download failed: ${response.status} ${response.statusText}`);
    }

    await pipeline(response.body, createWriteStream(filepath));
    
    // Calculate file hash
    const fileBuffer = await import('fs').then(fs => fs.promises.readFile(filepath));
    const hash = crypto.createHash('sha256').update(fileBuffer).digest('hex');
    
    this.stats.downloaded++;
    this.stats.bytes += fileBuffer.length;
    
    return { hash, size: fileBuffer.length };
  }

  async extractText(pdfPath) {
    const txtPath = pdfPath.replace('/pdfs/', '/text/').replace('.pdf', '.txt');
    
    try {
      // Try pdftotext first
      await execAsync(`pdftotext "${pdfPath}" "${txtPath}"`);
      
      // Check if text was extracted
      const text = await import('fs').then(fs => fs.promises.readFile(txtPath, 'utf8'));
      
      if (text.trim().length < 100) {
        // Text is too short, might be scanned - try OCR
        console.log(`⚠️  Short text extracted, attempting OCR for ${pdfPath}`);
        await execAsync(`ocrmypdf --skip-text "${pdfPath}" "${pdfPath}.ocr.pdf" && pdftotext "${pdfPath}.ocr.pdf" "${txtPath}"`);
      }
      
      return txtPath;
      
    } catch (error) {
      console.warn(`⚠️  Text extraction failed for ${pdfPath}:`, error.message);
      return null;
    }
  }

  async processPackage(packageData, collection) {
    try {
      const sourceResult = await this.client.query(
        'SELECT id FROM sources WHERE name = $1',
        ['govinfo']
      );
      const sourceId = sourceResult.rows[0].id;

      // Get package details
      const packageDetails = await this.makeApiRequest(`/packages/${packageData.packageId}`);
      
      // Download PDF if available
      let pdfPath = null;
      let txtPath = null;
      let fileHash = null;
      let fileSize = null;

      if (packageDetails.download?.pdfLink) {
        const filename = `${packageData.packageId}.pdf`;
        pdfPath = join(this.dataDir, 'govinfo', 'pdfs', filename);
        
        if (!existsSync(pdfPath)) {
          const downloadResult = await this.downloadFile(packageDetails.download.pdfLink, pdfPath);
          fileHash = downloadResult.hash;
          fileSize = downloadResult.size;
          
          // Extract text
          txtPath = await this.extractText(pdfPath);
        } else {
          // File already exists, calculate hash
          const fileBuffer = await import('fs').then(fs => fs.promises.readFile(pdfPath));
          fileHash = crypto.createHash('sha256').update(fileBuffer).digest('hex');
          fileSize = fileBuffer.length;
        }
      }

      // Read extracted text
      let fullText = '';
      if (txtPath && existsSync(txtPath)) {
        fullText = await import('fs').then(fs => fs.promises.readFile(txtPath, 'utf8'));
      }

      // Normalize document type
      const docType = this.normalizeDocumentType(collection, packageDetails);

      // Extract court information for USCOURTS
      const courtInfo = this.extractCourtInfo(packageDetails, collection);

      // Normalize and structure data
      const documentData = {
        source_id: sourceId,
        external_id: packageData.packageId,
        type: docType,
        court: courtInfo.court,
        jurisdiction: courtInfo.jurisdiction,
        docket_number: this.extractDocketNumber(packageDetails),
        case_name: this.extractCaseName(packageDetails),
        date_filed: this.parseDate(packageDetails.dateIssued || packageData.dateIssued),
        date_modified: this.parseDate(packageDetails.lastModified || packageData.lastModified),
        cites: this.extractCitations(packageDetails),
        parties: this.extractParties(packageDetails),
        judges: this.extractJudges(packageDetails),
        attorneys: [],
        metadata: {
          collection: collection,
          congress: packageDetails.congress,
          session: packageDetails.session,
          chamber: packageDetails.chamber,
          packageType: packageDetails.packageType,
          government_author: packageDetails.governmentAuthor,
          source: 'govinfo',
          api_version: 'v1'
        },
        text_summary: this.generateSummary(fullText),
        full_text: fullText,
        sha256: fileHash || crypto.createHash('sha256').update(JSON.stringify(packageDetails)).digest('hex'),
        url: packageDetails.packageLink || `https://www.govinfo.gov/app/details/${packageData.packageId}`,
        pdf_path: pdfPath,
        txt_path: txtPath,
        file_size: fileSize,
        page_count: this.estimatePageCount(fullText)
      };

      // Upsert into database
      await this.upsertDocument(documentData);
      
      // Index in search engines
      await this.indexDocument(documentData);
      
      this.stats.processed++;
      
      if (this.stats.processed % 50 === 0) {
        console.log(`📊 Processed ${this.stats.processed} documents...`);
      }
      
    } catch (error) {
      console.error(`❌ Failed to process package ${packageData.packageId}:`, error.message);
      this.stats.failed++;
    }
  }

  normalizeDocumentType(collection, packageDetails) {
    switch (collection) {
      case 'USCOURTS':
        return 'opinion';
      case 'CFR':
        return 'rule';
      case 'FR':
        return 'rule';
      case 'STATUTE':
        return 'rule';
      case 'BILLS':
        return 'filing';
      case 'CRPT':
        return 'filing';
      default:
        return 'filing';
    }
  }

  extractCourtInfo(packageDetails, collection) {
    if (collection === 'USCOURTS') {
      // Extract court from title or other metadata
      const title = packageDetails.title || '';
      
      // Common court patterns
      const courtPatterns = [
        { pattern: /Supreme Court/i, court: 'Supreme Court', jurisdiction: 'federal' },
        { pattern: /Court of Appeals/i, court: 'Court of Appeals', jurisdiction: 'federal' },
        { pattern: /District Court/i, court: 'District Court', jurisdiction: 'federal' },
        { pattern: /Bankruptcy Court/i, court: 'Bankruptcy Court', jurisdiction: 'federal' },
        { pattern: /Tax Court/i, court: 'Tax Court', jurisdiction: 'federal' }
      ];
      
      for (const { pattern, court, jurisdiction } of courtPatterns) {
        if (pattern.test(title)) {
          return { court, jurisdiction };
        }
      }
    }
    
    return { court: null, jurisdiction: 'federal' };
  }

  extractDocketNumber(packageDetails) {
    const title = packageDetails.title || '';
    
    // Common docket number patterns
    const patterns = [
      /No\.\s*(\d+[-\d]*)/i,
      /Case\s*No\.\s*(\d+[-\d]*)/i,
      /Docket\s*No\.\s*(\d+[-\d]*)/i
    ];
    
    for (const pattern of patterns) {
      const match = title.match(pattern);
      if (match) {
        return match[1];
      }
    }
    
    return null;
  }

  extractCaseName(packageDetails) {
    let title = packageDetails.title || '';
    
    // Clean up title for case name
    title = title.replace(/^United States Court of Appeals.*?:\s*/i, '');
    title = title.replace(/^United States District Court.*?:\s*/i, '');
    title = title.replace(/\s*\(.*?\)\s*$/, ''); // Remove parenthetical at end
    
    return title.trim() || null;
  }

  extractCitations(packageDetails) {
    const citations = [];
    
    // Extract citations from title or content
    const text = packageDetails.title || '';
    
    // Common citation patterns
    const patterns = [
      /(\d+)\s+U\.S\.\s+(\d+)/g,
      /(\d+)\s+F\.\s*(?:2d|3d)?\s+(\d+)/g,
      /(\d+)\s+S\.\s*Ct\.\s+(\d+)/g
    ];
    
    patterns.forEach(pattern => {
      let match;
      while ((match = pattern.exec(text)) !== null) {
        citations.push({
          cite: match[0],
          volume: match[1],
          page: match[2],
          reporter: this.extractReporter(match[0])
        });
      }
    });
    
    return citations;
  }

  extractReporter(cite) {
    if (cite.includes('U.S.')) return 'U.S.';
    if (cite.includes('F.')) return 'F.';
    if (cite.includes('S. Ct.')) return 'S. Ct.';
    return null;
  }

  extractParties(packageDetails) {
    const parties = [];
    const title = packageDetails.title || '';
    
    // Extract party names from title (basic pattern)
    const vsMatch = title.match(/^(.+?)\s+v\.?\s+(.+?)(?:\s|$)/i);
    if (vsMatch) {
      parties.push({
        name: vsMatch[1].trim(),
        role: 'plaintiff',
        type: 'party'
      });
      parties.push({
        name: vsMatch[2].trim(),
        role: 'defendant', 
        type: 'party'
      });
    }
    
    return parties;
  }

  extractJudges(packageDetails) {
    // GovInfo doesn't typically include judge information
    return [];
  }

  parseDate(dateString) {
    if (!dateString) return null;
    
    try {
      return new Date(dateString).toISOString();
    } catch (error) {
      return null;
    }
  }

  generateSummary(text) {
    if (!text || text.length < 100) return '';
    
    // Simple extractive summary - first few sentences
    const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 20);
    return sentences.slice(0, 3).join('. ') + '.';
  }

  estimatePageCount(text) {
    if (!text) return 0;
    // Rough estimate: 500 words per page
    const wordCount = text.split(/\s+/).length;
    return Math.ceil(wordCount / 500);
  }

  async upsertDocument(doc) {
    const query = `
      INSERT INTO documents (
        source_id, external_id, type, court, jurisdiction, docket_number,
        case_name, date_filed, date_modified, cites, parties, judges,
        attorneys, metadata, text_summary, full_text, sha256, url,
        pdf_path, txt_path, file_size, page_count
      ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
      )
      ON CONFLICT (source_id, external_id)
      DO UPDATE SET
        date_modified = EXCLUDED.date_modified,
        metadata = EXCLUDED.metadata,
        full_text = EXCLUDED.full_text,
        updated_at = NOW()
      RETURNING id
    `;
    
    const values = [
      doc.source_id, doc.external_id, doc.type, doc.court, doc.jurisdiction,
      doc.docket_number, doc.case_name, doc.date_filed, doc.date_modified,
      JSON.stringify(doc.cites), JSON.stringify(doc.parties), JSON.stringify(doc.judges),
      JSON.stringify(doc.attorneys), JSON.stringify(doc.metadata), doc.text_summary,
      doc.full_text, doc.sha256, doc.url, doc.pdf_path, doc.txt_path,
      doc.file_size, doc.page_count
    ];
    
    const result = await this.client.query(query, values);
    return result.rows[0].id;
  }

  async indexDocument(doc) {
    // TODO: Implement OpenSearch and Qdrant indexing
    // This would involve:
    // 1. Indexing full document in OpenSearch
    // 2. Chunking text and creating embeddings for Qdrant
    // 3. Storing vectors with metadata payload
  }

  async updateIngestState(collection, cursor, recordsProcessed, recordsFailed) {
    await this.client.query(`
      INSERT INTO ingest_state (source, collection, last_success_timestamp, cursor, status, records_processed, records_failed)
      VALUES ($1, $2, NOW(), $3, 'success', $4, $5)
      ON CONFLICT (source, collection)
      DO UPDATE SET
        last_success_timestamp = NOW(),
        cursor = EXCLUDED.cursor,
        status = EXCLUDED.status,
        records_processed = EXCLUDED.records_processed,
        records_failed = EXCLUDED.records_failed,
        updated_at = NOW()
    `, ['govinfo', collection, cursor, recordsProcessed, recordsFailed]);
  }

  async ingestCollection(collection, sinceDate = null) {
    console.log(`📚 Starting GovInfo ${collection} ingest...`);
    
    let offset = 0;
    let hasMore = true;
    
    // Get last offset if resuming
    if (!sinceDate) {
      const stateResult = await this.client.query(
        'SELECT cursor FROM ingest_state WHERE source = $1 AND collection = $2',
        ['govinfo', collection]
      );
      
      if (stateResult.rows.length > 0 && stateResult.rows[0].cursor) {
        offset = parseInt(stateResult.rows[0].cursor) || 0;
        console.log(`📍 Resuming from offset: ${offset}`);
      }
    }
    
    while (hasMore) {
      const params = {
        offset: offset,
        pageSize: this.batchSize
      };
      
      if (sinceDate) {
        params.lastModified = `range(${sinceDate},)`;
      }
      
      try {
        const response = await this.makeApiRequest(`/collections/${collection}`, params);
        
        if (!response.packages || response.packages.length === 0) {
          hasMore = false;
          break;
        }
        
        console.log(`📄 Processing batch of ${response.packages.length} packages...`);
        
        for (const packageData of response.packages) {
          await this.processPackage(packageData, collection);
        }
        
        // Update offset
        offset += response.packages.length;
        hasMore = response.packages.length === this.batchSize;
        
        // Update ingest state
        await this.updateIngestState(collection, offset.toString(), this.stats.processed, this.stats.failed);
        
        // Log progress
        const elapsed = (Date.now() - this.stats.startTime) / 1000;
        const rate = this.stats.processed / elapsed;
        console.log(`📊 Progress: ${this.stats.processed} processed, ${this.stats.failed} failed, ${rate.toFixed(2)}/sec`);
        
      } catch (error) {
        console.error('❌ Batch processing failed:', error.message);
        this.stats.failed += this.batchSize;
        
        // Continue with next batch on error
        offset += this.batchSize;
        hasMore = true;
      }
    }
  }

  async run(options = {}) {
    console.log('🚀 Starting GovInfo ingest...\n');
    
    try {
      await this.connect();
      
      const collectionsToProcess = options.collection ? [options.collection] : this.collections;
      
      for (const collection of collectionsToProcess) {
        console.log(`\n📚 Processing collection: ${collection}`);
        
        if (options.backfill) {
          await this.ingestCollection(collection);
        } else if (options.since) {
          await this.ingestCollection(collection, options.since);
        } else {
          // Default: incremental update
          const lastUpdate = await this.client.query(
            'SELECT last_success_timestamp FROM ingest_state WHERE source = $1 AND collection = $2',
            ['govinfo', collection]
          );
          
          const sinceDate = lastUpdate.rows.length > 0 
            ? lastUpdate.rows[0].last_success_timestamp.toISOString().split('T')[0]
            : null;
            
          await this.ingestCollection(collection, sinceDate);
        }
      }
      
      console.log('\n🎉 GovInfo ingest completed!');
      console.log(`📊 Final stats: ${this.stats.processed} processed, ${this.stats.failed} failed`);
      console.log(`💾 Downloaded: ${this.stats.downloaded} files, ${(this.stats.bytes / 1024 / 1024).toFixed(2)} MB`);
      
    } catch (error) {
      console.error('\n❌ GovInfo ingest failed:', error.message);
      process.exit(1);
    } finally {
      await this.disconnect();
    }
  }
}

// Command line interface
if (import.meta.url === `file://${process.argv[1]}`) {
  const args = process.argv.slice(2);
  const options = {};
  
  if (args.includes('--backfill')) {
    options.backfill = true;
  } else if (args.includes('--since')) {
    const sinceIndex = args.indexOf('--since');
    options.since = args[sinceIndex + 1];
  }
  
  if (args.includes('--collection')) {
    const collectionIndex = args.indexOf('--collection');
    options.collection = args[collectionIndex + 1];
  }
  
  const ingestor = new GovInfoIngestor();
  ingestor.run(options).catch(error => {
    console.error('Ingest failed:', error);
    process.exit(1);
  });
}

export { GovInfoIngestor };

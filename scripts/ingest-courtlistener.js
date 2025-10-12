#!/usr/bin/env node

/**
 * CourtListener Bulk Ingest Script
 * Fetches legal documents from CourtListener API and processes them for search
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

class CourtListenerIngestor {
  constructor() {
    this.apiToken = process.env.COURTLISTENER_API_TOKEN;
    this.baseUrl = 'https://www.courtlistener.com/api/rest/v3';
    this.dataDir = process.env.DATA_DIR || './data';
    this.rateLimit = parseInt(process.env.COURTLISTENER_RATE_LIMIT) || 5;
    this.batchSize = 100;
    
    if (!this.apiToken) {
      throw new Error('COURTLISTENER_API_TOKEN environment variable is required');
    }
    
    this.client = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    // Rate limiting
    this.lastRequest = 0;
    this.requestInterval = 1000 / this.rateLimit; // ms between requests
    
    // Statistics
    this.stats = {
      processed: 0,
      failed: 0,
      downloaded: 0,
      bytes: 0,
      startTime: Date.now()
    };
    
    // Ensure data directory exists
    mkdirSync(this.dataDir, { recursive: true });
    mkdirSync(join(this.dataDir, 'pdfs'), { recursive: true });
    mkdirSync(join(this.dataDir, 'text'), { recursive: true });
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
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        url.searchParams.append(key, value.toString());
      }
    });

    const response = await fetch(url.toString(), {
      headers: {
        'Authorization': `Token ${this.apiToken}`,
        'User-Agent': 'PA-eDocket-Desktop/1.0'
      }
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  async downloadFile(url, filepath) {
    await this.rateLimit();
    
    const response = await fetch(url, {
      headers: {
        'Authorization': `Token ${this.apiToken}`,
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
      // Try pdftotext first (faster)
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

  async processOpinion(opinion) {
    try {
      const sourceResult = await this.client.query(
        'SELECT id FROM sources WHERE name = $1',
        ['courtlistener']
      );
      const sourceId = sourceResult.rows[0].id;

      // Download PDF if available
      let pdfPath = null;
      let txtPath = null;
      let fileHash = null;
      let fileSize = null;

      if (opinion.download_url) {
        const filename = `${opinion.id}.pdf`;
        pdfPath = join(this.dataDir, 'pdfs', filename);
        
        if (!existsSync(pdfPath)) {
          const downloadResult = await this.downloadFile(opinion.download_url, pdfPath);
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

      // Normalize and structure data
      const documentData = {
        source_id: sourceId,
        external_id: opinion.id.toString(),
        type: 'opinion',
        court: opinion.cluster?.docket?.court || null,
        jurisdiction: opinion.cluster?.docket?.court_id || null,
        docket_number: opinion.cluster?.docket?.docket_number || null,
        case_name: opinion.cluster?.case_name || null,
        date_filed: opinion.cluster?.date_filed || null,
        date_modified: opinion.date_modified || null,
        cites: this.extractCitations(opinion),
        parties: this.extractParties(opinion),
        judges: this.extractJudges(opinion),
        attorneys: this.extractAttorneys(opinion),
        metadata: {
          cluster_id: opinion.cluster?.id,
          court_id: opinion.cluster?.docket?.court_id,
          nature_of_suit: opinion.cluster?.docket?.nature_of_suit,
          source: 'courtlistener',
          api_version: 'v3'
        },
        text_summary: this.generateSummary(fullText),
        full_text: fullText,
        sha256: fileHash || crypto.createHash('sha256').update(JSON.stringify(opinion)).digest('hex'),
        url: `https://www.courtlistener.com${opinion.absolute_url}`,
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
      
      if (this.stats.processed % 100 === 0) {
        console.log(`📊 Processed ${this.stats.processed} documents...`);
      }
      
    } catch (error) {
      console.error(`❌ Failed to process opinion ${opinion.id}:`, error.message);
      this.stats.failed++;
    }
  }

  extractCitations(opinion) {
    const citations = [];
    
    if (opinion.cluster?.citations) {
      opinion.cluster.citations.forEach(cite => {
        citations.push({
          cite: cite.cite,
          type: cite.type,
          reporter: cite.reporter,
          page: cite.page,
          year: cite.year
        });
      });
    }
    
    return citations;
  }

  extractParties(opinion) {
    const parties = [];
    
    if (opinion.cluster?.docket?.parties) {
      opinion.cluster.docket.parties.forEach(party => {
        parties.push({
          name: party.name,
          role: party.role,
          type: party.party_type
        });
      });
    }
    
    return parties;
  }

  extractJudges(opinion) {
    const judges = [];
    
    if (opinion.author) {
      judges.push({
        name: opinion.author.name_full,
        role: 'author'
      });
    }
    
    if (opinion.joined_by) {
      opinion.joined_by.forEach(judge => {
        judges.push({
          name: judge.name_full,
          role: 'joined'
        });
      });
    }
    
    return judges;
  }

  extractAttorneys(opinion) {
    const attorneys = [];
    
    if (opinion.cluster?.docket?.attorneys) {
      opinion.cluster.docket.attorneys.forEach(attorney => {
        attorneys.push({
          name: attorney.name,
          firm: attorney.firm,
          role: attorney.role
        });
      });
    }
    
    return attorneys;
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
    `, ['courtlistener', collection, cursor, recordsProcessed, recordsFailed]);
  }

  async ingestOpinions(sinceDate = null) {
    console.log('📚 Starting CourtListener opinions ingest...');
    
    let cursor = null;
    let hasMore = true;
    
    // Get last cursor if resuming
    if (!sinceDate) {
      const stateResult = await this.client.query(
        'SELECT cursor FROM ingest_state WHERE source = $1 AND collection = $2',
        ['courtlistener', 'opinions']
      );
      
      if (stateResult.rows.length > 0) {
        cursor = stateResult.rows[0].cursor;
        console.log(`📍 Resuming from cursor: ${cursor}`);
      }
    }
    
    while (hasMore) {
      const params = {
        format: 'json',
        order_by: 'id',
        page_size: this.batchSize
      };
      
      if (cursor) {
        params.cursor = cursor;
      }
      
      if (sinceDate) {
        params.date_modified__gte = sinceDate;
      }
      
      try {
        const response = await this.makeApiRequest('/opinions/', params);
        
        console.log(`📄 Processing batch of ${response.results.length} opinions...`);
        
        for (const opinion of response.results) {
          await this.processOpinion(opinion);
        }
        
        // Update cursor
        cursor = response.next ? new URL(response.next).searchParams.get('cursor') : null;
        hasMore = !!response.next;
        
        // Update ingest state
        await this.updateIngestState('opinions', cursor, this.stats.processed, this.stats.failed);
        
        // Log progress
        const elapsed = (Date.now() - this.stats.startTime) / 1000;
        const rate = this.stats.processed / elapsed;
        console.log(`📊 Progress: ${this.stats.processed} processed, ${this.stats.failed} failed, ${rate.toFixed(2)}/sec`);
        
      } catch (error) {
        console.error('❌ Batch processing failed:', error.message);
        this.stats.failed += this.batchSize;
        
        // Continue with next batch on error
        if (cursor) {
          hasMore = true;
        } else {
          hasMore = false;
        }
      }
    }
  }

  async run(options = {}) {
    console.log('🚀 Starting CourtListener ingest...\n');
    
    try {
      await this.connect();
      
      if (options.backfill) {
        await this.ingestOpinions();
      } else if (options.since) {
        await this.ingestOpinions(options.since);
      } else {
        // Default: incremental update
        const lastUpdate = await this.client.query(
          'SELECT last_success_timestamp FROM ingest_state WHERE source = $1 AND collection = $2',
          ['courtlistener', 'opinions']
        );
        
        const sinceDate = lastUpdate.rows.length > 0 
          ? lastUpdate.rows[0].last_success_timestamp.toISOString()
          : null;
          
        await this.ingestOpinions(sinceDate);
      }
      
      console.log('\n🎉 CourtListener ingest completed!');
      console.log(`📊 Final stats: ${this.stats.processed} processed, ${this.stats.failed} failed`);
      console.log(`💾 Downloaded: ${this.stats.downloaded} files, ${(this.stats.bytes / 1024 / 1024).toFixed(2)} MB`);
      
    } catch (error) {
      console.error('\n❌ CourtListener ingest failed:', error.message);
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
  
  const ingestor = new CourtListenerIngestor();
  ingestor.run(options).catch(error => {
    console.error('Ingest failed:', error);
    process.exit(1);
  });
}

export { CourtListenerIngestor };

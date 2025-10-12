#!/usr/bin/env node

/**
 * PA eDocket Desktop Background Worker
 * Handles document ingestion, processing, and automation tasks
 */

import { config } from 'dotenv';
import { Client } from 'pg';
import Bull from 'bull';
import IORedis from 'ioredis';
import winston from 'winston';
import { CourtListenerIngestor } from './ingest-courtlistener.js';
import { GovInfoIngestor } from './ingest-govinfo.js';
import { CitationEngine } from './citation-engine.js';
import { DocumentProcessor } from './document-processor.js';

// Load environment variables
config();

// Configure logging
const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ filename: 'logs/worker-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/worker-combined.log' })
  ]
});

class BackgroundWorker {
  constructor() {
    this.concurrency = parseInt(process.env.WORKER_CONCURRENCY) || 4;
    
    // Database connection
    this.db = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    // Redis connection for job queue
    this.redis = new IORedis(process.env.REDIS_URL || 'redis://localhost:6379');
    
    // Job queues
    this.ingestQueue = new Bull('ingest', {
      redis: this.redis,
      defaultJobOptions: {
        removeOnComplete: 100,
        removeOnFail: 50,
        attempts: 3,
        backoff: {
          type: 'exponential',
          delay: 2000
        }
      }
    });
    
    this.processingQueue = new Bull('processing', {
      redis: this.redis,
      defaultJobOptions: {
        removeOnComplete: 100,
        removeOnFail: 50,
        attempts: 3,
        backoff: {
          type: 'exponential',
          delay: 2000
        }
      }
    });
    
    this.exportQueue = new Bull('export', {
      redis: this.redis,
      defaultJobOptions: {
        removeOnComplete: 50,
        removeOnFail: 25,
        attempts: 2,
        backoff: {
          type: 'exponential',
          delay: 1000
        }
      }
    });
    
    // Initialize processors
    this.citationEngine = new CitationEngine();
    this.documentProcessor = new DocumentProcessor();
    
    // Statistics
    this.stats = {
      processed: 0,
      failed: 0,
      startTime: Date.now()
    };
  }

  async start() {
    try {
      // Connect to database
      await this.db.connect();
      logger.info('Connected to PostgreSQL database');
      
      // Setup job processors
      this.setupJobProcessors();
      
      // Schedule recurring jobs
      await this.scheduleRecurringJobs();
      
      // Setup graceful shutdown
      process.on('SIGTERM', this.shutdown.bind(this));
      process.on('SIGINT', this.shutdown.bind(this));
      
      logger.info(`Background worker started with concurrency: ${this.concurrency}`);
      
      // Keep the process alive
      setInterval(() => {
        const uptime = (Date.now() - this.stats.startTime) / 1000;
        logger.info('Worker heartbeat', {
          uptime: uptime,
          processed: this.stats.processed,
          failed: this.stats.failed,
          queues: {
            ingest: this.ingestQueue.waiting,
            processing: this.processingQueue.waiting,
            export: this.exportQueue.waiting
          }
        });
      }, 60000); // Every minute
      
    } catch (error) {
      logger.error('Failed to start background worker', error);
      process.exit(1);
    }
  }

  setupJobProcessors() {
    // Ingest job processor
    this.ingestQueue.process('courtlistener', this.concurrency, async (job) => {
      logger.info('Processing CourtListener ingest job', { jobId: job.id, data: job.data });
      
      try {
        const ingestor = new CourtListenerIngestor();
        await ingestor.run(job.data.options);
        
        this.stats.processed++;
        logger.info('CourtListener ingest job completed', { jobId: job.id });
        
      } catch (error) {
        this.stats.failed++;
        logger.error('CourtListener ingest job failed', { jobId: job.id, error: error.message });
        throw error;
      }
    });

    this.ingestQueue.process('govinfo', this.concurrency, async (job) => {
      logger.info('Processing GovInfo ingest job', { jobId: job.id, data: job.data });
      
      try {
        const ingestor = new GovInfoIngestor();
        await ingestor.run(job.data.options);
        
        this.stats.processed++;
        logger.info('GovInfo ingest job completed', { jobId: job.id });
        
      } catch (error) {
        this.stats.failed++;
        logger.error('GovInfo ingest job failed', { jobId: job.id, error: error.message });
        throw error;
      }
    });

    // Document processing job processor
    this.processingQueue.process('extract-citations', this.concurrency, async (job) => {
      logger.info('Processing citation extraction job', { jobId: job.id, data: job.data });
      
      try {
        const { documentId } = job.data;
        
        // Get document from database
        const result = await this.db.query('SELECT * FROM documents WHERE id = $1', [documentId]);
        if (result.rows.length === 0) {
          throw new Error(`Document ${documentId} not found`);
        }
        
        const document = result.rows[0];
        
        // Extract citations
        const citations = await this.citationEngine.extractCitations(document.full_text);
        
        // Update document with extracted citations
        await this.db.query(
          'UPDATE documents SET cites = $1, updated_at = NOW() WHERE id = $2',
          [JSON.stringify(citations), documentId]
        );
        
        this.stats.processed++;
        logger.info('Citation extraction completed', { jobId: job.id, documentId, citationCount: citations.length });
        
      } catch (error) {
        this.stats.failed++;
        logger.error('Citation extraction failed', { jobId: job.id, error: error.message });
        throw error;
      }
    });

    this.processingQueue.process('generate-summary', this.concurrency, async (job) => {
      logger.info('Processing summary generation job', { jobId: job.id, data: job.data });
      
      try {
        const { documentId } = job.data;
        
        // Get document from database
        const result = await this.db.query('SELECT * FROM documents WHERE id = $1', [documentId]);
        if (result.rows.length === 0) {
          throw new Error(`Document ${documentId} not found`);
        }
        
        const document = result.rows[0];
        
        // Generate summary
        const summary = await this.documentProcessor.generateSummary(document.full_text);
        
        // Update document with summary
        await this.db.query(
          'UPDATE documents SET text_summary = $1, updated_at = NOW() WHERE id = $2',
          [summary, documentId]
        );
        
        this.stats.processed++;
        logger.info('Summary generation completed', { jobId: job.id, documentId });
        
      } catch (error) {
        this.stats.failed++;
        logger.error('Summary generation failed', { jobId: job.id, error: error.message });
        throw error;
      }
    });

    // Export job processor
    this.exportQueue.process('export-documents', this.concurrency, async (job) => {
      logger.info('Processing export job', { jobId: job.id, data: job.data });
      
      try {
        const { exportId, format, query, filters } = job.data;
        
        // Update export status
        await this.db.query(
          'UPDATE export_history SET status = $1 WHERE id = $2',
          ['processing', exportId]
        );
        
        // Process export
        const result = await this.documentProcessor.exportDocuments(format, query, filters);
        
        // Update export record
        await this.db.query(`
          UPDATE export_history 
          SET status = $1, file_path = $2, file_size = $3, record_count = $4, completed_at = NOW()
          WHERE id = $5
        `, ['completed', result.filePath, result.fileSize, result.recordCount, exportId]);
        
        this.stats.processed++;
        logger.info('Export job completed', { jobId: job.id, exportId, format, recordCount: result.recordCount });
        
      } catch (error) {
        this.stats.failed++;
        
        // Update export status to failed
        await this.db.query(
          'UPDATE export_history SET status = $1, error_message = $2 WHERE id = $3',
          ['failed', error.message, job.data.exportId]
        );
        
        logger.error('Export job failed', { jobId: job.id, error: error.message });
        throw error;
      }
    });

    // Job event handlers
    this.ingestQueue.on('completed', (job, result) => {
      logger.info('Ingest job completed', { jobId: job.id, result });
    });

    this.ingestQueue.on('failed', (job, err) => {
      logger.error('Ingest job failed', { jobId: job.id, error: err.message });
    });

    this.processingQueue.on('completed', (job, result) => {
      logger.info('Processing job completed', { jobId: job.id, result });
    });

    this.processingQueue.on('failed', (job, err) => {
      logger.error('Processing job failed', { jobId: job.id, error: err.message });
    });

    this.exportQueue.on('completed', (job, result) => {
      logger.info('Export job completed', { jobId: job.id, result });
    });

    this.exportQueue.on('failed', (job, err) => {
      logger.error('Export job failed', { jobId: job.id, error: err.message });
    });
  }

  async scheduleRecurringJobs() {
    // Schedule incremental CourtListener sync every 6 hours
    await this.ingestQueue.add('courtlistener', {
      options: { incremental: true }
    }, {
      repeat: { cron: '0 */6 * * *' }, // Every 6 hours
      jobId: 'courtlistener-incremental'
    });

    // Schedule incremental GovInfo sync every 12 hours
    await this.ingestQueue.add('govinfo', {
      options: { incremental: true }
    }, {
      repeat: { cron: '0 */12 * * *' }, // Every 12 hours
      jobId: 'govinfo-incremental'
    });

    // Schedule cleanup of old cache entries daily
    await this.processingQueue.add('cleanup-cache', {}, {
      repeat: { cron: '0 2 * * *' }, // Daily at 2 AM
      jobId: 'cleanup-cache'
    });

    logger.info('Recurring jobs scheduled');
  }

  async queueIngestJob(source, options = {}) {
    const job = await this.ingestQueue.add(source, { options });
    logger.info('Ingest job queued', { jobId: job.id, source, options });
    return job;
  }

  async queueProcessingJob(type, data) {
    const job = await this.processingQueue.add(type, data);
    logger.info('Processing job queued', { jobId: job.id, type, data });
    return job;
  }

  async queueExportJob(exportId, format, query, filters) {
    const job = await this.exportQueue.add('export-documents', {
      exportId,
      format,
      query,
      filters
    });
    logger.info('Export job queued', { jobId: job.id, exportId, format });
    return job;
  }

  async getQueueStats() {
    const ingestStats = await this.ingestQueue.getJobCounts();
    const processingStats = await this.processingQueue.getJobCounts();
    const exportStats = await this.exportQueue.getJobCounts();

    return {
      ingest: ingestStats,
      processing: processingStats,
      export: exportStats,
      worker: {
        processed: this.stats.processed,
        failed: this.stats.failed,
        uptime: (Date.now() - this.stats.startTime) / 1000
      }
    };
  }

  async shutdown() {
    logger.info('Shutting down background worker...');
    
    try {
      // Close job queues
      await this.ingestQueue.close();
      await this.processingQueue.close();
      await this.exportQueue.close();
      
      // Close Redis connection
      await this.redis.quit();
      
      // Close database connection
      await this.db.end();
      
      logger.info('Background worker shutdown complete');
      process.exit(0);
      
    } catch (error) {
      logger.error('Error during shutdown', error);
      process.exit(1);
    }
  }
}

// Start worker if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const worker = new BackgroundWorker();
  worker.start().catch(error => {
    console.error('Failed to start background worker:', error);
    process.exit(1);
  });
}

export { BackgroundWorker };

#!/usr/bin/env node

/**
 * PA eDocket Desktop API Server
 * Production-grade Express.js API server for the desktop application
 */

import { config } from 'dotenv';
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import { Client } from 'pg';
import fetch from 'node-fetch';
import https from 'https';
import winston from 'winston';
import promClient from 'prom-client';

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
    new winston.transports.File({ filename: 'logs/api-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/api-combined.log' })
  ]
});

// Configure metrics
const register = new promClient.Registry();
promClient.collectDefaultMetrics({ register });

const httpRequestDuration = new promClient.Histogram({
  name: 'http_request_duration_seconds',
  help: 'Duration of HTTP requests in seconds',
  labelNames: ['method', 'route', 'status_code'],
  buckets: [0.1, 0.5, 1, 2, 5]
});

const httpRequestTotal = new promClient.Counter({
  name: 'http_requests_total',
  help: 'Total number of HTTP requests',
  labelNames: ['method', 'route', 'status_code']
});

register.registerMetric(httpRequestDuration);
register.registerMetric(httpRequestTotal);

class APIServer {
  constructor() {
    this.app = express();
    this.port = process.env.PORT || 3000;
    this.metricsPort = process.env.METRICS_PORT || 9090;
    
    // Database connection
    this.db = new Client({
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
    });
    
    // Search clients
    this.opensearchUrl = process.env.OPENSEARCH_URL;
    this.opensearchAuth = this.getOpenSearchAuth();
    this.qdrantUrl = process.env.QDRANT_URL;
    this.qdrantApiKey = process.env.QDRANT_API_KEY;
    
    this.httpsAgent = process.env.NODE_ENV === 'development' ? 
      new https.Agent({ rejectUnauthorized: false }) : undefined;
  }

  getOpenSearchAuth() {
    if (process.env.OPENSEARCH_USERNAME && process.env.OPENSEARCH_PASSWORD) {
      return Buffer.from(
        `${process.env.OPENSEARCH_USERNAME}:${process.env.OPENSEARCH_PASSWORD}`
      ).toString('base64');
    }
    return null;
  }

  async setupMiddleware() {
    // Security middleware
    this.app.use(helmet({
      contentSecurityPolicy: {
        directives: {
          defaultSrc: ["'self'"],
          scriptSrc: ["'self'", "'unsafe-inline'"],
          styleSrc: ["'self'", "'unsafe-inline'", "https://fonts.googleapis.com"],
          fontSrc: ["'self'", "https://fonts.gstatic.com"],
          imgSrc: ["'self'", "data:", "https:"],
          connectSrc: ["'self'", "https:"]
        }
      }
    }));

    // CORS configuration
    this.app.use(cors({
      origin: process.env.NODE_ENV === 'production' 
        ? ['tauri://localhost', 'https://localhost:1420']
        : true,
      credentials: true
    }));

    // Compression and parsing
    this.app.use(compression());
    this.app.use(express.json({ limit: '10mb' }));
    this.app.use(express.urlencoded({ extended: true, limit: '10mb' }));

    // Request logging and metrics
    this.app.use((req, res, next) => {
      const start = Date.now();
      
      res.on('finish', () => {
        const duration = (Date.now() - start) / 1000;
        const route = req.route?.path || req.path;
        
        httpRequestDuration
          .labels(req.method, route, res.statusCode.toString())
          .observe(duration);
          
        httpRequestTotal
          .labels(req.method, route, res.statusCode.toString())
          .inc();
        
        logger.info('HTTP Request', {
          method: req.method,
          url: req.url,
          status: res.statusCode,
          duration: duration,
          userAgent: req.get('User-Agent'),
          ip: req.ip
        });
      });
      
      next();
    });
  }

  async setupRoutes() {
    // Health check
    this.app.get('/health', async (req, res) => {
      try {
        await this.db.query('SELECT 1');
        res.json({ 
          status: 'healthy', 
          timestamp: new Date().toISOString(),
          version: process.env.npm_package_version || '1.0.0'
        });
      } catch (error) {
        logger.error('Health check failed', error);
        res.status(503).json({ 
          status: 'unhealthy', 
          error: error.message 
        });
      }
    });

    // Metrics endpoint
    this.app.get('/metrics', async (req, res) => {
      res.set('Content-Type', register.contentType);
      res.end(await register.metrics());
    });

    // Search endpoints
    this.app.get('/api/search', this.handleSearch.bind(this));
    this.app.get('/api/documents/:id', this.handleGetDocument.bind(this));
    this.app.get('/api/dockets/:docketNumber', this.handleGetDocket.bind(this));
    
    // Export endpoints
    this.app.post('/api/export', this.handleExport.bind(this));
    this.app.get('/api/export/:id/download', this.handleDownloadExport.bind(this));
    
    // Watchlist endpoints
    this.app.get('/api/watchlists', this.handleGetWatchlists.bind(this));
    this.app.post('/api/watchlists', this.handleCreateWatchlist.bind(this));
    this.app.post('/api/watchlists/:id/items', this.handleAddToWatchlist.bind(this));
    
    // Settings endpoints
    this.app.get('/api/settings', this.handleGetSettings.bind(this));
    this.app.put('/api/settings', this.handleUpdateSettings.bind(this));
    
    // Statistics endpoints
    this.app.get('/api/stats', this.handleGetStats.bind(this));
    
    // Error handling
    this.app.use((error, req, res, next) => {
      logger.error('Unhandled error', error);
      res.status(500).json({
        error: 'Internal server error',
        message: process.env.NODE_ENV === 'development' ? error.message : 'Something went wrong'
      });
    });

    // 404 handler
    this.app.use((req, res) => {
      res.status(404).json({ error: 'Not found' });
    });
  }

  async handleSearch(req, res) {
    try {
      const {
        q,
        court,
        jurisdiction,
        type,
        dateFrom,
        dateTo,
        page = 1,
        pageSize = 25
      } = req.query;

      // Validate parameters
      const validatedPageSize = Math.min(parseInt(pageSize) || 25, 100);
      const validatedPage = Math.max(parseInt(page) || 1, 1);
      const offset = (validatedPage - 1) * validatedPageSize;

      // Build search query
      let query = `
        SELECT d.*, s.name as source_name
        FROM documents d
        JOIN sources s ON d.source_id = s.id
        WHERE 1=1
      `;
      const params = [];
      let paramIndex = 1;

      if (q) {
        query += ` AND (
          to_tsvector('english', d.full_text) @@ plainto_tsquery('english', $${paramIndex})
          OR to_tsvector('english', d.case_name) @@ plainto_tsquery('english', $${paramIndex})
          OR d.docket_number ILIKE $${paramIndex + 1}
        )`;
        params.push(q, `%${q}%`);
        paramIndex += 2;
      }

      if (court) {
        query += ` AND d.court = $${paramIndex}`;
        params.push(court);
        paramIndex++;
      }

      if (jurisdiction) {
        query += ` AND d.jurisdiction = $${paramIndex}`;
        params.push(jurisdiction);
        paramIndex++;
      }

      if (type) {
        query += ` AND d.type = $${paramIndex}`;
        params.push(type);
        paramIndex++;
      }

      if (dateFrom) {
        query += ` AND d.date_filed >= $${paramIndex}`;
        params.push(dateFrom);
        paramIndex++;
      }

      if (dateTo) {
        query += ` AND d.date_filed <= $${paramIndex}`;
        params.push(dateTo);
        paramIndex++;
      }

      // Count total results
      const countQuery = query.replace('SELECT d.*, s.name as source_name', 'SELECT COUNT(*)');
      const countResult = await this.db.query(countQuery, params);
      const total = parseInt(countResult.rows[0].count);

      // Add pagination
      query += ` ORDER BY d.date_filed DESC, d.id DESC LIMIT $${paramIndex} OFFSET $${paramIndex + 1}`;
      params.push(validatedPageSize, offset);

      // Execute search
      const result = await this.db.query(query, params);

      res.json({
        results: result.rows,
        pagination: {
          page: validatedPage,
          pageSize: validatedPageSize,
          total: total,
          totalPages: Math.ceil(total / validatedPageSize)
        }
      });

    } catch (error) {
      logger.error('Search failed', error);
      res.status(500).json({ error: 'Search failed', message: error.message });
    }
  }

  async handleGetDocument(req, res) {
    try {
      const { id } = req.params;
      
      const result = await this.db.query(`
        SELECT d.*, s.name as source_name
        FROM documents d
        JOIN sources s ON d.source_id = s.id
        WHERE d.id = $1
      `, [id]);

      if (result.rows.length === 0) {
        return res.status(404).json({ error: 'Document not found' });
      }

      res.json(result.rows[0]);

    } catch (error) {
      logger.error('Get document failed', error);
      res.status(500).json({ error: 'Failed to get document', message: error.message });
    }
  }

  async handleGetDocket(req, res) {
    try {
      const { docketNumber } = req.params;
      
      const result = await this.db.query(`
        SELECT d.*, s.name as source_name
        FROM documents d
        JOIN sources s ON d.source_id = s.id
        WHERE d.docket_number = $1
        ORDER BY d.date_filed DESC
      `, [docketNumber]);

      res.json({
        docketNumber,
        documents: result.rows
      });

    } catch (error) {
      logger.error('Get docket failed', error);
      res.status(500).json({ error: 'Failed to get docket', message: error.message });
    }
  }

  async handleExport(req, res) {
    try {
      const { format, query, filters } = req.body;
      
      // Create export job
      const jobResult = await this.db.query(`
        INSERT INTO export_history (export_type, query_params, status)
        VALUES ($1, $2, 'pending')
        RETURNING id
      `, [format, JSON.stringify({ query, filters })]);

      const exportId = jobResult.rows[0].id;

      // Queue export job (would be handled by worker)
      // For now, return the export ID
      res.json({
        exportId,
        status: 'pending',
        message: 'Export job queued'
      });

    } catch (error) {
      logger.error('Export failed', error);
      res.status(500).json({ error: 'Export failed', message: error.message });
    }
  }

  async handleDownloadExport(req, res) {
    try {
      const { id } = req.params;
      
      const result = await this.db.query(`
        SELECT * FROM export_history WHERE id = $1
      `, [id]);

      if (result.rows.length === 0) {
        return res.status(404).json({ error: 'Export not found' });
      }

      const exportRecord = result.rows[0];
      
      if (exportRecord.status !== 'completed') {
        return res.status(202).json({ 
          status: exportRecord.status,
          message: 'Export not ready'
        });
      }

      // Serve the file
      res.download(exportRecord.file_path);

    } catch (error) {
      logger.error('Download export failed', error);
      res.status(500).json({ error: 'Download failed', message: error.message });
    }
  }

  async handleGetWatchlists(req, res) {
    try {
      const result = await this.db.query(`
        SELECT w.*, 
               COUNT(wi.id) as item_count
        FROM watchlists w
        LEFT JOIN watchlist_items wi ON w.id = wi.watchlist_id
        GROUP BY w.id
        ORDER BY w.created_at DESC
      `);

      res.json(result.rows);

    } catch (error) {
      logger.error('Get watchlists failed', error);
      res.status(500).json({ error: 'Failed to get watchlists', message: error.message });
    }
  }

  async handleCreateWatchlist(req, res) {
    try {
      const { name, description } = req.body;
      
      const result = await this.db.query(`
        INSERT INTO watchlists (name, description)
        VALUES ($1, $2)
        RETURNING *
      `, [name, description]);

      res.status(201).json(result.rows[0]);

    } catch (error) {
      logger.error('Create watchlist failed', error);
      res.status(500).json({ error: 'Failed to create watchlist', message: error.message });
    }
  }

  async handleAddToWatchlist(req, res) {
    try {
      const { id } = req.params;
      const { documentId, docketNumber, court, notes } = req.body;
      
      const result = await this.db.query(`
        INSERT INTO watchlist_items (watchlist_id, document_id, docket_number, court, notes)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
      `, [id, documentId, docketNumber, court, notes]);

      res.status(201).json(result.rows[0]);

    } catch (error) {
      logger.error('Add to watchlist failed', error);
      res.status(500).json({ error: 'Failed to add to watchlist', message: error.message });
    }
  }

  async handleGetSettings(req, res) {
    try {
      const result = await this.db.query(`
        SELECT key, value, description FROM settings ORDER BY key
      `);

      const settings = {};
      result.rows.forEach(row => {
        settings[row.key] = row.value;
      });

      res.json(settings);

    } catch (error) {
      logger.error('Get settings failed', error);
      res.status(500).json({ error: 'Failed to get settings', message: error.message });
    }
  }

  async handleUpdateSettings(req, res) {
    try {
      const updates = req.body;
      
      for (const [key, value] of Object.entries(updates)) {
        await this.db.query(`
          INSERT INTO settings (key, value)
          VALUES ($1, $2)
          ON CONFLICT (key)
          DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()
        `, [key, JSON.stringify(value)]);
      }

      res.json({ message: 'Settings updated successfully' });

    } catch (error) {
      logger.error('Update settings failed', error);
      res.status(500).json({ error: 'Failed to update settings', message: error.message });
    }
  }

  async handleGetStats(req, res) {
    try {
      const stats = await this.db.query(`
        SELECT 
          COUNT(*) as total_documents,
          COUNT(DISTINCT court) as unique_courts,
          COUNT(DISTINCT jurisdiction) as unique_jurisdictions,
          MIN(date_filed) as earliest_date,
          MAX(date_filed) as latest_date,
          SUM(file_size) as total_file_size
        FROM documents
      `);

      const sourceStats = await this.db.query(`
        SELECT s.name, COUNT(d.id) as document_count
        FROM sources s
        LEFT JOIN documents d ON s.id = d.source_id
        GROUP BY s.id, s.name
        ORDER BY document_count DESC
      `);

      const typeStats = await this.db.query(`
        SELECT type, COUNT(*) as count
        FROM documents
        GROUP BY type
        ORDER BY count DESC
      `);

      res.json({
        overview: stats.rows[0],
        by_source: sourceStats.rows,
        by_type: typeStats.rows
      });

    } catch (error) {
      logger.error('Get stats failed', error);
      res.status(500).json({ error: 'Failed to get stats', message: error.message });
    }
  }

  async start() {
    try {
      // Connect to database
      await this.db.connect();
      logger.info('Connected to PostgreSQL database');

      // Setup middleware and routes
      await this.setupMiddleware();
      await this.setupRoutes();

      // Start main server
      this.server = this.app.listen(this.port, () => {
        logger.info(`API server started on port ${this.port}`);
      });

      // Start metrics server
      const metricsApp = express();
      metricsApp.get('/metrics', async (req, res) => {
        res.set('Content-Type', register.contentType);
        res.end(await register.metrics());
      });
      
      this.metricsServer = metricsApp.listen(this.metricsPort, () => {
        logger.info(`Metrics server started on port ${this.metricsPort}`);
      });

      // Graceful shutdown
      process.on('SIGTERM', this.shutdown.bind(this));
      process.on('SIGINT', this.shutdown.bind(this));

    } catch (error) {
      logger.error('Failed to start API server', error);
      process.exit(1);
    }
  }

  async shutdown() {
    logger.info('Shutting down API server...');
    
    if (this.server) {
      this.server.close();
    }
    
    if (this.metricsServer) {
      this.metricsServer.close();
    }
    
    if (this.db) {
      await this.db.end();
    }
    
    logger.info('API server shutdown complete');
    process.exit(0);
  }
}

// Start server if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const server = new APIServer();
  server.start().catch(error => {
    console.error('Failed to start API server:', error);
    process.exit(1);
  });
}

export { APIServer };

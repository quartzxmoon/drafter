#!/usr/bin/env node

/**
 * Search Index Creation Script
 * Creates and configures OpenSearch and Qdrant indexes for document search
 */

import { config } from 'dotenv';
import fetch from 'node-fetch';
import https from 'https';

// Load environment variables
config();

class SearchIndexManager {
  constructor() {
    this.opensearchUrl = process.env.OPENSEARCH_URL;
    this.opensearchAuth = this.getOpenSearchAuth();
    this.qdrantUrl = process.env.QDRANT_URL;
    this.qdrantApiKey = process.env.QDRANT_API_KEY;
    
    // Allow self-signed certificates in development
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

  async makeOpenSearchRequest(path, method = 'GET', body = null) {
    const headers = {
      'Content-Type': 'application/json'
    };
    
    if (this.opensearchAuth) {
      headers['Authorization'] = `Basic ${this.opensearchAuth}`;
    }

    const response = await fetch(`${this.opensearchUrl}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : null,
      agent: this.httpsAgent
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`OpenSearch request failed: ${response.status} ${error}`);
    }

    return response.json();
  }

  async makeQdrantRequest(path, method = 'GET', body = null) {
    const headers = {
      'Content-Type': 'application/json'
    };
    
    if (this.qdrantApiKey) {
      headers['api-key'] = this.qdrantApiKey;
    }

    const response = await fetch(`${this.qdrantUrl}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : null
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Qdrant request failed: ${response.status} ${error}`);
    }

    return response.json();
  }

  async createOpenSearchIndex() {
    console.log('🔍 Creating OpenSearch index: documents_v1');

    const indexMapping = {
      settings: {
        number_of_shards: 1,
        number_of_replicas: 0,
        analysis: {
          analyzer: {
            legal_analyzer: {
              type: 'custom',
              tokenizer: 'standard',
              filter: [
                'lowercase',
                'stop',
                'legal_synonyms',
                'legal_shingles'
              ]
            }
          },
          filter: {
            legal_synonyms: {
              type: 'synonym',
              synonyms: [
                'plaintiff,petitioner',
                'defendant,respondent',
                'court,tribunal',
                'judge,justice',
                'attorney,lawyer,counsel'
              ]
            },
            legal_shingles: {
              type: 'shingle',
              min_shingle_size: 2,
              max_shingle_size: 3
            }
          }
        }
      },
      mappings: {
        properties: {
          id: { type: 'long' },
          source_id: { type: 'integer' },
          external_id: { type: 'keyword' },
          type: { 
            type: 'keyword',
            fields: {
              text: { type: 'text' }
            }
          },
          court: { 
            type: 'keyword',
            fields: {
              text: { type: 'text' }
            }
          },
          jurisdiction: { type: 'keyword' },
          docket_number: { 
            type: 'keyword',
            fields: {
              text: { type: 'text' }
            }
          },
          case_name: {
            type: 'text',
            analyzer: 'legal_analyzer',
            fields: {
              keyword: { type: 'keyword' },
              shingles: {
                type: 'text',
                analyzer: 'legal_analyzer'
              }
            }
          },
          date_filed: { type: 'date' },
          date_modified: { type: 'date' },
          full_text: {
            type: 'text',
            analyzer: 'legal_analyzer',
            fields: {
              shingles: {
                type: 'text',
                analyzer: 'legal_analyzer'
              }
            }
          },
          text_summary: {
            type: 'text',
            analyzer: 'legal_analyzer'
          },
          cites: {
            type: 'nested',
            properties: {
              cite: { type: 'keyword' },
              type: { type: 'keyword' },
              reporter: { type: 'keyword' },
              page: { type: 'integer' },
              year: { type: 'integer' }
            }
          },
          parties: {
            type: 'nested',
            properties: {
              name: {
                type: 'text',
                analyzer: 'legal_analyzer',
                fields: {
                  keyword: { type: 'keyword' }
                }
              },
              role: { type: 'keyword' },
              type: { type: 'keyword' }
            }
          },
          judges: {
            type: 'nested',
            properties: {
              name: {
                type: 'text',
                fields: {
                  keyword: { type: 'keyword' }
                }
              },
              role: { type: 'keyword' }
            }
          },
          attorneys: {
            type: 'nested',
            properties: {
              name: {
                type: 'text',
                fields: {
                  keyword: { type: 'keyword' }
                }
              },
              firm: { type: 'text' },
              role: { type: 'keyword' }
            }
          },
          metadata: { type: 'object' },
          sha256: { type: 'keyword' },
          url: { type: 'keyword' },
          file_size: { type: 'long' },
          page_count: { type: 'integer' },
          created_at: { type: 'date' },
          updated_at: { type: 'date' }
        }
      }
    };

    try {
      // Check if index exists
      try {
        await this.makeOpenSearchRequest('/documents_v1');
        console.log('⚠️  Index documents_v1 already exists, deleting...');
        await this.makeOpenSearchRequest('/documents_v1', 'DELETE');
      } catch (error) {
        // Index doesn't exist, which is fine
      }

      // Create the index
      await this.makeOpenSearchRequest('/documents_v1', 'PUT', indexMapping);
      console.log('✅ OpenSearch index documents_v1 created successfully');

      // Create index template for future indices
      const template = {
        index_patterns: ['documents_*'],
        template: indexMapping
      };

      await this.makeOpenSearchRequest('/_index_template/documents_template', 'PUT', template);
      console.log('✅ OpenSearch index template created');

    } catch (error) {
      console.error('❌ Failed to create OpenSearch index:', error.message);
      throw error;
    }
  }

  async createQdrantCollection() {
    console.log('🔍 Creating Qdrant collection: docs_v1');

    const collectionConfig = {
      vectors: {
        size: 384, // all-MiniLM-L6-v2 embedding size
        distance: 'Cosine'
      },
      optimizers_config: {
        default_segment_number: 2
      },
      replication_factor: 1
    };

    try {
      // Check if collection exists
      try {
        await this.makeQdrantRequest('/collections/docs_v1');
        console.log('⚠️  Collection docs_v1 already exists, deleting...');
        await this.makeQdrantRequest('/collections/docs_v1', 'DELETE');
      } catch (error) {
        // Collection doesn't exist, which is fine
      }

      // Create the collection
      await this.makeQdrantRequest('/collections/docs_v1', 'PUT', collectionConfig);
      console.log('✅ Qdrant collection docs_v1 created successfully');

      // Create payload index for faster filtering
      const payloadIndexes = [
        { field_name: 'source', field_schema: 'keyword' },
        { field_name: 'court', field_schema: 'keyword' },
        { field_name: 'jurisdiction', field_schema: 'keyword' },
        { field_name: 'type', field_schema: 'keyword' },
        { field_name: 'date_filed', field_schema: 'datetime' },
        { field_name: 'docket_number', field_schema: 'keyword' }
      ];

      for (const index of payloadIndexes) {
        await this.makeQdrantRequest(
          `/collections/docs_v1/index`,
          'PUT',
          index
        );
        console.log(`✅ Created payload index for: ${index.field_name}`);
      }

    } catch (error) {
      console.error('❌ Failed to create Qdrant collection:', error.message);
      throw error;
    }
  }

  async verifyIndexes() {
    console.log('🔍 Verifying search indexes...');

    try {
      // Verify OpenSearch
      const osHealth = await this.makeOpenSearchRequest('/_cluster/health');
      console.log(`✅ OpenSearch cluster status: ${osHealth.status}`);

      const osIndex = await this.makeOpenSearchRequest('/documents_v1');
      console.log('✅ OpenSearch index documents_v1 verified');

      // Verify Qdrant
      const qdrantCollections = await this.makeQdrantRequest('/collections');
      const hasDocsCollection = qdrantCollections.result.collections.some(
        c => c.name === 'docs_v1'
      );
      
      if (hasDocsCollection) {
        console.log('✅ Qdrant collection docs_v1 verified');
      } else {
        throw new Error('Qdrant collection docs_v1 not found');
      }

      console.log('🎉 All search indexes verified successfully!');

    } catch (error) {
      console.error('❌ Index verification failed:', error.message);
      throw error;
    }
  }

  async run() {
    console.log('🚀 Starting search index creation...\n');

    try {
      if (this.opensearchUrl) {
        await this.createOpenSearchIndex();
      } else {
        console.log('⚠️  OPENSEARCH_URL not set, skipping OpenSearch setup');
      }

      if (this.qdrantUrl) {
        await this.createQdrantCollection();
      } else {
        console.log('⚠️  QDRANT_URL not set, skipping Qdrant setup');
      }

      await this.verifyIndexes();

      console.log('\n🎉 Search index creation completed successfully!');

    } catch (error) {
      console.error('\n❌ Search index creation failed:', error.message);
      process.exit(1);
    }
  }
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const manager = new SearchIndexManager();
  manager.run().catch(error => {
    console.error('Search index creation failed:', error);
    process.exit(1);
  });
}

export { SearchIndexManager };

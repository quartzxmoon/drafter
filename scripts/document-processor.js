#!/usr/bin/env node

/**
 * Document Processing Engine
 * Handles document generation, formatting, and export functionality
 */

import { config } from 'dotenv';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import PDFDocument from 'pdfkit';
import archiver from 'archiver';
import { Parser } from 'json2csv';
import crypto from 'crypto';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load environment variables
config();

class DocumentProcessor {
  constructor() {
    this.templatesDir = join(__dirname, '..', 'templates');
    this.outputDir = process.env.UPLOAD_DIR || './uploads';
    this.tempDir = process.env.TEMP_DIR || './temp';
    
    // Ensure directories exist
    mkdirSync(this.outputDir, { recursive: true });
    mkdirSync(this.tempDir, { recursive: true });
    
    // Court formatting rules
    this.courtRules = this.loadCourtRules();
  }

  loadCourtRules() {
    try {
      const rulesPath = join(__dirname, '..', 'config', 'courts.yaml');
      if (existsSync(rulesPath)) {
        // For now, return default rules - would parse YAML in production
        return this.getDefaultCourtRules();
      }
    } catch (error) {
      console.warn('Could not load court rules, using defaults');
    }
    
    return this.getDefaultCourtRules();
  }

  getDefaultCourtRules() {
    return {
      'default': {
        margins: { top: 72, bottom: 72, left: 72, right: 72 },
        font: { family: 'Times-Roman', size: 12 },
        lineSpacing: 1.5,
        pageNumbering: true,
        headerHeight: 36,
        footerHeight: 36
      },
      'pa-supreme': {
        margins: { top: 72, bottom: 72, left: 90, right: 72 },
        font: { family: 'Times-Roman', size: 12 },
        lineSpacing: 2.0,
        pageNumbering: true,
        headerHeight: 36,
        footerHeight: 36
      },
      'pa-superior': {
        margins: { top: 72, bottom: 72, left: 72, right: 72 },
        font: { family: 'Times-Roman', size: 12 },
        lineSpacing: 1.5,
        pageNumbering: true,
        headerHeight: 36,
        footerHeight: 36
      }
    };
  }

  /**
   * Generate a legal document from template
   */
  async generateDocument(templateName, data, options = {}) {
    const { court = 'default', format = 'pdf' } = options;
    
    try {
      // Load template
      const template = await this.loadTemplate(templateName);
      
      // Apply data to template
      const content = this.applyTemplateData(template, data);
      
      // Get court-specific formatting
      const formatting = this.courtRules[court] || this.courtRules['default'];
      
      // Generate document based on format
      switch (format) {
        case 'pdf':
          return await this.generatePDF(content, formatting, options);
        case 'docx':
          return await this.generateDOCX(content, formatting, options);
        case 'html':
          return await this.generateHTML(content, formatting, options);
        default:
          throw new Error(`Unsupported format: ${format}`);
      }
      
    } catch (error) {
      throw new Error(`Document generation failed: ${error.message}`);
    }
  }

  async loadTemplate(templateName) {
    const templatePath = join(this.templatesDir, `${templateName}.txt`);
    
    if (!existsSync(templatePath)) {
      throw new Error(`Template not found: ${templateName}`);
    }
    
    return readFileSync(templatePath, 'utf8');
  }

  applyTemplateData(template, data) {
    let content = template;
    
    // Replace template variables
    for (const [key, value] of Object.entries(data)) {
      const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g');
      content = content.replace(regex, value || '');
    }
    
    // Handle conditional blocks
    content = this.processConditionals(content, data);
    
    // Handle loops
    content = this.processLoops(content, data);
    
    return content;
  }

  processConditionals(content, data) {
    // Process {{#if condition}} blocks
    const ifRegex = /{{#if\s+(\w+)}}([\s\S]*?){{\/if}}/g;
    
    return content.replace(ifRegex, (match, condition, block) => {
      return data[condition] ? block : '';
    });
  }

  processLoops(content, data) {
    // Process {{#each array}} blocks
    const eachRegex = /{{#each\s+(\w+)}}([\s\S]*?){{\/each}}/g;
    
    return content.replace(eachRegex, (match, arrayName, block) => {
      const array = data[arrayName];
      if (!Array.isArray(array)) return '';
      
      return array.map(item => {
        let itemBlock = block;
        for (const [key, value] of Object.entries(item)) {
          const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g');
          itemBlock = itemBlock.replace(regex, value || '');
        }
        return itemBlock;
      }).join('');
    });
  }

  async generatePDF(content, formatting, options = {}) {
    const doc = new PDFDocument({
      size: 'LETTER',
      margins: formatting.margins,
      info: {
        Title: options.title || 'Legal Document',
        Author: options.author || 'PA eDocket Desktop',
        Subject: options.subject || '',
        Creator: 'PA eDocket Desktop'
      }
    });

    // Set font
    doc.font(formatting.font.family, formatting.font.size);
    
    // Add header if specified
    if (options.header) {
      this.addHeader(doc, options.header, formatting);
    }
    
    // Add content
    const lines = content.split('\n');
    let currentY = doc.y;
    
    for (const line of lines) {
      if (line.trim() === '') {
        currentY += formatting.font.size * formatting.lineSpacing;
        doc.y = currentY;
        continue;
      }
      
      // Check for page break
      if (currentY > doc.page.height - formatting.margins.bottom - 50) {
        doc.addPage();
        currentY = formatting.margins.top;
        
        // Add header to new page
        if (options.header) {
          this.addHeader(doc, options.header, formatting);
          currentY = doc.y;
        }
      }
      
      doc.text(line, {
        lineGap: formatting.font.size * (formatting.lineSpacing - 1)
      });
      
      currentY = doc.y;
    }
    
    // Add page numbers if enabled
    if (formatting.pageNumbering) {
      this.addPageNumbers(doc, formatting);
    }
    
    // Add footer if specified
    if (options.footer) {
      this.addFooter(doc, options.footer, formatting);
    }
    
    // Generate filename and save
    const filename = `${options.filename || 'document'}_${Date.now()}.pdf`;
    const filepath = join(this.outputDir, filename);
    
    return new Promise((resolve, reject) => {
      const stream = doc.pipe(require('fs').createWriteStream(filepath));
      
      stream.on('finish', () => {
        resolve({
          filename,
          filepath,
          size: require('fs').statSync(filepath).size,
          format: 'pdf'
        });
      });
      
      stream.on('error', reject);
      
      doc.end();
    });
  }

  addHeader(doc, headerText, formatting) {
    const currentY = doc.y;
    doc.y = formatting.margins.top / 2;
    doc.text(headerText, {
      align: 'center'
    });
    doc.y = currentY;
  }

  addFooter(doc, footerText, formatting) {
    const pages = doc.bufferedPageRange();
    
    for (let i = 0; i < pages.count; i++) {
      doc.switchToPage(i);
      doc.y = doc.page.height - formatting.margins.bottom / 2;
      doc.text(footerText, {
        align: 'center'
      });
    }
  }

  addPageNumbers(doc, formatting) {
    const pages = doc.bufferedPageRange();
    
    for (let i = 0; i < pages.count; i++) {
      doc.switchToPage(i);
      doc.y = doc.page.height - formatting.margins.bottom / 2;
      doc.text(`Page ${i + 1} of ${pages.count}`, {
        align: 'right'
      });
    }
  }

  async generateHTML(content, formatting, options = {}) {
    const html = `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>${options.title || 'Legal Document'}</title>
    <style>
        body {
            font-family: ${formatting.font.family}, serif;
            font-size: ${formatting.font.size}pt;
            line-height: ${formatting.lineSpacing};
            margin: ${formatting.margins.top}pt ${formatting.margins.right}pt ${formatting.margins.bottom}pt ${formatting.margins.left}pt;
        }
        .header {
            text-align: center;
            margin-bottom: 20pt;
        }
        .footer {
            text-align: center;
            margin-top: 20pt;
        }
        .page-break {
            page-break-before: always;
        }
    </style>
</head>
<body>
    ${options.header ? `<div class="header">${options.header}</div>` : ''}
    <div class="content">
        ${content.replace(/\n/g, '<br>')}
    </div>
    ${options.footer ? `<div class="footer">${options.footer}</div>` : ''}
</body>
</html>`;

    const filename = `${options.filename || 'document'}_${Date.now()}.html`;
    const filepath = join(this.outputDir, filename);
    
    writeFileSync(filepath, html, 'utf8');
    
    return {
      filename,
      filepath,
      size: Buffer.byteLength(html, 'utf8'),
      format: 'html'
    };
  }

  /**
   * Export documents in various formats
   */
  async exportDocuments(format, query, filters = {}) {
    try {
      // This would typically query the database
      // For now, return mock data structure
      const documents = await this.queryDocuments(query, filters);
      
      switch (format) {
        case 'json':
          return await this.exportJSON(documents);
        case 'csv':
          return await this.exportCSV(documents);
        case 'pdf':
          return await this.exportPDF(documents);
        case 'zip':
          return await this.exportZIP(documents);
        default:
          throw new Error(`Unsupported export format: ${format}`);
      }
      
    } catch (error) {
      throw new Error(`Export failed: ${error.message}`);
    }
  }

  async queryDocuments(query, filters) {
    // Mock implementation - would query actual database
    return [
      {
        id: 1,
        case_name: 'Sample v. Case',
        court: 'PA Supreme Court',
        date_filed: '2024-01-01',
        docket_number: 'No. 123-2024'
      }
    ];
  }

  async exportJSON(documents) {
    const data = {
      exported_at: new Date().toISOString(),
      count: documents.length,
      documents: documents
    };
    
    const filename = `export_${Date.now()}.json`;
    const filepath = join(this.outputDir, filename);
    
    writeFileSync(filepath, JSON.stringify(data, null, 2), 'utf8');
    
    return {
      filename,
      filepath,
      recordCount: documents.length,
      fileSize: require('fs').statSync(filepath).size
    };
  }

  async exportCSV(documents) {
    if (documents.length === 0) {
      throw new Error('No documents to export');
    }
    
    const fields = Object.keys(documents[0]);
    const parser = new Parser({ fields });
    const csv = parser.parse(documents);
    
    const filename = `export_${Date.now()}.csv`;
    const filepath = join(this.outputDir, filename);
    
    writeFileSync(filepath, csv, 'utf8');
    
    return {
      filename,
      filepath,
      recordCount: documents.length,
      fileSize: Buffer.byteLength(csv, 'utf8')
    };
  }

  async exportPDF(documents) {
    const doc = new PDFDocument();
    const filename = `export_${Date.now()}.pdf`;
    const filepath = join(this.outputDir, filename);
    
    const stream = doc.pipe(require('fs').createWriteStream(filepath));
    
    doc.fontSize(16).text('Document Export Report', { align: 'center' });
    doc.moveDown();
    
    doc.fontSize(12).text(`Generated: ${new Date().toLocaleString()}`);
    doc.text(`Total Documents: ${documents.length}`);
    doc.moveDown();
    
    documents.forEach((document, index) => {
      doc.text(`${index + 1}. ${document.case_name || 'Untitled'}`);
      doc.text(`   Court: ${document.court || 'Unknown'}`);
      doc.text(`   Date: ${document.date_filed || 'Unknown'}`);
      doc.text(`   Docket: ${document.docket_number || 'Unknown'}`);
      doc.moveDown();
    });
    
    return new Promise((resolve, reject) => {
      stream.on('finish', () => {
        resolve({
          filename,
          filepath,
          recordCount: documents.length,
          fileSize: require('fs').statSync(filepath).size
        });
      });
      
      stream.on('error', reject);
      
      doc.end();
    });
  }

  async exportZIP(documents) {
    const filename = `export_${Date.now()}.zip`;
    const filepath = join(this.outputDir, filename);
    
    const output = require('fs').createWriteStream(filepath);
    const archive = archiver('zip', { zlib: { level: 9 } });
    
    return new Promise((resolve, reject) => {
      output.on('close', () => {
        resolve({
          filename,
          filepath,
          recordCount: documents.length,
          fileSize: archive.pointer()
        });
      });
      
      archive.on('error', reject);
      
      archive.pipe(output);
      
      // Add manifest
      const manifest = {
        exported_at: new Date().toISOString(),
        count: documents.length,
        format: 'zip',
        contents: ['documents.json', 'documents.csv']
      };
      
      archive.append(JSON.stringify(manifest, null, 2), { name: 'manifest.json' });
      
      // Add JSON export
      archive.append(JSON.stringify(documents, null, 2), { name: 'documents.json' });
      
      // Add CSV export
      if (documents.length > 0) {
        const fields = Object.keys(documents[0]);
        const parser = new Parser({ fields });
        const csv = parser.parse(documents);
        archive.append(csv, { name: 'documents.csv' });
      }
      
      archive.finalize();
    });
  }

  /**
   * Generate document summary
   */
  async generateSummary(text) {
    if (!text || text.length < 100) {
      return '';
    }
    
    // Simple extractive summarization
    const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 20);
    
    // Take first 3 sentences as summary
    const summary = sentences.slice(0, 3).join('. ').trim();
    
    return summary.length > 0 ? summary + '.' : '';
  }

  /**
   * Validate document template
   */
  validateTemplate(templateContent) {
    const errors = [];
    const warnings = [];
    
    // Check for unmatched template variables
    const variables = templateContent.match(/{{[^}]+}}/g) || [];
    const unclosedBlocks = [];
    
    variables.forEach(variable => {
      if (variable.includes('#if') && !templateContent.includes('{{/if}}')) {
        unclosedBlocks.push('if block');
      }
      if (variable.includes('#each') && !templateContent.includes('{{/each}}')) {
        unclosedBlocks.push('each block');
      }
    });
    
    if (unclosedBlocks.length > 0) {
      errors.push(`Unclosed template blocks: ${unclosedBlocks.join(', ')}`);
    }
    
    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }
}

export { DocumentProcessor };

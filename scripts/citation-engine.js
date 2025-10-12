#!/usr/bin/env node

/**
 * Bluebook Citation Engine
 * Production-grade legal citation parsing, formatting, and validation
 */

import { config } from 'dotenv';

// Load environment variables
config();

class CitationEngine {
  constructor() {
    // Citation patterns for different types of legal documents
    this.patterns = {
      // Case citations
      case: [
        // U.S. Supreme Court: 123 U.S. 456 (1890)
        /(\d+)\s+U\.S\.\s+(\d+)(?:\s*,\s*(\d+))?\s*\((\d{4})\)/g,
        
        // Federal Courts: 123 F.2d 456 (9th Cir. 1990)
        /(\d+)\s+F\.(?:\s*2d|\s*3d|\s*4th)?\s+(\d+)(?:\s*,\s*(\d+))?\s*\(([^)]+)\s+(\d{4})\)/g,
        
        // State courts: 123 Cal. App. 4th 567 (2010)
        /(\d+)\s+([A-Z][a-z]+\.?\s*(?:App\.?\s*)?(?:\d+[a-z]*)?)\s+(\d+)(?:\s*,\s*(\d+))?\s*\((\d{4})\)/g,
        
        // Supreme Court Reporter: 123 S. Ct. 456 (2010)
        /(\d+)\s+S\.\s*Ct\.\s+(\d+)(?:\s*,\s*(\d+))?\s*\((\d{4})\)/g,
        
        // Lawyers' Edition: 123 L. Ed. 2d 456 (2010)
        /(\d+)\s+L\.\s*Ed\.?\s*(?:2d)?\s+(\d+)(?:\s*,\s*(\d+))?\s*\((\d{4})\)/g
      ],
      
      // Statute citations
      statute: [
        // U.S.C.: 42 U.S.C. § 1983
        /(\d+)\s+U\.S\.C\.?\s*§\s*(\d+(?:\([a-z0-9]+\))*)/g,
        
        // State statutes: Cal. Civ. Code § 1234
        /([A-Z][a-z]+\.?\s*(?:[A-Z][a-z]+\.?\s*)*Code)\s*§\s*(\d+(?:\.[a-z0-9]+)*)/g,
        
        // CFR: 29 C.F.R. § 1630.2
        /(\d+)\s+C\.F\.R\.?\s*§\s*(\d+(?:\.\d+)*)/g
      ],
      
      // Constitutional citations
      constitution: [
        // U.S. Constitution: U.S. Const. art. I, § 8
        /U\.S\.\s*Const\.?\s*(?:art\.?\s*([IVX]+))?(?:\s*,?\s*§\s*(\d+))?(?:\s*,?\s*cl\.?\s*(\d+))?/g,
        
        // Amendments: U.S. Const. amend. XIV, § 1
        /U\.S\.\s*Const\.?\s*amend\.?\s*([IVX]+)(?:\s*,?\s*§\s*(\d+))?/g
      ],
      
      // Rules citations
      rules: [
        // Federal Rules: Fed. R. Civ. P. 12(b)(6)
        /Fed\.?\s*R\.?\s*([A-Z][a-z]+\.?\s*P\.?)\s*(\d+(?:\([a-z0-9]+\))*)/g,
        
        // Local rules: E.D. Pa. L.R. 7.1
        /([A-Z]\.D\.\s*[A-Z][a-z]+\.?\s*L\.R\.?)\s*(\d+(?:\.\d+)*)/g
      ]
    };
    
    // Signal words for citation context
    this.signals = [
      'see', 'see also', 'cf.', 'compare', 'contra', 'but see', 'but cf.',
      'see generally', 'accord', 'e.g.', 'i.e.', 'viz.', 'cert. denied',
      'cert. granted', 'aff\'d', 'rev\'d', 'overruled by', 'superseded by'
    ];
    
    // Court abbreviations
    this.courts = {
      'U.S.': 'United States Supreme Court',
      '1st Cir.': 'United States Court of Appeals for the First Circuit',
      '2d Cir.': 'United States Court of Appeals for the Second Circuit',
      '3d Cir.': 'United States Court of Appeals for the Third Circuit',
      '4th Cir.': 'United States Court of Appeals for the Fourth Circuit',
      '5th Cir.': 'United States Court of Appeals for the Fifth Circuit',
      '6th Cir.': 'United States Court of Appeals for the Sixth Circuit',
      '7th Cir.': 'United States Court of Appeals for the Seventh Circuit',
      '8th Cir.': 'United States Court of Appeals for the Eighth Circuit',
      '9th Cir.': 'United States Court of Appeals for the Ninth Circuit',
      '10th Cir.': 'United States Court of Appeals for the Tenth Circuit',
      '11th Cir.': 'United States Court of Appeals for the Eleventh Circuit',
      'D.C. Cir.': 'United States Court of Appeals for the District of Columbia Circuit',
      'Fed. Cir.': 'United States Court of Appeals for the Federal Circuit'
    };
  }

  /**
   * Extract citations from text
   */
  async extractCitations(text) {
    if (!text || typeof text !== 'string') {
      return [];
    }

    const citations = [];
    const seen = new Set(); // Avoid duplicates

    // Extract each type of citation
    for (const [type, patterns] of Object.entries(this.patterns)) {
      for (const pattern of patterns) {
        let match;
        while ((match = pattern.exec(text)) !== null) {
          const citation = this.parseCitation(match, type);
          if (citation && !seen.has(citation.cite)) {
            citations.push(citation);
            seen.add(citation.cite);
          }
        }
      }
    }

    // Sort by position in text
    return citations.sort((a, b) => a.position - b.position);
  }

  /**
   * Parse a citation match into structured data
   */
  parseCitation(match, type) {
    const fullMatch = match[0];
    const position = match.index;

    switch (type) {
      case 'case':
        return this.parseCaseCitation(match, position);
      case 'statute':
        return this.parseStatuteCitation(match, position);
      case 'constitution':
        return this.parseConstitutionCitation(match, position);
      case 'rules':
        return this.parseRulesCitation(match, position);
      default:
        return null;
    }
  }

  parseCaseCitation(match, position) {
    const [fullMatch, volume, reporter, page, pinpoint, year] = match;
    
    return {
      cite: fullMatch.trim(),
      type: 'case',
      volume: parseInt(volume),
      reporter: this.normalizeReporter(reporter || match[2]),
      page: parseInt(page || match[3]),
      pinpoint: pinpoint ? parseInt(pinpoint) : null,
      year: parseInt(year || match[match.length - 1]),
      court: this.inferCourt(reporter || match[2]),
      position: position,
      isValid: this.validateCaseCitation(volume, reporter, page, year)
    };
  }

  parseStatuteCitation(match, position) {
    const [fullMatch, title, section] = match;
    
    return {
      cite: fullMatch.trim(),
      type: 'statute',
      title: title,
      section: section,
      position: position,
      isValid: this.validateStatuteCitation(title, section)
    };
  }

  parseConstitutionCitation(match, position) {
    const [fullMatch, article, section, clause] = match;
    
    return {
      cite: fullMatch.trim(),
      type: 'constitution',
      article: article || null,
      section: section ? parseInt(section) : null,
      clause: clause ? parseInt(clause) : null,
      position: position,
      isValid: true
    };
  }

  parseRulesCitation(match, position) {
    const [fullMatch, ruleType, rule] = match;
    
    return {
      cite: fullMatch.trim(),
      type: 'rules',
      ruleType: ruleType,
      rule: rule,
      position: position,
      isValid: this.validateRulesCitation(ruleType, rule)
    };
  }

  /**
   * Normalize reporter abbreviations
   */
  normalizeReporter(reporter) {
    const normalizations = {
      'U.S.': 'U.S.',
      'US': 'U.S.',
      'F.': 'F.',
      'F.2d': 'F.2d',
      'F.3d': 'F.3d',
      'F.4th': 'F.4th',
      'S.Ct.': 'S. Ct.',
      'S. Ct.': 'S. Ct.',
      'L.Ed.': 'L. Ed.',
      'L.Ed.2d': 'L. Ed. 2d'
    };
    
    return normalizations[reporter] || reporter;
  }

  /**
   * Infer court from reporter
   */
  inferCourt(reporter) {
    if (!reporter) return null;
    
    if (reporter.includes('U.S.') || reporter.includes('S. Ct.') || reporter.includes('L. Ed.')) {
      return 'U.S.';
    }
    
    if (reporter.includes('F.')) {
      return 'Federal';
    }
    
    return 'State';
  }

  /**
   * Validate case citation
   */
  validateCaseCitation(volume, reporter, page, year) {
    // Basic validation rules
    if (!volume || !reporter || !page || !year) return false;
    if (parseInt(volume) < 1 || parseInt(page) < 1) return false;
    if (parseInt(year) < 1789 || parseInt(year) > new Date().getFullYear()) return false;
    
    return true;
  }

  /**
   * Validate statute citation
   */
  validateStatuteCitation(title, section) {
    if (!title || !section) return false;
    return true;
  }

  /**
   * Validate rules citation
   */
  validateRulesCitation(ruleType, rule) {
    if (!ruleType || !rule) return false;
    return true;
  }

  /**
   * Format citation according to Bluebook rules
   */
  formatCitation(citation, options = {}) {
    const { shortForm = false, pinpoint = null, signal = null } = options;
    
    let formatted = '';
    
    // Add signal if provided
    if (signal) {
      formatted += `${signal} `;
    }
    
    switch (citation.type) {
      case 'case':
        formatted += this.formatCaseCitation(citation, shortForm, pinpoint);
        break;
      case 'statute':
        formatted += this.formatStatuteCitation(citation, shortForm);
        break;
      case 'constitution':
        formatted += this.formatConstitutionCitation(citation);
        break;
      case 'rules':
        formatted += this.formatRulesCitation(citation);
        break;
      default:
        formatted += citation.cite;
    }
    
    return formatted;
  }

  formatCaseCitation(citation, shortForm = false, pinpoint = null) {
    if (shortForm) {
      // Short form: Case Name, volume reporter page, pinpoint
      return `${citation.volume} ${citation.reporter} at ${pinpoint || citation.page}`;
    }
    
    // Full form: Case Name, volume reporter page (court year)
    let formatted = `${citation.volume} ${citation.reporter} ${citation.page}`;
    
    if (pinpoint || citation.pinpoint) {
      formatted += `, ${pinpoint || citation.pinpoint}`;
    }
    
    if (citation.court && citation.court !== 'U.S.') {
      formatted += ` (${citation.court} ${citation.year})`;
    } else {
      formatted += ` (${citation.year})`;
    }
    
    return formatted;
  }

  formatStatuteCitation(citation, shortForm = false) {
    return citation.cite;
  }

  formatConstitutionCitation(citation) {
    return citation.cite;
  }

  formatRulesCitation(citation) {
    return citation.cite;
  }

  /**
   * Generate citation suggestions for a document
   */
  async generateCitationSuggestions(document) {
    const suggestions = [];
    
    // Extract basic citation info from document metadata
    if (document.case_name && document.court && document.date_filed) {
      const year = new Date(document.date_filed).getFullYear();
      
      // Generate basic case citation format
      suggestions.push({
        type: 'case',
        format: 'basic',
        citation: `${document.case_name}, [volume] [reporter] [page] (${document.court} ${year})`,
        note: 'Complete with volume, reporter, and page information'
      });
      
      // Generate short form
      suggestions.push({
        type: 'case',
        format: 'short',
        citation: `${document.case_name}, [volume] [reporter] at [page]`,
        note: 'Short form for subsequent references'
      });
    }
    
    return suggestions;
  }

  /**
   * Validate citation format
   */
  validateCitationFormat(citation) {
    const errors = [];
    const warnings = [];
    
    if (!citation || typeof citation !== 'string') {
      errors.push('Citation is required');
      return { isValid: false, errors, warnings };
    }
    
    // Check for common formatting issues
    if (citation.includes('  ')) {
      warnings.push('Multiple spaces detected');
    }
    
    if (!citation.match(/\(\d{4}\)/)) {
      warnings.push('Year in parentheses not found');
    }
    
    // Check for proper punctuation
    if (!citation.includes('.')) {
      warnings.push('Missing periods in abbreviations');
    }
    
    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Convert citation to different formats
   */
  convertCitationFormat(citation, targetFormat) {
    switch (targetFormat) {
      case 'bluebook':
        return this.formatCitation(citation);
      case 'alwd':
        return this.formatCitationALWD(citation);
      case 'chicago':
        return this.formatCitationChicago(citation);
      default:
        return citation.cite;
    }
  }

  formatCitationALWD(citation) {
    // ALWD Citation Manual format (simplified)
    return this.formatCitation(citation);
  }

  formatCitationChicago(citation) {
    // Chicago Manual of Style format (simplified)
    return this.formatCitation(citation);
  }
}

export { CitationEngine };

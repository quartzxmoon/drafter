// Citation Parser - Bluebook citation parsing engine

import type { 
  ParsedCitation, 
  CitationAST, 
  CitationType, 
  CitationRule,
  ComponentType,
  Signal
} from './types';
import { CITATION_RULES } from './rules';

export class CitationParser {
  private rules: CitationRule[];
  
  constructor() {
    this.rules = CITATION_RULES;
  }
  
  /**
   * Parse citations from text
   */
  public parseText(text: string): ParsedCitation[] {
    const citations: ParsedCitation[] = [];
    
    // Find all potential citations using combined regex
    const citationPattern = this.buildCombinedPattern();
    const matches = Array.from(text.matchAll(citationPattern));
    
    for (const match of matches) {
      if (match.index !== undefined) {
        const citationText = match[0];
        const startIndex = match.index;
        const endIndex = startIndex + citationText.length;
        
        const parsed = this.parseSingle(citationText);
        if (parsed) {
          parsed.startIndex = startIndex;
          parsed.endIndex = endIndex;
          citations.push(parsed);
        }
      }
    }
    
    return this.postProcessCitations(citations);
  }
  
  /**
   * Parse a single citation string
   */
  public parseSingle(citationText: string): ParsedCitation | null {
    const trimmed = citationText.trim();
    
    // Try each citation type rule
    for (const rule of this.rules) {
      const match = trimmed.match(rule.pattern);
      if (match) {
        return this.buildCitationFromMatch(rule, match, trimmed);
      }
    }
    
    return null;
  }
  
  /**
   * Generate AST for a citation
   */
  public generateAST(citation: ParsedCitation): CitationAST {
    return {
      type: 'citation',
      citationType: citation.type,
      components: this.extractComponents(citation),
      raw: citation.fullCitation,
      position: {
        start: citation.startIndex || 0,
        end: citation.endIndex || citation.fullCitation.length
      }
    };
  }
  
  private buildCombinedPattern(): RegExp {
    // Combine all citation patterns with alternation
    const patterns = this.rules.map(rule => `(${rule.pattern.source})`);
    return new RegExp(patterns.join('|'), 'gi');
  }
  
  private buildCitationFromMatch(
    rule: CitationRule, 
    match: RegExpMatchArray, 
    fullText: string
  ): ParsedCitation {
    const citation: ParsedCitation = {
      type: rule.type,
      fullCitation: fullText,
      isValid: true,
      errors: [],
      suggestions: []
    };
    
    // Extract components based on rule
    for (let i = 0; i < rule.components.length; i++) {
      const component = rule.components[i];
      const value = match[component.position];
      
      if (value) {
        this.setComponentValue(citation, component.type, value.trim());
      } else if (component.required) {
        citation.errors.push(`Missing required component: ${component.type}`);
        citation.isValid = false;
      }
    }
    
    // Parse signal if present
    this.extractSignal(citation);
    
    // Parse parenthetical if present
    this.extractParenthetical(citation);
    
    // Parse pin cite if present
    this.extractPinCite(citation);
    
    return citation;
  }
  
  private setComponentValue(citation: ParsedCitation, type: ComponentType, value: string): void {
    switch (type) {
      case 'party_name':
        if (!citation.partyNames) citation.partyNames = [];
        citation.partyNames.push(value);
        if (!citation.title) citation.title = value;
        break;
      case 'volume':
        citation.volume = value;
        break;
      case 'reporter':
        citation.reporter = this.normalizeReporter(value);
        break;
      case 'page':
        citation.page = value;
        break;
      case 'court':
        citation.court = this.normalizeCourt(value);
        break;
      case 'year':
        citation.year = value;
        break;
      case 'title':
        citation.title = value;
        break;
      case 'code':
        citation.code = value;
        break;
      case 'section':
        citation.section = value;
        break;
      case 'rule_number':
        citation.ruleNumber = value;
        break;
      case 'author':
        citation.author = value;
        break;
      case 'publisher':
        citation.publisher = value;
        break;
      case 'edition':
        citation.edition = value;
        break;
    }
  }
  
  private extractSignal(citation: ParsedCitation): void {
    const signals: Signal[] = [
      'See', 'See also', 'Cf.', 'Compare', 'E.g.', 
      'Accord', 'See generally', 'But see', 'But cf.', 'Contra'
    ];
    
    for (const signal of signals) {
      if (citation.fullCitation.toLowerCase().startsWith(signal.toLowerCase())) {
        citation.signal = signal;
        break;
      }
    }
  }
  
  private extractParenthetical(citation: ParsedCitation): void {
    const parentheticalMatch = citation.fullCitation.match(/\(([^)]+)\)$/);
    if (parentheticalMatch) {
      const content = parentheticalMatch[1];
      
      // Check if it's a year (for cases) or actual parenthetical
      if (!/^\d{4}$/.test(content)) {
        citation.parenthetical = content;
      }
    }
  }
  
  private extractPinCite(citation: ParsedCitation): void {
    // Look for pin cites like "at 123" or ", 123"
    const pinCitePatterns = [
      /,\s*(\d+(?:-\d+)?)\s*$/,  // ", 123" or ", 123-125"
      /\s+at\s+(\d+(?:-\d+)?)\s*$/,  // " at 123"
      /\s+¶\s*(\d+(?:-\d+)?)\s*$/,   // " ¶ 123" (paragraph)
    ];
    
    for (const pattern of pinCitePatterns) {
      const match = citation.fullCitation.match(pattern);
      if (match) {
        citation.pinCite = match[1];
        break;
      }
    }
  }
  
  private normalizeReporter(reporter: string): string {
    // Common reporter abbreviations
    const reporterMap: Record<string, string> = {
      'F.': 'F.',
      'F.2d': 'F.2d',
      'F.3d': 'F.3d',
      'F.4th': 'F.4th',
      'F.Supp.': 'F. Supp.',
      'F.Supp.2d': 'F. Supp. 2d',
      'F.Supp.3d': 'F. Supp. 3d',
      'U.S.': 'U.S.',
      'S.Ct.': 'S. Ct.',
      'L.Ed.': 'L. Ed.',
      'L.Ed.2d': 'L. Ed. 2d',
      'A.': 'A.',
      'A.2d': 'A.2d',
      'A.3d': 'A.3d',
      'Pa.': 'Pa.',
      'Pa.Super.': 'Pa. Super.',
      'Pa.Commw.': 'Pa. Commw.',
    };
    
    return reporterMap[reporter] || reporter;
  }
  
  private normalizeCourt(court: string): string {
    // Common court abbreviations
    const courtMap: Record<string, string> = {
      'E.D. Pa.': 'E.D. Pa.',
      'M.D. Pa.': 'M.D. Pa.',
      'W.D. Pa.': 'W.D. Pa.',
      'Pa.': 'Pa.',
      'Pa. Super. Ct.': 'Pa. Super. Ct.',
      'Pa. Commw. Ct.': 'Pa. Commw. Ct.',
    };
    
    return courtMap[court] || court;
  }
  
  private extractComponents(citation: ParsedCitation): any[] {
    // TODO: Implement component extraction for AST
    return [];
  }
  
  private postProcessCitations(citations: ParsedCitation[]): ParsedCitation[] {
    // Post-process to handle short forms, supra, id., etc.
    const processed = [...citations];
    
    for (let i = 0; i < processed.length; i++) {
      const citation = processed[i];
      
      // Check for short forms
      if (this.isShortForm(citation.fullCitation)) {
        const fullCitation = this.findFullCitation(citation, processed.slice(0, i));
        if (fullCitation) {
          citation.shortForm = citation.fullCitation;
          citation.fullCitation = fullCitation.fullCitation;
          citation.type = fullCitation.type;
        }
      }
      
      // Handle "Id." citations
      if (this.isIdCitation(citation.fullCitation)) {
        const previousCitation = this.findPreviousCitation(processed, i);
        if (previousCitation) {
          citation.type = previousCitation.type;
          citation.title = previousCitation.title;
          citation.shortForm = citation.fullCitation;
          citation.fullCitation = previousCitation.fullCitation;
        }
      }
    }
    
    return processed;
  }
  
  private isShortForm(text: string): boolean {
    // Simple heuristics for short forms
    return text.length < 50 && 
           !text.includes(' v. ') && 
           !text.includes('§') &&
           /\d+/.test(text);
  }
  
  private isIdCitation(text: string): boolean {
    return /^Id\./i.test(text.trim());
  }
  
  private findFullCitation(shortCitation: ParsedCitation, previousCitations: ParsedCitation[]): ParsedCitation | null {
    // Look for matching full citation in previous citations
    for (let i = previousCitations.length - 1; i >= 0; i--) {
      const prev = previousCitations[i];
      if (prev.type === 'case' && prev.partyNames) {
        // Check if short form matches case name
        const firstParty = prev.partyNames[0];
        if (shortCitation.fullCitation.includes(firstParty)) {
          return prev;
        }
      }
    }
    return null;
  }
  
  private findPreviousCitation(citations: ParsedCitation[], currentIndex: number): ParsedCitation | null {
    for (let i = currentIndex - 1; i >= 0; i--) {
      const citation = citations[i];
      if (!this.isIdCitation(citation.fullCitation) && !this.isShortForm(citation.fullCitation)) {
        return citation;
      }
    }
    return null;
  }
}

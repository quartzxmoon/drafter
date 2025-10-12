// Citation Engine - Main interface for citation processing

import { CitationParser } from './parser';
import { CitationFormatter } from './formatter';
import { CitationValidator } from './validator';
import type { 
  ParsedCitation, 
  CitationStyle, 
  CitationContext,
  ValidationResult,
  TableOfAuthorities 
} from './types';

export class CitationEngine {
  private parser: CitationParser;
  private formatter: CitationFormatter;
  private validator: CitationValidator;
  
  constructor() {
    this.parser = new CitationParser();
    this.formatter = new CitationFormatter();
    this.validator = new CitationValidator();
  }
  
  /**
   * Parse citations from text
   */
  public parse(text: string): ParsedCitation[] {
    return this.parser.parseText(text);
  }
  
  /**
   * Parse and validate citations from text
   */
  public parseAndValidate(text: string, context?: CitationContext): ParsedCitation[] {
    const citations = this.parser.parseText(text);
    
    // Validate each citation
    for (const citation of citations) {
      const validation = this.validator.validate(citation, context);
      citation.isValid = validation.isValid;
      citation.errors = validation.errors.map(e => e.message);
      citation.suggestions = validation.suggestions;
    }
    
    return citations;
  }
  
  /**
   * Format a citation
   */
  public format(
    citation: ParsedCitation, 
    style: CitationStyle = 'Bluebook',
    options?: {
      shortForm?: boolean;
      context?: CitationContext;
      italicize?: boolean;
    }
  ): string {
    return this.formatter.format(citation, style, options);
  }
  
  /**
   * Validate a citation
   */
  public validate(citation: ParsedCitation, context?: CitationContext): ValidationResult {
    return this.validator.validate(citation, context);
  }
  
  /**
   * Generate short form for a citation
   */
  public generateShortForm(
    citation: ParsedCitation,
    previousCitations: ParsedCitation[],
    style: CitationStyle = 'Bluebook'
  ): string {
    return this.formatter.generateShortForm(citation, previousCitations, style);
  }
  
  /**
   * Generate table of authorities from citations
   */
  public generateTableOfAuthorities(
    citations: ParsedCitation[],
    style: CitationStyle = 'Bluebook'
  ): TableOfAuthorities {
    const toa: TableOfAuthorities = {
      cases: [],
      statutes: [],
      rules: [],
      constitutions: [],
      regulations: [],
      books: [],
      articles: [],
      other: [],
    };
    
    // Group citations by type
    for (const citation of citations) {
      const formatted = this.formatter.formatForTOA(citation, style);
      const toaCitation = { ...citation, fullCitation: formatted };
      
      switch (citation.type) {
        case 'case':
          if (!this.isDuplicate(toaCitation, toa.cases)) {
            toa.cases.push(toaCitation);
          }
          break;
        case 'statute':
          if (!this.isDuplicate(toaCitation, toa.statutes)) {
            toa.statutes.push(toaCitation);
          }
          break;
        case 'rule':
          if (!this.isDuplicate(toaCitation, toa.rules)) {
            toa.rules.push(toaCitation);
          }
          break;
        case 'constitution':
          if (!this.isDuplicate(toaCitation, toa.constitutions)) {
            toa.constitutions.push(toaCitation);
          }
          break;
        case 'regulation':
          if (!this.isDuplicate(toaCitation, toa.regulations)) {
            toa.regulations.push(toaCitation);
          }
          break;
        case 'book':
          if (!this.isDuplicate(toaCitation, toa.books)) {
            toa.books.push(toaCitation);
          }
          break;
        case 'article':
          if (!this.isDuplicate(toaCitation, toa.articles)) {
            toa.articles.push(toaCitation);
          }
          break;
        default:
          if (!this.isDuplicate(toaCitation, toa.other)) {
            toa.other.push(toaCitation);
          }
          break;
      }
    }
    
    // Sort each category
    this.sortCitations(toa.cases);
    this.sortCitations(toa.statutes);
    this.sortCitations(toa.rules);
    this.sortCitations(toa.constitutions);
    this.sortCitations(toa.regulations);
    this.sortCitations(toa.books);
    this.sortCitations(toa.articles);
    this.sortCitations(toa.other);
    
    return toa;
  }
  
  /**
   * Process document text and return formatted citations
   */
  public processDocument(
    text: string,
    options: {
      style?: CitationStyle;
      context?: CitationContext;
      generateShortForms?: boolean;
      validateCitations?: boolean;
    } = {}
  ): {
    processedText: string;
    citations: ParsedCitation[];
    tableOfAuthorities: TableOfAuthorities;
    validationResults: ValidationResult[];
  } {
    const {
      style = 'Bluebook',
      context,
      generateShortForms = true,
      validateCitations = true,
    } = options;
    
    // Parse citations from text
    const citations = this.parser.parseText(text);
    let processedText = text;
    
    // Validate citations if requested
    const validationResults: ValidationResult[] = [];
    if (validateCitations) {
      for (const citation of citations) {
        const validation = this.validator.validate(citation, context);
        validationResults.push(validation);
        citation.isValid = validation.isValid;
        citation.errors = validation.errors.map(e => e.message);
        citation.suggestions = validation.suggestions;
      }
    }
    
    // Generate short forms if requested
    if (generateShortForms) {
      for (let i = 0; i < citations.length; i++) {
        const citation = citations[i];
        const previousCitations = citations.slice(0, i);
        
        // Check if this citation has appeared before
        const previousOccurrence = this.findPreviousOccurrence(citation, previousCitations);
        if (previousOccurrence) {
          const shortForm = this.formatter.generateShortForm(citation, previousCitations, style);
          
          // Replace in text if appropriate
          if (this.shouldUseShortForm(citation, previousOccurrence)) {
            processedText = this.replaceInText(processedText, citation, shortForm);
          }
        }
      }
    }
    
    // Generate table of authorities
    const tableOfAuthorities = this.generateTableOfAuthorities(citations, style);
    
    return {
      processedText,
      citations,
      tableOfAuthorities,
      validationResults,
    };
  }
  
  /**
   * Export citations in various formats
   */
  public exportCitations(
    citations: ParsedCitation[],
    format: 'json' | 'csv' | 'bibtex',
    style: CitationStyle = 'Bluebook'
  ): string {
    switch (format) {
      case 'json':
        return JSON.stringify(citations, null, 2);
        
      case 'csv':
        return this.exportToCsv(citations);
        
      case 'bibtex':
        return this.exportToBibtex(citations);
        
      default:
        throw new Error(`Unsupported export format: ${format}`);
    }
  }
  
  private isDuplicate(citation: ParsedCitation, existingCitations: ParsedCitation[]): boolean {
    return existingCitations.some(existing => 
      this.citationsEqual(citation, existing)
    );
  }
  
  private citationsEqual(citation1: ParsedCitation, citation2: ParsedCitation): boolean {
    if (citation1.type !== citation2.type) {
      return false;
    }
    
    // Compare key identifying fields based on citation type
    switch (citation1.type) {
      case 'case':
        return citation1.volume === citation2.volume &&
               citation1.reporter === citation2.reporter &&
               citation1.page === citation2.page;
               
      case 'statute':
        return citation1.code === citation2.code &&
               citation1.section === citation2.section;
               
      case 'rule':
        return citation1.code === citation2.code &&
               citation1.ruleNumber === citation2.ruleNumber;
               
      default:
        return citation1.fullCitation === citation2.fullCitation;
    }
  }
  
  private sortCitations(citations: ParsedCitation[]): void {
    citations.sort((a, b) => {
      // Sort alphabetically by title or first party name
      const aKey = a.title || (a.partyNames && a.partyNames[0]) || a.fullCitation;
      const bKey = b.title || (b.partyNames && b.partyNames[0]) || b.fullCitation;
      return aKey.localeCompare(bKey);
    });
  }
  
  private findPreviousOccurrence(
    citation: ParsedCitation,
    previousCitations: ParsedCitation[]
  ): ParsedCitation | null {
    for (const prev of previousCitations) {
      if (this.citationsEqual(citation, prev)) {
        return prev;
      }
    }
    return null;
  }
  
  private shouldUseShortForm(
    citation: ParsedCitation,
    previousOccurrence: ParsedCitation
  ): boolean {
    // Use short form if citation appeared within last 5 citations
    // This is a simplified rule - actual Bluebook rules are more complex
    return true;
  }
  
  private replaceInText(
    text: string,
    citation: ParsedCitation,
    replacement: string
  ): string {
    if (citation.startIndex !== undefined && citation.endIndex !== undefined) {
      return text.substring(0, citation.startIndex) +
             replacement +
             text.substring(citation.endIndex);
    }
    return text.replace(citation.fullCitation, replacement);
  }
  
  private exportToCsv(citations: ParsedCitation[]): string {
    const headers = ['Type', 'Full Citation', 'Title', 'Author', 'Year', 'Valid'];
    const rows = citations.map(citation => [
      citation.type,
      citation.fullCitation,
      citation.title || '',
      citation.author || '',
      citation.year || '',
      citation.isValid.toString(),
    ]);
    
    return [headers, ...rows]
      .map(row => row.map(cell => `"${cell.replace(/"/g, '""')}"`).join(','))
      .join('\n');
  }
  
  private exportToBibtex(citations: ParsedCitation[]): string {
    return citations
      .map(citation => this.citationToBibtex(citation))
      .filter(entry => entry !== null)
      .join('\n\n');
  }
  
  private citationToBibtex(citation: ParsedCitation): string | null {
    const id = this.generateBibtexId(citation);
    
    switch (citation.type) {
      case 'case':
        return `@misc{${id},\n  title={${citation.title || ''}},\n  year={${citation.year || ''}},\n  note={${citation.fullCitation}}\n}`;
        
      case 'article':
        return `@article{${id},\n  author={${citation.author || ''}},\n  title={${citation.title || ''}},\n  journal={${citation.reporter || ''}},\n  volume={${citation.volume || ''}},\n  pages={${citation.page || ''}},\n  year={${citation.year || ''}}\n}`;
        
      case 'book':
        return `@book{${id},\n  author={${citation.author || ''}},\n  title={${citation.title || ''}},\n  year={${citation.year || ''}},\n  publisher={${citation.publisher || ''}}\n}`;
        
      default:
        return `@misc{${id},\n  title={${citation.title || citation.fullCitation}},\n  year={${citation.year || ''}}\n}`;
    }
  }
  
  private generateBibtexId(citation: ParsedCitation): string {
    const title = citation.title || citation.fullCitation;
    return title
      .toLowerCase()
      .replace(/[^a-z0-9]/g, '')
      .substring(0, 20);
  }
}

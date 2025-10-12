// Citation Formatter - Bluebook citation formatting engine

import type { 
  ParsedCitation, 
  CitationStyle, 
  FormattingRule,
  CitationContext 
} from './types';

export class CitationFormatter {
  private rules: Map<string, FormattingRule>;
  
  constructor() {
    this.rules = new Map();
    this.initializeRules();
  }
  
  /**
   * Format a citation according to specified style
   */
  public format(
    citation: ParsedCitation, 
    style: CitationStyle = 'Bluebook',
    options: {
      shortForm?: boolean;
      context?: CitationContext;
      italicize?: boolean;
    } = {}
  ): string {
    const rule = this.getFormattingRule(citation.type, style);
    if (!rule) {
      return citation.fullCitation;
    }
    
    if (options.shortForm && rule.shortFormTemplate) {
      return this.applyTemplate(citation, rule.shortFormTemplate, rule, options);
    }
    
    return this.applyTemplate(citation, rule.template, rule, options);
  }
  
  /**
   * Generate short form citation
   */
  public generateShortForm(
    citation: ParsedCitation,
    previousCitations: ParsedCitation[],
    style: CitationStyle = 'Bluebook'
  ): string {
    // Check if this citation has appeared before
    const previousOccurrence = this.findPreviousOccurrence(citation, previousCitations);
    
    if (previousOccurrence) {
      return this.generateSubsequentShortForm(citation, style);
    } else {
      return this.generateFirstShortForm(citation, style);
    }
  }
  
  /**
   * Format for table of authorities
   */
  public formatForTOA(citation: ParsedCitation, style: CitationStyle = 'Bluebook'): string {
    // TOA formatting typically uses full citations without signals
    const formatted = this.format(citation, style);
    
    // Remove signals for TOA
    return this.removeSignal(formatted);
  }
  
  /**
   * Apply italicization rules
   */
  public applyItalics(text: string, citation: ParsedCitation): string {
    switch (citation.type) {
      case 'case':
        // Italicize case names
        if (citation.partyNames && citation.partyNames.length >= 2) {
          const caseName = `${citation.partyNames[0]} v. ${citation.partyNames[1]}`;
          return text.replace(caseName, `<em>${caseName}</em>`);
        }
        break;
        
      case 'book':
        // Italicize book titles
        if (citation.title) {
          return text.replace(citation.title, `<em>${citation.title}</em>`);
        }
        break;
        
      case 'article':
        // Italicize article titles
        if (citation.title) {
          return text.replace(citation.title, `<em>${citation.title}</em>`);
        }
        break;
    }
    
    return text;
  }
  
  private initializeRules(): void {
    // Case formatting rules
    this.rules.set('case-Bluebook', {
      style: 'Bluebook',
      type: 'case',
      template: '{partyNames}, {volume} {reporter} {page}{pinCite} ({court} {year}){parenthetical}',
      shortFormTemplate: '{shortPartyName}, {volume} {reporter} at {pinCite}',
      italicize: ['party_name'],
    });
    
    // Statute formatting rules
    this.rules.set('statute-Bluebook', {
      style: 'Bluebook',
      type: 'statute',
      template: '{title} {code} § {section}{year}',
      shortFormTemplate: '§ {section}',
    });
    
    // Rule formatting rules
    this.rules.set('rule-Bluebook', {
      style: 'Bluebook',
      type: 'rule',
      template: '{code} {ruleNumber}',
    });
    
    // Constitution formatting rules
    this.rules.set('constitution-Bluebook', {
      style: 'Bluebook',
      type: 'constitution',
      template: '{title} {section}',
    });
    
    // Book formatting rules
    this.rules.set('book-Bluebook', {
      style: 'Bluebook',
      type: 'book',
      template: '{author}, {title}{edition}{year}',
      italicize: ['title'],
    });
    
    // Article formatting rules
    this.rules.set('article-Bluebook', {
      style: 'Bluebook',
      type: 'article',
      template: '{author}, {title}, {volume} {reporter} {page}{pinCite} ({year})',
      italicize: ['title'],
    });
  }
  
  private getFormattingRule(type: string, style: CitationStyle): FormattingRule | undefined {
    return this.rules.get(`${type}-${style}`);
  }
  
  private applyTemplate(
    citation: ParsedCitation, 
    template: string, 
    rule: FormattingRule,
    options: any
  ): string {
    let result = template;
    
    // Replace template variables
    result = result.replace(/{partyNames}/g, this.formatPartyNames(citation));
    result = result.replace(/{shortPartyName}/g, this.formatShortPartyName(citation));
    result = result.replace(/{volume}/g, citation.volume || '');
    result = result.replace(/{reporter}/g, citation.reporter || '');
    result = result.replace(/{page}/g, citation.page || '');
    result = result.replace(/{pinCite}/g, this.formatPinCite(citation));
    result = result.replace(/{court}/g, citation.court || '');
    result = result.replace(/{year}/g, this.formatYear(citation));
    result = result.replace(/{parenthetical}/g, this.formatParenthetical(citation));
    result = result.replace(/{title}/g, citation.title || '');
    result = result.replace(/{code}/g, citation.code || '');
    result = result.replace(/{section}/g, citation.section || '');
    result = result.replace(/{ruleNumber}/g, citation.ruleNumber || '');
    result = result.replace(/{author}/g, citation.author || '');
    result = result.replace(/{edition}/g, this.formatEdition(citation));
    
    // Clean up extra spaces and punctuation
    result = this.cleanupFormatting(result);
    
    // Apply italics if requested
    if (options.italicize !== false) {
      result = this.applyItalics(result, citation);
    }
    
    return result;
  }
  
  private formatPartyNames(citation: ParsedCitation): string {
    if (!citation.partyNames || citation.partyNames.length < 2) {
      return citation.title || '';
    }
    
    return `${citation.partyNames[0]} v. ${citation.partyNames[1]}`;
  }
  
  private formatShortPartyName(citation: ParsedCitation): string {
    if (!citation.partyNames || citation.partyNames.length === 0) {
      return citation.title || '';
    }
    
    // Use first party name for short form
    return citation.partyNames[0];
  }
  
  private formatPinCite(citation: ParsedCitation): string {
    if (!citation.pinCite) {
      return '';
    }
    
    // Add comma and space before pin cite for cases
    if (citation.type === 'case') {
      return `, ${citation.pinCite}`;
    }
    
    return citation.pinCite;
  }
  
  private formatYear(citation: ParsedCitation): string {
    if (!citation.year) {
      return '';
    }
    
    // For statutes, year goes in parentheses
    if (citation.type === 'statute') {
      return ` (${citation.year})`;
    }
    
    return citation.year;
  }
  
  private formatParenthetical(citation: ParsedCitation): string {
    if (!citation.parenthetical) {
      return '';
    }
    
    return ` (${citation.parenthetical})`;
  }
  
  private formatEdition(citation: ParsedCitation): string {
    if (!citation.edition) {
      return '';
    }
    
    return ` (${citation.edition})`;
  }
  
  private cleanupFormatting(text: string): string {
    // Remove extra spaces
    text = text.replace(/\s+/g, ' ');
    
    // Remove spaces before punctuation
    text = text.replace(/\s+([,.;:])/g, '$1');
    
    // Remove empty parentheses
    text = text.replace(/\(\s*\)/g, '');
    
    // Remove trailing spaces and punctuation
    text = text.trim().replace(/[,\s]+$/, '');
    
    return text;
  }
  
  private removeSignal(text: string): string {
    const signals = [
      'See ', 'See also ', 'Cf. ', 'Compare ', 'E.g., ',
      'Accord ', 'See generally ', 'But see ', 'But cf. ', 'Contra '
    ];
    
    for (const signal of signals) {
      if (text.startsWith(signal)) {
        return text.substring(signal.length);
      }
    }
    
    return text;
  }
  
  private findPreviousOccurrence(
    citation: ParsedCitation, 
    previousCitations: ParsedCitation[]
  ): ParsedCitation | null {
    for (const prev of previousCitations) {
      if (this.citationsMatch(citation, prev)) {
        return prev;
      }
    }
    return null;
  }
  
  private citationsMatch(citation1: ParsedCitation, citation2: ParsedCitation): boolean {
    if (citation1.type !== citation2.type) {
      return false;
    }
    
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
  
  private generateFirstShortForm(citation: ParsedCitation, style: CitationStyle): string {
    // First occurrence uses full citation
    return this.format(citation, style);
  }
  
  private generateSubsequentShortForm(citation: ParsedCitation, style: CitationStyle): string {
    // Subsequent occurrences use short form
    return this.format(citation, style, { shortForm: true });
  }
}

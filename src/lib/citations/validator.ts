// Citation Validator - Bluebook citation validation engine

import type { 
  ParsedCitation, 
  ValidationResult, 
  ValidationError, 
  ValidationWarning,
  CitationErrorCode,
  CitationContext 
} from './types';
import { CITATION_ERROR_CODES } from './types';
import { REPORTERS, COURTS } from './rules';

export class CitationValidator {
  
  /**
   * Validate a single citation
   */
  public validate(
    citation: ParsedCitation, 
    context?: CitationContext
  ): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const suggestions: string[] = [];
    
    // Basic format validation
    this.validateBasicFormat(citation, errors);
    
    // Type-specific validation
    switch (citation.type) {
      case 'case':
        this.validateCase(citation, errors, warnings, context);
        break;
      case 'statute':
        this.validateStatute(citation, errors, warnings);
        break;
      case 'rule':
        this.validateRule(citation, errors, warnings);
        break;
      case 'constitution':
        this.validateConstitution(citation, errors, warnings);
        break;
      case 'book':
        this.validateBook(citation, errors, warnings);
        break;
      case 'article':
        this.validateArticle(citation, errors, warnings);
        break;
    }
    
    // Generate suggestions based on errors
    this.generateSuggestions(citation, errors, suggestions);
    
    return {
      isValid: errors.filter(e => e.severity === 'error').length === 0,
      errors,
      warnings,
      suggestions,
    };
  }
  
  /**
   * Validate multiple citations for consistency
   */
  public validateMultiple(citations: ParsedCitation[]): ValidationResult[] {
    const results = citations.map(citation => this.validate(citation));
    
    // Check for consistency issues across citations
    this.validateConsistency(citations, results);
    
    return results;
  }
  
  /**
   * Validate citation format for specific court rules
   */
  public validateForCourt(
    citation: ParsedCitation, 
    courtRules: any
  ): ValidationResult {
    const result = this.validate(citation);
    
    // Apply court-specific validation rules
    this.applyCourtSpecificRules(citation, courtRules, result);
    
    return result;
  }
  
  private validateBasicFormat(citation: ParsedCitation, errors: ValidationError[]): void {
    if (!citation.fullCitation || citation.fullCitation.trim().length === 0) {
      errors.push({
        code: CITATION_ERROR_CODES.INVALID_FORMAT,
        message: 'Citation cannot be empty',
        severity: 'error',
      });
    }
    
    if (!citation.type) {
      errors.push({
        code: CITATION_ERROR_CODES.INVALID_FORMAT,
        message: 'Citation type must be specified',
        severity: 'error',
      });
    }
  }
  
  private validateCase(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[],
    context?: CitationContext
  ): void {
    // Required components for case citations
    if (!citation.partyNames || citation.partyNames.length < 2) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Case citation must include party names',
        component: 'party_name',
        severity: 'error',
        suggestion: 'Include both plaintiff and defendant names separated by "v."',
      });
    }
    
    if (!citation.volume) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Case citation must include volume number',
        component: 'volume',
        severity: 'error',
      });
    }
    
    if (!citation.reporter) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Case citation must include reporter',
        component: 'reporter',
        severity: 'error',
      });
    } else {
      this.validateReporter(citation.reporter, errors, warnings);
    }
    
    if (!citation.page) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Case citation must include page number',
        component: 'page',
        severity: 'error',
      });
    } else {
      this.validatePageNumber(citation.page, errors);
    }
    
    if (!citation.year) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Case citation must include year',
        component: 'year',
        severity: 'error',
      });
    } else {
      this.validateYear(citation.year, errors);
    }
    
    if (!citation.court) {
      // Court designation required for some reporters
      if (this.requiresCourtDesignation(citation.reporter)) {
        errors.push({
          code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
          message: 'Court designation required for this reporter',
          component: 'court',
          severity: 'error',
        });
      }
    } else {
      this.validateCourt(citation.court, errors, warnings);
    }
    
    // Validate pin cite format
    if (citation.pinCite) {
      this.validatePinCite(citation.pinCite, errors);
    }
    
    // Check for parallel citations if required
    if (context?.requireParallelCitations) {
      this.validateParallelCitations(citation, errors, warnings);
    }
  }
  
  private validateStatute(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!citation.code && !citation.title) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Statute citation must include code or title',
        component: 'code',
        severity: 'error',
      });
    }
    
    if (!citation.section) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Statute citation must include section number',
        component: 'section',
        severity: 'error',
      });
    }
    
    // Validate section format
    if (citation.section && !/^[0-9a-z.-]+$/i.test(citation.section)) {
      warnings.push({
        code: 'INVALID_SECTION_FORMAT',
        message: 'Section number format may be incorrect',
        component: 'section',
        suggestion: 'Use standard section numbering format (e.g., 1983, 12.1, 101(a))',
      });
    }
  }
  
  private validateRule(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!citation.code) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Rule citation must include rule code',
        component: 'code',
        severity: 'error',
      });
    }
    
    if (!citation.ruleNumber) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Rule citation must include rule number',
        component: 'rule_number',
        severity: 'error',
      });
    }
  }
  
  private validateConstitution(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!citation.title) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Constitution citation must include title',
        component: 'title',
        severity: 'error',
      });
    }
    
    if (!citation.section) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Constitution citation must include article or amendment',
        component: 'section',
        severity: 'error',
      });
    }
  }
  
  private validateBook(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!citation.author) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Book citation must include author',
        component: 'author',
        severity: 'error',
      });
    }
    
    if (!citation.title) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Book citation must include title',
        component: 'title',
        severity: 'error',
      });
    }
  }
  
  private validateArticle(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!citation.author) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Article citation must include author',
        component: 'author',
        severity: 'error',
      });
    }
    
    if (!citation.title) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Article citation must include title',
        component: 'title',
        severity: 'error',
      });
    }
    
    if (!citation.volume || !citation.reporter || !citation.page) {
      errors.push({
        code: CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT,
        message: 'Article citation must include volume, journal, and page',
        severity: 'error',
      });
    }
  }
  
  private validateReporter(
    reporter: string, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!REPORTERS[reporter as keyof typeof REPORTERS]) {
      warnings.push({
        code: CITATION_ERROR_CODES.INVALID_REPORTER,
        message: `Unknown reporter: ${reporter}`,
        component: 'reporter',
        suggestion: 'Verify reporter abbreviation is correct',
      });
    }
  }
  
  private validateCourt(
    court: string, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    if (!COURTS[court as keyof typeof COURTS]) {
      warnings.push({
        code: CITATION_ERROR_CODES.INVALID_COURT,
        message: `Unknown court: ${court}`,
        component: 'court',
        suggestion: 'Verify court abbreviation is correct',
      });
    }
  }
  
  private validateYear(year: string, errors: ValidationError[]): void {
    const yearNum = parseInt(year, 10);
    const currentYear = new Date().getFullYear();
    
    if (isNaN(yearNum) || yearNum < 1600 || yearNum > currentYear + 1) {
      errors.push({
        code: CITATION_ERROR_CODES.INVALID_YEAR,
        message: `Invalid year: ${year}`,
        component: 'year',
        severity: 'error',
        suggestion: 'Year should be a four-digit number',
      });
    }
  }
  
  private validatePageNumber(page: string, errors: ValidationError[]): void {
    if (!/^\d+$/.test(page)) {
      errors.push({
        code: CITATION_ERROR_CODES.INVALID_PAGE,
        message: `Invalid page number: ${page}`,
        component: 'page',
        severity: 'error',
        suggestion: 'Page number should contain only digits',
      });
    }
  }
  
  private validatePinCite(pinCite: string, errors: ValidationError[]): void {
    if (!/^\d+(?:-\d+)?$/.test(pinCite)) {
      errors.push({
        code: CITATION_ERROR_CODES.PIN_CITE_FORMATTING,
        message: `Invalid pin cite format: ${pinCite}`,
        component: 'pin_cite',
        severity: 'error',
        suggestion: 'Pin cite should be a page number or range (e.g., 123 or 123-125)',
      });
    }
  }
  
  private requiresCourtDesignation(reporter?: string): boolean {
    if (!reporter) return false;
    
    // Federal district court reporters require court designation
    return reporter.includes('F. Supp');
  }
  
  private validateParallelCitations(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    warnings: ValidationWarning[]
  ): void {
    // Check if parallel citation is required for this jurisdiction
    if (citation.reporter && REPORTERS[citation.reporter as keyof typeof REPORTERS]) {
      const reporterInfo = REPORTERS[citation.reporter as keyof typeof REPORTERS];
      if (!reporterInfo.isOfficial) {
        warnings.push({
          code: CITATION_ERROR_CODES.MISSING_PARALLEL_CITATION,
          message: 'Consider including parallel citation to official reporter',
          component: 'reporter',
          suggestion: 'Include citation to official reporter when available',
        });
      }
    }
  }
  
  private validateConsistency(
    citations: ParsedCitation[], 
    results: ValidationResult[]
  ): void {
    // Check for consistent citation style across document
    // This would be implemented based on specific consistency rules
  }
  
  private applyCourtSpecificRules(
    citation: ParsedCitation, 
    courtRules: any, 
    result: ValidationResult
  ): void {
    // Apply court-specific citation requirements
    // This would be implemented based on local court rules
  }
  
  private generateSuggestions(
    citation: ParsedCitation, 
    errors: ValidationError[], 
    suggestions: string[]
  ): void {
    // Generate helpful suggestions based on common errors
    if (errors.some(e => e.code === CITATION_ERROR_CODES.MISSING_REQUIRED_COMPONENT)) {
      suggestions.push('Ensure all required components are included for this citation type');
    }
    
    if (errors.some(e => e.code === CITATION_ERROR_CODES.INVALID_REPORTER)) {
      suggestions.push('Check reporter abbreviation against Bluebook Table T1');
    }
    
    if (errors.some(e => e.code === CITATION_ERROR_CODES.INVALID_COURT)) {
      suggestions.push('Verify court abbreviation against Bluebook Table T7');
    }
  }
}

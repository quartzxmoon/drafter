// Citation types and interfaces

export type CitationType = 
  | 'case' 
  | 'statute' 
  | 'rule' 
  | 'constitution' 
  | 'regulation' 
  | 'book' 
  | 'article';

export type CitationStyle = 'Bluebook' | 'ALWD';

export type Signal = 
  | 'See' 
  | 'See also' 
  | 'Cf.' 
  | 'Compare' 
  | 'E.g.' 
  | 'Accord' 
  | 'See generally' 
  | 'But see' 
  | 'But cf.' 
  | 'Contra';

export interface ParsedCitation {
  id?: string;
  type: CitationType;
  fullCitation: string;
  shortForm?: string;
  pinCite?: string;
  parenthetical?: string;
  signal?: Signal;
  
  // Parsed components
  title?: string;
  reporter?: string;
  volume?: string;
  page?: string;
  year?: string;
  court?: string;
  jurisdiction?: string;
  
  // Case-specific
  partyNames?: string[];
  
  // Statute-specific
  code?: string;
  section?: string;
  
  // Rule-specific
  ruleNumber?: string;
  
  // Book/Article-specific
  author?: string;
  publisher?: string;
  edition?: string;
  
  // Validation
  isValid: boolean;
  errors: string[];
  suggestions: string[];
  
  // Position in text
  startIndex?: number;
  endIndex?: number;
}

export interface CitationAST {
  type: 'citation';
  citationType: CitationType;
  components: CitationComponent[];
  raw: string;
  position: {
    start: number;
    end: number;
  };
}

export interface CitationComponent {
  type: ComponentType;
  value: string;
  position: {
    start: number;
    end: number;
  };
  isRequired: boolean;
  isValid: boolean;
  errors: string[];
}

export type ComponentType = 
  | 'party_name'
  | 'volume'
  | 'reporter'
  | 'page'
  | 'court'
  | 'year'
  | 'pin_cite'
  | 'parenthetical'
  | 'signal'
  | 'title'
  | 'code'
  | 'section'
  | 'rule_number'
  | 'author'
  | 'publisher'
  | 'edition';

export interface ReporterInfo {
  name: string;
  abbreviation: string;
  jurisdiction: string;
  startYear?: number;
  endYear?: number;
  series?: number;
  isOfficial: boolean;
  parallelReporters?: string[];
}

export interface CourtInfo {
  name: string;
  abbreviation: string;
  jurisdiction: string;
  level: 'trial' | 'appellate' | 'supreme';
  requiresCourtDesignation: boolean;
}

export interface CitationRule {
  type: CitationType;
  pattern: RegExp;
  components: ComponentRule[];
  examples: string[];
  notes?: string;
}

export interface ComponentRule {
  type: ComponentType;
  pattern: RegExp;
  required: boolean;
  position: number;
  validation?: (value: string) => boolean;
  normalization?: (value: string) => string;
}

export interface FormattingRule {
  style: CitationStyle;
  type: CitationType;
  template: string;
  shortFormTemplate?: string;
  italicize?: ComponentType[];
  capitalize?: ComponentType[];
  abbreviate?: Record<ComponentType, Record<string, string>>;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  suggestions: string[];
}

export interface ValidationError {
  code: string;
  message: string;
  component?: ComponentType;
  severity: 'error' | 'warning';
  suggestion?: string;
}

export interface ValidationWarning {
  code: string;
  message: string;
  component?: ComponentType;
  suggestion?: string;
}

export interface TableOfAuthorities {
  cases: ParsedCitation[];
  statutes: ParsedCitation[];
  rules: ParsedCitation[];
  constitutions: ParsedCitation[];
  regulations: ParsedCitation[];
  books: ParsedCitation[];
  articles: ParsedCitation[];
  other: ParsedCitation[];
}

export interface CitationContext {
  documentType: string;
  jurisdiction: string;
  court?: string;
  style: CitationStyle;
  previousCitations: ParsedCitation[];
  allowShortForms: boolean;
  requireParallelCitations: boolean;
}

// Error codes for citation validation
export const CITATION_ERROR_CODES = {
  INVALID_FORMAT: 'INVALID_FORMAT',
  MISSING_REQUIRED_COMPONENT: 'MISSING_REQUIRED_COMPONENT',
  INVALID_REPORTER: 'INVALID_REPORTER',
  INVALID_COURT: 'INVALID_COURT',
  INVALID_YEAR: 'INVALID_YEAR',
  INVALID_PAGE: 'INVALID_PAGE',
  INVALID_VOLUME: 'INVALID_VOLUME',
  INCONSISTENT_JURISDICTION: 'INCONSISTENT_JURISDICTION',
  MISSING_PARALLEL_CITATION: 'MISSING_PARALLEL_CITATION',
  IMPROPER_SHORT_FORM: 'IMPROPER_SHORT_FORM',
  SIGNAL_FORMATTING: 'SIGNAL_FORMATTING',
  PARENTHETICAL_FORMATTING: 'PARENTHETICAL_FORMATTING',
  PIN_CITE_FORMATTING: 'PIN_CITE_FORMATTING',
} as const;

export type CitationErrorCode = typeof CITATION_ERROR_CODES[keyof typeof CITATION_ERROR_CODES];

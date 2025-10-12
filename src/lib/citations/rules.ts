// Citation rules for Bluebook parsing

import type { CitationRule, ComponentRule } from './types';

// Case citation patterns
const CASE_PATTERNS = {
  // Standard case: Party v. Party, Volume Reporter Page (Court Year)
  STANDARD: /^(.+?)\s+v\.\s+(.+?),\s*(\d+)\s+([A-Za-z.]+(?:\s+\d+[a-z])?)\s+(\d+)(?:\s*,\s*(\d+(?:-\d+)?))?\s*\(([^)]+)\s+(\d{4})\)$/,
  
  // Case with pin cite: Party v. Party, Volume Reporter Page, PinCite (Court Year)
  WITH_PIN: /^(.+?)\s+v\.\s+(.+?),\s*(\d+)\s+([A-Za-z.]+(?:\s+\d+[a-z])?)\s+(\d+),\s*(\d+(?:-\d+)?)\s*\(([^)]+)\s+(\d{4})\)$/,
  
  // Short form: Party, Volume Reporter at Page
  SHORT_FORM: /^(.+?),\s*(\d+)\s+([A-Za-z.]+(?:\s+\d+[a-z])?)\s+at\s+(\d+(?:-\d+)?)$/,
  
  // Id. citation
  ID_CITATION: /^Id\.\s*(?:at\s+(\d+(?:-\d+)?))?$/i,
  
  // Supra citation: Party, supra note X, at Page
  SUPRA: /^(.+?),\s*supra\s+note\s+(\d+)(?:,\s*at\s+(\d+(?:-\d+)?))?$/i,
};

// Statute citation patterns
const STATUTE_PATTERNS = {
  // Federal statute: Title U.S.C. § Section (Year)
  USC: /^(\d+)\s+U\.S\.C\.\s*§\s*([0-9a-z.-]+)(?:\s*\(([^)]+)\))?$/,
  
  // Pennsylvania statute: Title Pa.C.S. § Section
  PA_STATUTE: /^(\d+)\s+Pa\.C\.S\.\s*§\s*([0-9a-z.-]+)$/,
  
  // Code of Federal Regulations: Title C.F.R. § Section (Year)
  CFR: /^(\d+)\s+C\.F\.R\.\s*§\s*([0-9a-z.-]+)(?:\s*\((\d{4})\))?$/,
};

// Rule citation patterns
const RULE_PATTERNS = {
  // Federal Rules: Fed. R. Civ. P. Number
  FED_RULES: /^Fed\.\s+R\.\s+(Civ\.\s+P\.|Crim\.\s+P\.|Evid\.|App\.\s+P\.)\s+(\d+(?:\([a-z0-9]+\))?)$/,
  
  // Pennsylvania Rules: Pa. R. Civ. P. Number
  PA_RULES: /^Pa\.\s+R\.\s+(Civ\.\s+P\.|Crim\.\s+P\.|Evid\.|App\.\s+P\.)\s+(\d+(?:\.[0-9a-z]+)?)$/,
  
  // Local Rules: Local Rule Number
  LOCAL_RULES: /^(?:Local\s+)?(?:Civil\s+)?Rule\s+([0-9.-]+)$/,
};

// Constitution citation patterns
const CONSTITUTION_PATTERNS = {
  // U.S. Constitution: U.S. Const. art. Article, § Section
  US_CONST: /^U\.S\.\s+Const\.\s+(?:art\.\s+([IVX]+)(?:,\s*§\s*(\d+))?|amend\.\s+([IVX]+))$/,
  
  // Pennsylvania Constitution: Pa. Const. art. Article, § Section
  PA_CONST: /^Pa\.\s+Const\.\s+art\.\s+([IVX]+)(?:,\s*§\s*(\d+))?$/,
};

// Book citation patterns
const BOOK_PATTERNS = {
  // Standard book: Author, Title (Edition Year)
  STANDARD: /^([^,]+),\s+(.+?)\s+(?:\((\d+(?:st|nd|rd|th)\s+ed\.\s+)?(\d{4})\))?$/,
  
  // Treatise with section: Author, Title § Section (Edition Year)
  WITH_SECTION: /^([^,]+),\s+(.+?)\s+§\s*([0-9a-z.-]+)(?:\s+\((\d+(?:st|nd|rd|th)\s+ed\.\s+)?(\d{4})\))?$/,
};

// Article citation patterns
const ARTICLE_PATTERNS = {
  // Law review: Author, Title, Volume Journal Page (Year)
  LAW_REVIEW: /^([^,]+),\s+(.+?),\s+(\d+)\s+([^0-9]+)\s+(\d+)(?:\s*,\s*(\d+(?:-\d+)?))?\s*\((\d{4})\)$/,
  
  // Newspaper: Author, Title, Newspaper, Date, at Page
  NEWSPAPER: /^([^,]+),\s+(.+?),\s+([^,]+),\s+([^,]+),\s+at\s+([A-Z]?\d+)$/,
};

export const CITATION_RULES: CitationRule[] = [
  // Case citations
  {
    type: 'case',
    pattern: CASE_PATTERNS.STANDARD,
    components: [
      { type: 'party_name', pattern: /(.+)/, required: true, position: 1 },
      { type: 'party_name', pattern: /(.+)/, required: true, position: 2 },
      { type: 'volume', pattern: /(\d+)/, required: true, position: 3 },
      { type: 'reporter', pattern: /([A-Za-z.]+(?:\s+\d+[a-z])?)/, required: true, position: 4 },
      { type: 'page', pattern: /(\d+)/, required: true, position: 5 },
      { type: 'pin_cite', pattern: /(\d+(?:-\d+)?)/, required: false, position: 6 },
      { type: 'court', pattern: /([^)]+)/, required: true, position: 7 },
      { type: 'year', pattern: /(\d{4})/, required: true, position: 8 },
    ],
    examples: [
      'Brown v. Board of Education, 347 U.S. 483 (1954)',
      'Miranda v. Arizona, 384 U.S. 436, 444 (1966)',
      'Commonwealth v. Smith, 123 Pa. 456 (Pa. 2020)',
    ],
  },
  
  {
    type: 'case',
    pattern: CASE_PATTERNS.SHORT_FORM,
    components: [
      { type: 'party_name', pattern: /(.+)/, required: true, position: 1 },
      { type: 'volume', pattern: /(\d+)/, required: true, position: 2 },
      { type: 'reporter', pattern: /([A-Za-z.]+(?:\s+\d+[a-z])?)/, required: true, position: 3 },
      { type: 'pin_cite', pattern: /(\d+(?:-\d+)?)/, required: true, position: 4 },
    ],
    examples: [
      'Brown, 347 U.S. at 483',
      'Miranda, 384 U.S. at 444',
    ],
  },
  
  // Statute citations
  {
    type: 'statute',
    pattern: STATUTE_PATTERNS.USC,
    components: [
      { type: 'title', pattern: /(\d+)/, required: true, position: 1 },
      { type: 'section', pattern: /([0-9a-z.-]+)/, required: true, position: 2 },
      { type: 'year', pattern: /([^)]+)/, required: false, position: 3 },
    ],
    examples: [
      '42 U.S.C. § 1983',
      '28 U.S.C. § 1331 (2018)',
    ],
  },
  
  {
    type: 'statute',
    pattern: STATUTE_PATTERNS.PA_STATUTE,
    components: [
      { type: 'title', pattern: /(\d+)/, required: true, position: 1 },
      { type: 'section', pattern: /([0-9a-z.-]+)/, required: true, position: 2 },
    ],
    examples: [
      '42 Pa.C.S. § 8301',
      '23 Pa.C.S. § 5301',
    ],
  },
  
  // Rule citations
  {
    type: 'rule',
    pattern: RULE_PATTERNS.FED_RULES,
    components: [
      { type: 'code', pattern: /(Fed\.\s+R\.\s+(?:Civ\.\s+P\.|Crim\.\s+P\.|Evid\.|App\.\s+P\.))/, required: true, position: 1 },
      { type: 'rule_number', pattern: /(\d+(?:\([a-z0-9]+\))?)/, required: true, position: 2 },
    ],
    examples: [
      'Fed. R. Civ. P. 12(b)(6)',
      'Fed. R. Evid. 401',
    ],
  },
  
  {
    type: 'rule',
    pattern: RULE_PATTERNS.PA_RULES,
    components: [
      { type: 'code', pattern: /(Pa\.\s+R\.\s+(?:Civ\.\s+P\.|Crim\.\s+P\.|Evid\.|App\.\s+P\.))/, required: true, position: 1 },
      { type: 'rule_number', pattern: /(\d+(?:\.[0-9a-z]+)?)/, required: true, position: 2 },
    ],
    examples: [
      'Pa. R. Civ. P. 1035.2',
      'Pa. R. Evid. 401',
    ],
  },
  
  // Constitution citations
  {
    type: 'constitution',
    pattern: CONSTITUTION_PATTERNS.US_CONST,
    components: [
      { type: 'title', pattern: /(U\.S\.\s+Const\.)/, required: true, position: 0 },
      { type: 'section', pattern: /(art\.\s+[IVX]+(?:,\s*§\s*\d+)?|amend\.\s+[IVX]+)/, required: true, position: 1 },
    ],
    examples: [
      'U.S. Const. art. I, § 8',
      'U.S. Const. amend. XIV',
    ],
  },
  
  // Book citations
  {
    type: 'book',
    pattern: BOOK_PATTERNS.STANDARD,
    components: [
      { type: 'author', pattern: /([^,]+)/, required: true, position: 1 },
      { type: 'title', pattern: /(.+?)/, required: true, position: 2 },
      { type: 'edition', pattern: /(\d+(?:st|nd|rd|th)\s+ed\.)/, required: false, position: 3 },
      { type: 'year', pattern: /(\d{4})/, required: false, position: 4 },
    ],
    examples: [
      'Wright & Miller, Federal Practice and Procedure (3d ed. 2020)',
      'Black, Black\'s Law Dictionary (11th ed. 2019)',
    ],
  },
  
  // Article citations
  {
    type: 'article',
    pattern: ARTICLE_PATTERNS.LAW_REVIEW,
    components: [
      { type: 'author', pattern: /([^,]+)/, required: true, position: 1 },
      { type: 'title', pattern: /(.+?)/, required: true, position: 2 },
      { type: 'volume', pattern: /(\d+)/, required: true, position: 3 },
      { type: 'reporter', pattern: /([^0-9]+)/, required: true, position: 4 },
      { type: 'page', pattern: /(\d+)/, required: true, position: 5 },
      { type: 'pin_cite', pattern: /(\d+(?:-\d+)?)/, required: false, position: 6 },
      { type: 'year', pattern: /(\d{4})/, required: true, position: 7 },
    ],
    examples: [
      'Tribe, The Constitutional Structure of American Federalism, 123 Harv. L. Rev. 1 (2010)',
    ],
  },
];

// Reporter abbreviations and information
export const REPORTERS = {
  'U.S.': { name: 'United States Reports', jurisdiction: 'federal', isOfficial: true },
  'S. Ct.': { name: 'Supreme Court Reporter', jurisdiction: 'federal', isOfficial: false },
  'L. Ed.': { name: 'Lawyers\' Edition', jurisdiction: 'federal', isOfficial: false },
  'L. Ed. 2d': { name: 'Lawyers\' Edition, Second Series', jurisdiction: 'federal', isOfficial: false },
  'F.': { name: 'Federal Reporter', jurisdiction: 'federal', isOfficial: true },
  'F.2d': { name: 'Federal Reporter, Second Series', jurisdiction: 'federal', isOfficial: true },
  'F.3d': { name: 'Federal Reporter, Third Series', jurisdiction: 'federal', isOfficial: true },
  'F.4th': { name: 'Federal Reporter, Fourth Series', jurisdiction: 'federal', isOfficial: true },
  'F. Supp.': { name: 'Federal Supplement', jurisdiction: 'federal', isOfficial: true },
  'F. Supp. 2d': { name: 'Federal Supplement, Second Series', jurisdiction: 'federal', isOfficial: true },
  'F. Supp. 3d': { name: 'Federal Supplement, Third Series', jurisdiction: 'federal', isOfficial: true },
  'Pa.': { name: 'Pennsylvania Reports', jurisdiction: 'pennsylvania', isOfficial: true },
  'Pa. Super.': { name: 'Pennsylvania Superior Court Reports', jurisdiction: 'pennsylvania', isOfficial: true },
  'Pa. Commw.': { name: 'Pennsylvania Commonwealth Court Reports', jurisdiction: 'pennsylvania', isOfficial: true },
  'A.': { name: 'Atlantic Reporter', jurisdiction: 'regional', isOfficial: false },
  'A.2d': { name: 'Atlantic Reporter, Second Series', jurisdiction: 'regional', isOfficial: false },
  'A.3d': { name: 'Atlantic Reporter, Third Series', jurisdiction: 'regional', isOfficial: false },
};

// Court abbreviations
export const COURTS = {
  'U.S.': { name: 'Supreme Court of the United States', level: 'supreme' },
  'E.D. Pa.': { name: 'United States District Court for the Eastern District of Pennsylvania', level: 'trial' },
  'M.D. Pa.': { name: 'United States District Court for the Middle District of Pennsylvania', level: 'trial' },
  'W.D. Pa.': { name: 'United States District Court for the Western District of Pennsylvania', level: 'trial' },
  '3d Cir.': { name: 'United States Court of Appeals for the Third Circuit', level: 'appellate' },
  'Pa.': { name: 'Supreme Court of Pennsylvania', level: 'supreme' },
  'Pa. Super. Ct.': { name: 'Superior Court of Pennsylvania', level: 'appellate' },
  'Pa. Commw. Ct.': { name: 'Commonwealth Court of Pennsylvania', level: 'appellate' },
} as const;

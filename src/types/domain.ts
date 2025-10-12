// Core domain types for PA eDocket Desktop
// Production-ready TypeScript interfaces matching Rust structs

export type CourtLevel = "MDJ" | "CP" | "APP";

export type PartyRole = 
  | "Plaintiff" 
  | "Defendant" 
  | "Appellant" 
  | "Appellee" 
  | "Petitioner" 
  | "Respondent"
  | "Intervenor"
  | "Third Party Defendant"
  | "Cross Defendant"
  | "Cross Plaintiff";

export type ChargeGrade = 
  | "F1" | "F2" | "F3" 
  | "M1" | "M2" | "M3" 
  | "S" | "V";

export type CaseStatus = 
  | "Active" 
  | "Closed" 
  | "Pending" 
  | "Disposed" 
  | "Appealed" 
  | "Transferred";

export type EventType = 
  | "Filing" 
  | "Hearing" 
  | "Order" 
  | "Motion" 
  | "Trial" 
  | "Sentencing" 
  | "Appeal" 
  | "Settlement" 
  | "Dismissal";

export type FinancialType = 
  | "Fine" 
  | "Cost" 
  | "Restitution" 
  | "Fee" 
  | "Bail" 
  | "Bond";

// Search parameters for court records
export interface SearchParams {
  term?: string;
  court?: CourtLevel;
  county?: string;
  from?: string; // ISO date string
  to?: string;   // ISO date string
  docket?: string;
  otn?: string;  // Originating Tracking Number
  sid?: string;  // State ID Number
  page?: number;
  limit?: number;
}

// Search result item
export interface SearchResult {
  [x: string]: any;
  [x: string]: any;
  [x: string]: boolean;
  [x: string]: ReactNode;
  caseType: any;
  parties: boolean;
  filedDate(filedDate: any): import("react").ReactNode;
  lastActivity: any;
  id: string;
  caption: string;
  court: CourtLevel;
  county: string;
  filed: string; // ISO date string
  status: CaseStatus;
  lastUpdated?: string; // ISO date string
  docketNumber?: string;
  otn?: string;
  sid?: string;
  judge?: string;
  courtroom?: string;
}

// Party information
export interface Party {
  id?: string;
  name: string;
  role: PartyRole;
  address?: string;
  city?: string;
  state?: string;
  zipCode?: string;
  phone?: string;
  email?: string;
  attorney?: string;
  attorneyId?: string;
  attorneyPhone?: string;
  attorneyEmail?: string;
  dateAdded?: string;
}

// Criminal charge information
export interface Charge {
  sequence: ReactNode;
  id?: string;
  statute: string;
  grade?: ChargeGrade;
  description: string;
  disposition?: string;
  dispositionDate?: string;
  sentence?: string;
  plea?: string;
  verdict?: string;
  counts?: number;
}

// Court event/proceeding
export interface Event {
  description: ReactNode;
  time: ReactNode;
  time: any;
  date: string | number | Date;
  id?: string;
  type: EventType;
  when: string; // ISO datetime string
  location?: string;
  courtroom?: string;
  judge?: string;
  notes?: string;
  result?: string;
  nextDate?: string;
}

// Court filing/document
export interface Filing {
  document_url: any;
  status: ReactNode;
  status: string;
  status: string;
  [x: string]: ReactNode;
  document_type: ReactNode;
  document_type: any;
  document_title: ReactNode;
  filed_date: string | number | Date;
  id?: string;
  date: string; // ISO date string
  title: string;
  by?: string; // Filed by (party/attorney)
  docUrl?: string;
  docType?: string;
  pages?: number;
  size?: number; // File size in bytes
  hash?: string; // SHA-256 hash
}

// Financial information
export interface Financial {
  [x: string]: any;
  payment_plan: any;
  balance_due: any;
  balance_due: number;
  amount_paid: any;
  amount_assessed: any;
  id?: string;
  type: FinancialType;
  amount: number;
  balance: number;
  description?: string;
  dueDate?: string;
  paidDate?: string;
  paidAmount?: number;
  paymentMethod?: string;
}

// Attachment/exhibit
export interface Attachment {
  [x: string]: ReactNode;
  [x: string]: number;
  description: ReactNode;
  [x: string]: ReactNode;
  id?: string;
  name: string;
  url: string;
  type?: string;
  size?: number;
  hash?: string;
  uploadDate?: string;
}

// Complete docket information
export interface Docket {
  source: ReactNode;
  id: string;
  caption: string;
  status: CaseStatus;
  court: CourtLevel;
  county: string;
  filed: string; // ISO date string
  docketNumber?: string;
  otn?: string;
  sid?: string;
  judge?: string;
  courtroom?: string;
  division?: string;
  
  // Related data
  parties: Party[];
  charges: Charge[];
  events: Event[];
  filings: Filing[];
  financials: Financial[];
  attachments?: Attachment[];
  
  // Metadata
  lastUpdated?: string;
  sourceUrl?: string;
  fetchedAt?: string;
  hash?: string;
}

// Document drafting job specification
export interface DraftJob {
  id?: string;
  courtId: string;
  templateId: string;
  dockets: string[]; // Docket IDs
  variables: Record<string, unknown>;
  output: "PDF" | "DOCX" | "BOTH";
  
  // Optional metadata
  title?: string;
  description?: string;
  createdAt?: string;
  status?: "pending" | "processing" | "completed" | "failed";
  resultPath?: string;
  errorMessage?: string;
}

// E-filing capability
export interface EFilingCapability {
  courtId: string;
  enabled: boolean;
  provider: string;
  documentTypes: string[];
  maxFileSize: number;
  allowedFormats: string[];
  requiresCoverSheet: boolean;
  supportsElectronicService: boolean;
  feeCalculation: boolean;
}

// E-filing session
export interface EFilingSession {
  id: string;
  courtId: string;
  provider: string;
  token: string;
  refreshToken?: string;
  expiresAt: string;
  userId?: string;
  permissions: string[];
}

// E-filing submission
export interface EFilingSubmission {
  id: string;
  sessionId: string;
  docketId?: string;
  documentType: string;
  files: string[]; // File paths
  metadata: Record<string, unknown>;
  status: "pending" | "submitted" | "accepted" | "rejected" | "error";
  submissionId?: string;
  receiptPath?: string;
  errorMessage?: string;
  submittedAt?: string;
  processedAt?: string;
}

// Export manifest for audit trail
export interface ExportManifest {
  id: string;
  type: "JSON" | "CSV" | "PDF" | "ZIP";
  source: "search" | "docket" | "draft";
  query?: SearchParams;
  docketId?: string;
  jobId?: string;
  
  // Files included
  files: Array<{
    name: string;
    path: string;
    size: number;
    hash: string;
    type: string;
  }>;
  
  // Metadata
  createdAt: string;
  sourceUrl?: string;
  totalSize: number;
  checksum: string;
  version: string;
}

// Watchlist item for monitoring case changes
export interface WatchlistItem {
  id: string;
  docketId: string;
  caption: string;
  court: CourtLevel;
  county: string;
  addedAt: string;
  lastChecked?: string;
  lastChanged?: string;
  notifyOnChange: boolean;
  checkInterval: number; // Minutes
}

// Citation for Bluebook engine
export interface Citation {
  id?: string;
  type: "case" | "statute" | "rule" | "constitution" | "regulation" | "book" | "article";
  fullCitation: string;
  shortForm?: string;
  pinCite?: string;
  parenthetical?: string;
  signal?: string;
  
  // Parsed components
  title?: string;
  reporter?: string;
  volume?: string;
  page?: string;
  year?: string;
  court?: string;
  jurisdiction?: string;
  
  // Validation
  isValid: boolean;
  errors: string[];
  suggestions: string[];
}

// Court formatting rules
export interface CourtRules {
  courtId: string;
  margins: {
    top: string;
    bottom: string;
    left: string;
    right: string;
  };
  font: {
    family: string;
    size: string;
    lineSpacing: string;
  };
  caption: {
    format: string;
    includeDocket: boolean;
    includeCourt: boolean;
    includeCounty: boolean;
    includeJudge: boolean;
    includeDivision?: boolean;
  };
  signature: {
    attorneyName: boolean;
    attorneyId: boolean;
    firmName: boolean;
    address: boolean;
    phone: boolean;
    email: boolean;
  };
  serviceCertificate: boolean;
  tableOfContents?: boolean;
  tableOfAuthorities?: boolean;
  pageLimits: Record<string, number>;
}

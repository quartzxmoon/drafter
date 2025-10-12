// API types for Tauri commands and responses
// Production-ready TypeScript interfaces for frontend-backend communication

import type {
  SearchParams,
  SearchResult,
  Docket,
  DraftJob,
  EFilingCapability,
  EFilingSession,
  EFilingSubmission,
  ExportManifest,
  WatchlistItem,
  Citation,
  CourtRules,
  Attachment
} from './domain';

// API Response wrapper
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  code?: string;
}

// Pagination wrapper
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  hasNext: boolean;
  hasPrev: boolean;
}

// Search API
export interface SearchRequest {
  params: SearchParams;
}

export interface SearchResponse extends PaginatedResponse<SearchResult> {}

// Docket API
export interface DocketRequest {
  id: string;
  forceRefresh?: boolean;
}

export interface DocketResponse {
  docket: Docket;
  cached: boolean;
  lastUpdated: string;
}

// Attachments API
export interface AttachmentsRequest {
  docketId: string;
}

export interface AttachmentsResponse {
  attachments: Attachment[];
}

// Export API
export type ExportType = "JSON" | "CSV" | "PDF" | "ZIP";

export interface ExportRequest {
  type: ExportType;
  payload: any; // SearchResult[], Docket, or DraftJob
  options?: {
    includeManifest?: boolean;
    compression?: boolean;
    password?: string;
  };
}

export interface ExportResponse {
  filePath: string;
  manifest?: ExportManifest;
  size: number;
  checksum: string;
}

// Document Drafting API
export interface DraftRequest {
  job: DraftJob;
  options?: {
    preview?: boolean;
    validate?: boolean;
  };
}

export interface DraftResponse {
  pdfPath?: string;
  docxPath?: string;
  manifestPath: string;
  validationErrors?: string[];
  warnings?: string[];
}

// E-filing API
export interface EFilingCapabilitiesRequest {
  courtId: string;
}

export interface EFilingCapabilitiesResponse {
  capabilities: EFilingCapability[];
}

export interface EFilingLoginRequest {
  courtId: string;
  provider: string;
  credentials: {
    username: string;
    password: string;
    mfaCode?: string;
  };
}

export interface EFilingLoginResponse {
  session: EFilingSession;
  requiresMfa: boolean;
  mfaMethod?: string;
}

export interface EFilingSubmitRequest {
  sessionId: string;
  docketId?: string;
  documentType: string;
  files: string[];
  metadata: Record<string, unknown>;
}

export interface EFilingSubmitResponse {
  submission: EFilingSubmission;
}

export interface EFilingStatusRequest {
  submissionId: string;
}

export interface EFilingStatusResponse {
  submission: EFilingSubmission;
  receipt?: {
    path: string;
    confirmationNumber: string;
    timestamp: string;
  };
}

// Watchlist API
export interface WatchAddRequest {
  docketId: string;
  notifyOnChange?: boolean;
  checkInterval?: number;
}

export interface WatchRemoveRequest {
  docketId: string;
}

export interface WatchListResponse {
  items: WatchlistItem[];
}

// Citation API
export interface CitationParseRequest {
  text: string;
  style?: "Bluebook" | "ALWD";
}

export interface CitationParseResponse {
  citations: Citation[];
  errors: string[];
}

export interface CitationFormatRequest {
  citation: Citation;
  style: "Bluebook" | "ALWD";
  shortForm?: boolean;
}

export interface CitationFormatResponse {
  formatted: string;
  shortForm?: string;
  errors: string[];
}

export interface CitationValidateRequest {
  citations: Citation[];
  courtRules?: CourtRules;
}

export interface CitationValidateResponse {
  results: Array<{
    citation: Citation;
    isValid: boolean;
    errors: string[];
    suggestions: string[];
  }>;
}

// Court Rules API
export interface CourtRulesRequest {
  courtId: string;
}

export interface CourtRulesResponse {
  rules: CourtRules;
}

// Configuration API
export interface ConfigUpdateRequest {
  section: string;
  key: string;
  value: any;
}

export interface ConfigResponse {
  success: boolean;
  message?: string;
}

// System API
export interface SystemInfoResponse {
  version: string;
  platform: string;
  arch: string;
  buildDate: string;
  gitCommit?: string;
}

export interface SystemHealthResponse {
  status: "healthy" | "degraded" | "unhealthy";
  checks: Array<{
    name: string;
    status: "pass" | "fail" | "warn";
    message?: string;
    duration?: number;
  }>;
  timestamp: string;
}

// Logging API
export interface LogEntry {
  timestamp: string;
  level: "error" | "warn" | "info" | "debug" | "trace";
  target: string;
  message: string;
  fields?: Record<string, any>;
}

export interface LogsRequest {
  level?: string;
  target?: string;
  since?: string;
  limit?: number;
}

export interface LogsResponse {
  entries: LogEntry[];
  total: number;
}

// Update API
export interface UpdateCheckResponse {
  available: boolean;
  version?: string;
  releaseNotes?: string;
  downloadUrl?: string;
  signature?: string;
}

export interface UpdateInstallRequest {
  version: string;
  downloadUrl: string;
  signature: string;
}

export interface UpdateInstallResponse {
  success: boolean;
  message?: string;
  restartRequired: boolean;
}

// Error types
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: string;
  requestId?: string;
}

// Provider-specific error codes
export const ERROR_CODES = {
  // General
  INVALID_REQUEST: 'INVALID_REQUEST',
  UNAUTHORIZED: 'UNAUTHORIZED',
  FORBIDDEN: 'FORBIDDEN',
  NOT_FOUND: 'NOT_FOUND',
  RATE_LIMITED: 'RATE_LIMITED',
  INTERNAL_ERROR: 'INTERNAL_ERROR',
  
  // Provider-specific
  UJS_PORTAL_UNAVAILABLE: 'UJS_PORTAL_UNAVAILABLE',
  UJS_PORTAL_INVALID_RESPONSE: 'UJS_PORTAL_INVALID_RESPONSE',
  PACFILE_AUTH_FAILED: 'PACFILE_AUTH_FAILED',
  PACFILE_MFA_REQUIRED: 'PACFILE_MFA_REQUIRED',
  PACFILE_SUBMISSION_FAILED: 'PACFILE_SUBMISSION_FAILED',
  COUNTY_EFILING_UNAVAILABLE: 'COUNTY_EFILING_UNAVAILABLE',
  CTRACK_API_ERROR: 'CTRACK_API_ERROR',
  
  // Document processing
  TEMPLATE_NOT_FOUND: 'TEMPLATE_NOT_FOUND',
  TEMPLATE_INVALID: 'TEMPLATE_INVALID',
  CITATION_PARSE_ERROR: 'CITATION_PARSE_ERROR',
  COURT_RULES_NOT_FOUND: 'COURT_RULES_NOT_FOUND',
  DOCUMENT_GENERATION_FAILED: 'DOCUMENT_GENERATION_FAILED',
  
  // Export
  EXPORT_FAILED: 'EXPORT_FAILED',
  FILE_NOT_FOUND: 'FILE_NOT_FOUND',
  INSUFFICIENT_DISK_SPACE: 'INSUFFICIENT_DISK_SPACE',
  
  // Security
  CREDENTIAL_STORAGE_FAILED: 'CREDENTIAL_STORAGE_FAILED',
  CREDENTIAL_RETRIEVAL_FAILED: 'CREDENTIAL_RETRIEVAL_FAILED',
  INVALID_CREDENTIALS: 'INVALID_CREDENTIALS',
} as const;

export type ErrorCode = typeof ERROR_CODES[keyof typeof ERROR_CODES];

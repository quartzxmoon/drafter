// Step components for the e-filing wizard

import React, { useState } from 'react';
import { 
  Building, 
  Shield, 
  Upload, 
  FileText, 
  CheckCircle, 
  XCircle, 
  Clock, 
  RefreshCw,
  Eye,
  Download,
  AlertTriangle,
  Key
} from 'lucide-react';

interface EFilingCapability {
  provider: string;
  court_id: string;
  court_name: string;
  supported_document_types: string[];
  authentication_required: boolean;
  max_file_size: number;
  supported_formats: string[];
  fees_required: boolean;
}

interface AuthenticationStatus {
  provider: string;
  authenticated: boolean;
  expires_at?: string;
  user_info?: {
    name: string;
    email: string;
    bar_number?: string;
  };
}

interface DocumentUpload {
  file_path: string;
  document_type: string;
  title: string;
  description?: string;
  confidential: boolean;
}

interface EFilingSubmission {
  id: string;
  provider: string;
  court_id: string;
  case_number?: string;
  documents: DocumentUpload[];
  status: 'pending' | 'submitted' | 'accepted' | 'rejected' | 'error';
  submission_id?: string;
  submitted_at?: string;
  response_message?: string;
  tracking_number?: string;
}

// Discovery Step
export const DiscoveryStep: React.FC<{
  capabilities: EFilingCapability[];
  loading: boolean;
  selectedCapability: EFilingCapability | null;
  onSelect: (capability: EFilingCapability) => void;
  onRefresh: () => void;
}> = ({ capabilities, loading, selectedCapability, onSelect, onRefresh }) => (
  <div className="p-6">
    <div className="flex items-center justify-between mb-4">
      <h2 className="text-xl font-semibold text-gray-900">Discover E-Filing Systems</h2>
      <button
        type="button"
        onClick={onRefresh}
        disabled={loading}
        className="flex items-center px-3 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
      >
        <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
        Refresh
      </button>
    </div>
    
    <p className="text-gray-600 mb-6">
      Select a court's e-filing system to submit documents electronically.
    </p>
    
    {loading ? (
      <div className="flex items-center justify-center h-32">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600">Discovering e-filing capabilities...</span>
      </div>
    ) : capabilities.length === 0 ? (
      <div className="text-center py-8">
        <Building className="h-12 w-12 text-gray-400 mx-auto mb-4" />
        <p className="text-gray-500">No e-filing systems found.</p>
      </div>
    ) : (
      <div className="space-y-4">
        {capabilities.map((capability) => (
          <div
            key={`${capability.provider}-${capability.court_id}`}
            onClick={() => onSelect(capability)}
            className={`border rounded-lg p-4 cursor-pointer transition-all ${
              selectedCapability?.court_id === capability.court_id
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <div className="flex items-start">
              <div className={`w-4 h-4 rounded-full border-2 mr-3 mt-1 ${
                selectedCapability?.court_id === capability.court_id
                  ? 'border-blue-500 bg-blue-500'
                  : 'border-gray-300'
              }`}>
                {selectedCapability?.court_id === capability.court_id && (
                  <div className="w-2 h-2 bg-white rounded-full mx-auto mt-0.5"></div>
                )}
              </div>
              
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <h3 className="font-medium text-gray-900">{capability.court_name}</h3>
                  <div className="flex items-center space-x-2">
                    {capability.authentication_required && (
                      <Shield className="h-4 w-4 text-yellow-500" title="Authentication Required" />
                    )}
                    {capability.fees_required && (
                      <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded">Fees</span>
                    )}
                  </div>
                </div>
                
                <p className="text-sm text-gray-500 mb-2">Provider: {capability.provider}</p>
                
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="font-medium text-gray-700">Supported Documents:</span>
                    <div className="text-gray-600">
                      {capability.supported_document_types.slice(0, 3).join(', ')}
                      {capability.supported_document_types.length > 3 && ` +${capability.supported_document_types.length - 3} more`}
                    </div>
                  </div>
                  
                  <div>
                    <span className="font-medium text-gray-700">Max File Size:</span>
                    <div className="text-gray-600">{formatFileSize(capability.max_file_size)}</div>
                  </div>
                  
                  <div>
                    <span className="font-medium text-gray-700">Formats:</span>
                    <div className="text-gray-600">{capability.supported_formats.join(', ')}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    )}
  </div>
);

// Authentication Step
export const AuthenticationStep: React.FC<{
  capability: EFilingCapability;
  authStatus: AuthenticationStatus | null;
  loading: boolean;
  onAuthenticate: (credentials: Record<string, string>) => void;
}> = ({ capability, authStatus, loading, onAuthenticate }) => {
  const [credentials, setCredentials] = useState<Record<string, string>>({
    username: '',
    password: '',
    mfa_code: ''
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onAuthenticate(credentials);
  };

  return (
    <div className="p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">Authentication Required</h2>
      <p className="text-gray-600 mb-6">
        Please authenticate with <strong>{capability.court_name}</strong> to proceed with e-filing.
      </p>
      
      {authStatus?.authenticated ? (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-center">
            <CheckCircle className="h-5 w-5 text-green-500 mr-2" />
            <div>
              <h3 className="text-green-800 font-medium">Successfully Authenticated</h3>
              {authStatus.user_info && (
                <div className="mt-2 text-sm text-green-700">
                  <p>Name: {authStatus.user_info.name}</p>
                  <p>Email: {authStatus.user_info.email}</p>
                  {authStatus.user_info.bar_number && (
                    <p>Bar Number: {authStatus.user_info.bar_number}</p>
                  )}
                </div>
              )}
              {authStatus.expires_at && (
                <p className="mt-2 text-sm text-green-600">
                  Session expires: {new Date(authStatus.expires_at).toLocaleString()}
                </p>
              )}
            </div>
          </div>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Username / Email
            </label>
            <input
              type="text"
              value={credentials.username}
              onChange={(e) => setCredentials(prev => ({ ...prev, username: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Password
            </label>
            <input
              type="password"
              value={credentials.password}
              onChange={(e) => setCredentials(prev => ({ ...prev, password: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              MFA Code (if required)
            </label>
            <input
              type="text"
              value={credentials.mfa_code}
              onChange={(e) => setCredentials(prev => ({ ...prev, mfa_code: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Enter 6-digit code"
            />
          </div>
          
          <button
            type="submit"
            disabled={loading}
            className="w-full flex items-center justify-center px-4 py-2 bg-blue-600 text-white font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                Authenticating...
              </>
            ) : (
              <>
                <Key className="h-4 w-4 mr-2" />
                Authenticate
              </>
            )}
          </button>
        </form>
      )}
    </div>
  );
};

// Documents Step
export const DocumentsStep: React.FC<{
  capability: EFilingCapability;
  documents: DocumentUpload[];
  caseNumber: string;
  onCaseNumberChange: (caseNumber: string) => void;
  onDocumentAdd: (document: DocumentUpload) => void;
  onDocumentRemove: (index: number) => void;
}> = ({ capability, documents, caseNumber, onCaseNumberChange, onDocumentAdd, onDocumentRemove }) => {
  const [showAddForm, setShowAddForm] = useState(false);
  const [newDocument, setNewDocument] = useState<Partial<DocumentUpload>>({
    document_type: '',
    title: '',
    description: '',
    confidential: false
  });

  const handleFileSelect = async () => {
    try {
      // In a real implementation, this would open a file dialog
      const filePath = '/path/to/selected/file.pdf'; // Placeholder
      setNewDocument(prev => ({ ...prev, file_path: filePath }));
    } catch (err) {
      console.error('File selection failed:', err);
    }
  };

  const handleAddDocument = () => {
    if (newDocument.file_path && newDocument.document_type && newDocument.title) {
      onDocumentAdd(newDocument as DocumentUpload);
      setNewDocument({
        document_type: '',
        title: '',
        description: '',
        confidential: false
      });
      setShowAddForm(false);
    }
  };

  return (
    <div className="p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">Upload Documents</h2>
      <p className="text-gray-600 mb-6">
        Add documents to file with <strong>{capability.court_name}</strong>.
      </p>

      {/* Case Number */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Case Number (optional)
        </label>
        <input
          type="text"
          value={caseNumber}
          onChange={(e) => onCaseNumberChange(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter case number if filing to existing case"
        />
      </div>

      {/* Document List */}
      <div className="space-y-4 mb-6">
        {documents.map((doc, index) => (
          <div key={index} className="border border-gray-200 rounded-lg p-4">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h3 className="font-medium text-gray-900">{doc.title}</h3>
                <p className="text-sm text-gray-500">{doc.document_type}</p>
                {doc.description && (
                  <p className="text-sm text-gray-600 mt-1">{doc.description}</p>
                )}
                <p className="text-xs text-gray-500 mt-2">{doc.file_path}</p>
                {doc.confidential && (
                  <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 mt-2">
                    Confidential
                  </span>
                )}
              </div>
              <button
                type="button"
                onClick={() => onDocumentRemove(index)}
                className="text-red-600 hover:text-red-800"
              >
                <XCircle className="h-5 w-5" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Add Document Form */}
      {showAddForm ? (
        <div className="border border-gray-200 rounded-lg p-4 mb-4">
          <h3 className="font-medium text-gray-900 mb-4">Add Document</h3>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Document Type
              </label>
              <select
                value={newDocument.document_type || ''}
                onChange={(e) => setNewDocument(prev => ({ ...prev, document_type: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">Select document type...</option>
                {capability.supported_document_types.map(type => (
                  <option key={type} value={type}>{type}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Document Title
              </label>
              <input
                type="text"
                value={newDocument.title || ''}
                onChange={(e) => setNewDocument(prev => ({ ...prev, title: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Enter document title"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Description (optional)
              </label>
              <textarea
                value={newDocument.description || ''}
                onChange={(e) => setNewDocument(prev => ({ ...prev, description: e.target.value }))}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Enter document description"
              />
            </div>

            <div>
              <button
                type="button"
                onClick={handleFileSelect}
                className="flex items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
              >
                <Upload className="h-4 w-4 mr-2" />
                Select File
              </button>
              {newDocument.file_path && (
                <p className="text-xs text-gray-500 mt-1">{newDocument.file_path}</p>
              )}
            </div>

            <div className="flex items-center">
              <input
                type="checkbox"
                id="confidential"
                checked={newDocument.confidential || false}
                onChange={(e) => setNewDocument(prev => ({ ...prev, confidential: e.target.checked }))}
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label htmlFor="confidential" className="ml-2 block text-sm text-gray-900">
                Mark as confidential
              </label>
            </div>
          </div>

          <div className="flex justify-end space-x-3 mt-4">
            <button
              type="button"
              onClick={() => setShowAddForm(false)}
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleAddDocument}
              disabled={!newDocument.file_path || !newDocument.document_type || !newDocument.title}
              className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
            >
              Add Document
            </button>
          </div>
        </div>
      ) : (
        <button
          type="button"
          onClick={() => setShowAddForm(true)}
          className="flex items-center px-4 py-2 border border-dashed border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
        >
          <Upload className="h-4 w-4 mr-2" />
          Add Document
        </button>
      )}
    </div>
  );
};

// Review Step
export const ReviewStep: React.FC<{
  capability: EFilingCapability;
  documents: DocumentUpload[];
  caseNumber: string;
  authStatus: AuthenticationStatus | null;
}> = ({ capability, documents, caseNumber, authStatus }) => (
  <div className="p-6">
    <h2 className="text-xl font-semibold text-gray-900 mb-4">Review E-Filing Submission</h2>
    <p className="text-gray-600 mb-6">
      Please review your submission details before filing.
    </p>

    <div className="space-y-6">
      {/* Court Information */}
      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="font-medium text-gray-900 mb-3">Court Information</h3>
        <dl className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <dt className="text-sm font-medium text-gray-500">Court</dt>
            <dd className="text-sm text-gray-900">{capability.court_name}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Provider</dt>
            <dd className="text-sm text-gray-900">{capability.provider}</dd>
          </div>
          {caseNumber && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Case Number</dt>
              <dd className="text-sm text-gray-900">{caseNumber}</dd>
            </div>
          )}
          {authStatus?.user_info && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Filing Attorney</dt>
              <dd className="text-sm text-gray-900">{authStatus.user_info.name}</dd>
            </div>
          )}
        </dl>
      </div>

      {/* Documents */}
      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="font-medium text-gray-900 mb-3">Documents ({documents.length})</h3>
        <div className="space-y-3">
          {documents.map((doc, index) => (
            <div key={index} className="bg-white rounded border p-3">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <h4 className="font-medium text-gray-900">{doc.title}</h4>
                  <p className="text-sm text-gray-500">{doc.document_type}</p>
                  {doc.description && (
                    <p className="text-sm text-gray-600 mt-1">{doc.description}</p>
                  )}
                  {doc.confidential && (
                    <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 mt-2">
                      Confidential
                    </span>
                  )}
                </div>
                <FileText className="h-5 w-5 text-gray-400" />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Fees Warning */}
      {capability.fees_required && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <div className="flex">
            <AlertTriangle className="h-5 w-5 text-yellow-400" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800">Filing Fees Required</h3>
              <p className="mt-1 text-sm text-yellow-700">
                This court requires filing fees. You will be prompted to pay during the submission process.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  </div>
);

// Status Step
export const StatusStep: React.FC<{
  submission: EFilingSubmission;
}> = ({ submission }) => {
  const getStatusIcon = () => {
    switch (submission.status) {
      case 'pending':
        return <Clock className="h-8 w-8 text-yellow-500" />;
      case 'submitted':
        return <Upload className="h-8 w-8 text-blue-500" />;
      case 'accepted':
        return <CheckCircle className="h-8 w-8 text-green-500" />;
      case 'rejected':
        return <XCircle className="h-8 w-8 text-red-500" />;
      case 'error':
        return <XCircle className="h-8 w-8 text-red-500" />;
      default:
        return <Clock className="h-8 w-8 text-gray-500" />;
    }
  };

  const getStatusMessage = () => {
    switch (submission.status) {
      case 'pending':
        return 'Your submission is being processed...';
      case 'submitted':
        return 'Your documents have been submitted successfully!';
      case 'accepted':
        return 'Your filing has been accepted by the court!';
      case 'rejected':
        return 'Your filing was rejected by the court.';
      case 'error':
        return 'An error occurred during submission.';
      default:
        return 'Unknown status';
    }
  };

  return (
    <div className="p-6">
      <div className="text-center mb-6">
        {getStatusIcon()}
        <h2 className="text-xl font-semibold text-gray-900 mt-4">
          {getStatusMessage()}
        </h2>
      </div>

      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="font-medium text-gray-900 mb-3">Submission Details</h3>
        <dl className="space-y-3">
          <div>
            <dt className="text-sm font-medium text-gray-500">Submission ID</dt>
            <dd className="text-sm text-gray-900">{submission.id}</dd>
          </div>

          {submission.submission_id && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Court Submission ID</dt>
              <dd className="text-sm text-gray-900">{submission.submission_id}</dd>
            </div>
          )}

          {submission.tracking_number && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Tracking Number</dt>
              <dd className="text-sm text-gray-900">{submission.tracking_number}</dd>
            </div>
          )}

          <div>
            <dt className="text-sm font-medium text-gray-500">Court</dt>
            <dd className="text-sm text-gray-900">{submission.court_id}</dd>
          </div>

          <div>
            <dt className="text-sm font-medium text-gray-500">Provider</dt>
            <dd className="text-sm text-gray-900">{submission.provider}</dd>
          </div>

          {submission.case_number && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Case Number</dt>
              <dd className="text-sm text-gray-900">{submission.case_number}</dd>
            </div>
          )}

          {submission.submitted_at && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Submitted At</dt>
              <dd className="text-sm text-gray-900">
                {new Date(submission.submitted_at).toLocaleString()}
              </dd>
            </div>
          )}

          <div>
            <dt className="text-sm font-medium text-gray-500">Documents</dt>
            <dd className="text-sm text-gray-900">{submission.documents.length} files</dd>
          </div>
        </dl>

        {submission.response_message && (
          <div className="mt-4 p-3 bg-white rounded border">
            <h4 className="text-sm font-medium text-gray-900 mb-2">Court Response</h4>
            <p className="text-sm text-gray-700">{submission.response_message}</p>
          </div>
        )}
      </div>
    </div>
  );
};

// Helper function
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

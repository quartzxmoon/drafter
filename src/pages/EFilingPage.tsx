// E-filing wizard for PA eDocket Desktop

import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Upload,
  ChevronLeft,
  ChevronRight,
  Check,
  AlertCircle,
  Shield,
  FileText,
  Clock,
  CheckCircle,
  XCircle,
  RefreshCw,
  Building,
  Key,
  Eye,
  Download
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import {
  DiscoveryStep,
  AuthenticationStep,
  DocumentsStep,
  ReviewStep,
  StatusStep
} from '../components/EFilingSteps';

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

type WizardStep = 'discovery' | 'authentication' | 'documents' | 'review' | 'submit' | 'status';

export const EFilingPage: React.FC = () => {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState<WizardStep>('discovery');
  const [capabilities, setCapabilities] = useState<EFilingCapability[]>([]);
  const [selectedCapability, setSelectedCapability] = useState<EFilingCapability | null>(null);
  const [authStatus, setAuthStatus] = useState<AuthenticationStatus | null>(null);
  const [documents, setDocuments] = useState<DocumentUpload[]>([]);
  const [caseNumber, setCaseNumber] = useState<string>('');
  const [submission, setSubmission] = useState<EFilingSubmission | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    discoverCapabilities();
  }, []);

  const discoverCapabilities = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<EFilingCapability[]>('cmd_discover_efiling_capabilities');
      setCapabilities(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to discover e-filing capabilities');
    } finally {
      setLoading(false);
    }
  };

  const handleCapabilitySelect = async (capability: EFilingCapability) => {
    setSelectedCapability(capability);

    if (capability.authentication_required) {
      // Check if already authenticated
      try {
        const authResult = await invoke<AuthenticationStatus>('cmd_check_efiling_auth', {
          provider: capability.provider
        });
        setAuthStatus(authResult);

        if (authResult.authenticated) {
          setCurrentStep('documents');
        } else {
          setCurrentStep('authentication');
        }
      } catch (err) {
        setCurrentStep('authentication');
      }
    } else {
      setCurrentStep('documents');
    }
  };

  const handleAuthentication = async (credentials: Record<string, string>) => {
    if (!selectedCapability) return;

    try {
      setLoading(true);
      setError(null);

      const authResult = await invoke<AuthenticationStatus>('cmd_authenticate_efiling', {
        provider: selectedCapability.provider,
        credentials
      });

      setAuthStatus(authResult);

      if (authResult.authenticated) {
        setCurrentStep('documents');
      } else {
        setError('Authentication failed. Please check your credentials.');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setLoading(false);
    }
  };

  const handleDocumentAdd = (document: DocumentUpload) => {
    setDocuments(prev => [...prev, document]);
  };

  const handleDocumentRemove = (index: number) => {
    setDocuments(prev => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    if (!selectedCapability || documents.length === 0) return;

    try {
      setLoading(true);
      setError(null);

      const submissionData = {
        provider: selectedCapability.provider,
        court_id: selectedCapability.court_id,
        case_number: caseNumber,
        documents
      };

      const result = await invoke<EFilingSubmission>('cmd_submit_efiling', submissionData);
      setSubmission(result);
      setCurrentStep('status');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit e-filing');
    } finally {
      setLoading(false);
    }
  };

  const handleNext = () => {
    if (currentStep === 'discovery' && selectedCapability) {
      handleCapabilitySelect(selectedCapability);
    } else if (currentStep === 'documents') {
      setCurrentStep('review');
    } else if (currentStep === 'review') {
      handleSubmit();
    }
  };

  const handleBack = () => {
    if (currentStep === 'authentication' || currentStep === 'documents') {
      setCurrentStep('discovery');
    } else if (currentStep === 'review') {
      setCurrentStep('documents');
    } else if (currentStep === 'status') {
      setCurrentStep('review');
    }
  };

  const steps = [
    { id: 'discovery', title: 'Discover Courts', description: 'Find available e-filing systems' },
    { id: 'authentication', title: 'Authenticate', description: 'Login to e-filing system' },
    { id: 'documents', title: 'Upload Documents', description: 'Select files to submit' },
    { id: 'review', title: 'Review', description: 'Confirm submission details' },
    { id: 'submit', title: 'Submit', description: 'File documents electronically' },
    { id: 'status', title: 'Status', description: 'Track submission status' },
  ];

  const currentStepIndex = steps.findIndex(step => step.id === currentStep);

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Progress Steps */}
      <div className="bg-white rounded-lg shadow p-6">
        <nav aria-label="Progress">
          <ol className="flex items-center">
            {steps.map((step, index) => (
              <li key={step.id} className={`relative ${index !== steps.length - 1 ? 'pr-8 sm:pr-20' : ''}`}>
                <div className="flex items-center">
                  <div className={`relative flex h-8 w-8 items-center justify-center rounded-full ${
                    index < currentStepIndex ? 'bg-green-600' :
                    index === currentStepIndex ? 'bg-blue-600' :
                    'bg-gray-300'
                  }`}>
                    {index < currentStepIndex ? (
                      <Check className="h-5 w-5 text-white" />
                    ) : (
                      <span className="text-white text-sm font-medium">{index + 1}</span>
                    )}
                  </div>
                  <span className="ml-4 text-sm font-medium text-gray-900">{step.title}</span>
                </div>
                {index !== steps.length - 1 && (
                  <div className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-gray-300" />
                )}
              </li>
            ))}
          </ol>
        </nav>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex">
            <AlertCircle className="h-5 w-5 text-red-400" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">Error</h3>
              <p className="mt-1 text-sm text-red-700">{error}</p>
            </div>
          </div>
        </div>
      )}

      {/* Step Content */}
      <div className="bg-white rounded-lg shadow">
        {currentStep === 'discovery' && (
          <DiscoveryStep
            capabilities={capabilities}
            loading={loading}
            selectedCapability={selectedCapability}
            onSelect={setSelectedCapability}
            onRefresh={discoverCapabilities}
          />
        )}

        {currentStep === 'authentication' && selectedCapability && (
          <AuthenticationStep
            capability={selectedCapability}
            authStatus={authStatus}
            loading={loading}
            onAuthenticate={handleAuthentication}
          />
        )}

        {currentStep === 'documents' && selectedCapability && (
          <DocumentsStep
            capability={selectedCapability}
            documents={documents}
            caseNumber={caseNumber}
            onCaseNumberChange={setCaseNumber}
            onDocumentAdd={handleDocumentAdd}
            onDocumentRemove={handleDocumentRemove}
          />
        )}

        {currentStep === 'review' && selectedCapability && (
          <ReviewStep
            capability={selectedCapability}
            documents={documents}
            caseNumber={caseNumber}
            authStatus={authStatus}
          />
        )}

        {currentStep === 'status' && submission && (
          <StatusStep submission={submission} />
        )}
      </div>

      {/* Navigation */}
      <div className="flex justify-between">
        <button
          type="button"
          onClick={handleBack}
          disabled={currentStep === 'discovery'}
          className={`flex items-center px-4 py-2 text-sm font-medium rounded-md ${
            currentStep === 'discovery'
              ? 'text-gray-400 cursor-not-allowed'
              : 'text-gray-700 bg-white border border-gray-300 hover:bg-gray-50'
          }`}
        >
          <ChevronLeft className="h-4 w-4 mr-2" />
          Back
        </button>

        {currentStep !== 'status' && currentStep !== 'authentication' && (
          <button
            type="button"
            onClick={handleNext}
            disabled={loading || !selectedCapability || (currentStep === 'documents' && documents.length === 0)}
            className="flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                Processing...
              </>
            ) : currentStep === 'review' ? (
              <>
                Submit E-Filing
                <Upload className="h-4 w-4 ml-2" />
              </>
            ) : (
              <>
                Next
                <ChevronRight className="h-4 w-4 ml-2" />
              </>
            )}
          </button>
        )}

        {currentStep === 'status' && (
          <button
            type="button"
            onClick={() => navigate('/search')}
            className="flex items-center px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700"
          >
            New E-Filing
            <Upload className="h-4 w-4 ml-2" />
          </button>
        )}
      </div>
    </div>
  );
};

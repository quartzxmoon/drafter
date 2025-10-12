// Document drafting wizard for PA eDocket Desktop

import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  FileText,
  ChevronLeft,
  ChevronRight,
  Check,
  AlertCircle,
  Download,
  Eye,
  Settings,
  Calendar,
  User,
  Building
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import {
  TemplateSelectionStep,
  VariablesStep,
  CourtSelectionStep,
  PreviewStep,
  ResultStep
} from '../components/DraftingSteps';

interface TemplateInfo {
  id: string;
  name: string;
  category: string;
  description: string;
  court_types: string[];
  document_type: string;
  variable_count: number;
}

interface TemplateVariable {
  name: string;
  var_type: string;
  required: boolean;
  description: string;
  options?: string[];
  default_value?: string;
}

interface DraftJob {
  template_id: string;
  document_type: string;
  court_id?: string;
  variables: Record<string, string>;
}

interface DraftResult {
  pdf_path?: string;
  docx_path?: string;
  manifest_path: string;
  validation_errors: string[];
  warnings: string[];
}

type WizardStep = 'template' | 'variables' | 'court' | 'preview' | 'result';

export const DraftingPage: React.FC = () => {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState<WizardStep>('template');
  const [templates, setTemplates] = useState<TemplateInfo[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<TemplateInfo | null>(null);
  const [templateVariables, setTemplateVariables] = useState<TemplateVariable[]>([]);
  const [variableValues, setVariableValues] = useState<Record<string, string>>({});
  const [selectedCourt, setSelectedCourt] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [draftResult, setDraftResult] = useState<DraftResult | null>(null);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      setLoading(true);
      const result = await invoke<TemplateInfo[]>('cmd_list_templates');
      setTemplates(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load templates');
    } finally {
      setLoading(false);
    }
  };

  const loadTemplateVariables = async (templateId: string) => {
    try {
      setLoading(true);
      const result = await invoke<TemplateVariable[]>('cmd_get_template_variables', { templateId });
      setTemplateVariables(result);

      // Initialize variable values with defaults
      const initialValues: Record<string, string> = {};
      result.forEach(variable => {
        if (variable.default_value) {
          initialValues[variable.name] = variable.default_value;
        }
      });
      setVariableValues(initialValues);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load template variables');
    } finally {
      setLoading(false);
    }
  };

  const handleTemplateSelect = async (template: TemplateInfo) => {
    setSelectedTemplate(template);
    await loadTemplateVariables(template.id);
    setCurrentStep('variables');
  };

  const handleVariableChange = (name: string, value: string) => {
    setVariableValues(prev => ({
      ...prev,
      [name]: value
    }));
  };

  const validateVariables = (): string[] => {
    const errors: string[] = [];
    templateVariables.forEach(variable => {
      if (variable.required) {
        const value = variableValues[variable.name];
        if (!value || value.trim() === '') {
          errors.push(`${variable.description} is required`);
        }
      }
    });
    return errors;
  };

  const handleNext = () => {
    if (currentStep === 'variables') {
      const errors = validateVariables();
      if (errors.length > 0) {
        setError(errors.join(', '));
        return;
      }
      setError(null);
      setCurrentStep('court');
    } else if (currentStep === 'court') {
      setCurrentStep('preview');
    } else if (currentStep === 'preview') {
      handleDraft();
    }
  };

  const handleBack = () => {
    if (currentStep === 'variables') {
      setCurrentStep('template');
    } else if (currentStep === 'court') {
      setCurrentStep('variables');
    } else if (currentStep === 'preview') {
      setCurrentStep('court');
    } else if (currentStep === 'result') {
      setCurrentStep('preview');
    }
  };

  const handleDraft = async () => {
    if (!selectedTemplate) return;

    try {
      setLoading(true);
      setError(null);

      const job: DraftJob = {
        template_id: selectedTemplate.id,
        document_type: selectedTemplate.document_type,
        court_id: selectedCourt || undefined,
        variables: variableValues
      };

      const result = await invoke<DraftResult>('cmd_draft_document', { job });

      if (result.validation_errors.length > 0) {
        setError(result.validation_errors.join(', '));
        return;
      }

      setDraftResult(result);
      setCurrentStep('result');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to draft document');
    } finally {
      setLoading(false);
    }
  };

  const steps = [
    { id: 'template', title: 'Select Template', description: 'Choose a document template' },
    { id: 'variables', title: 'Fill Variables', description: 'Enter document information' },
    { id: 'court', title: 'Court Rules', description: 'Select court formatting' },
    { id: 'preview', title: 'Preview', description: 'Review before drafting' },
    { id: 'result', title: 'Complete', description: 'Download your document' },
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
        {currentStep === 'template' && (
          <TemplateSelectionStep
            templates={templates}
            loading={loading}
            onSelect={handleTemplateSelect}
          />
        )}

        {currentStep === 'variables' && selectedTemplate && (
          <VariablesStep
            template={selectedTemplate}
            variables={templateVariables}
            values={variableValues}
            onChange={handleVariableChange}
          />
        )}

        {currentStep === 'court' && (
          <CourtSelectionStep
            selectedCourt={selectedCourt}
            onSelect={setSelectedCourt}
          />
        )}

        {currentStep === 'preview' && selectedTemplate && (
          <PreviewStep
            template={selectedTemplate}
            variables={variableValues}
            court={selectedCourt}
          />
        )}

        {currentStep === 'result' && draftResult && (
          <ResultStep result={draftResult} />
        )}
      </div>

      {/* Navigation */}
      <div className="flex justify-between">
        <button
          type="button"
          onClick={handleBack}
          disabled={currentStep === 'template'}
          className={`flex items-center px-4 py-2 text-sm font-medium rounded-md ${
            currentStep === 'template'
              ? 'text-gray-400 cursor-not-allowed'
              : 'text-gray-700 bg-white border border-gray-300 hover:bg-gray-50'
          }`}
        >
          <ChevronLeft className="h-4 w-4 mr-2" />
          Back
        </button>

        {currentStep !== 'result' && (
          <button
            type="button"
            onClick={handleNext}
            disabled={loading || !selectedTemplate || (currentStep === 'variables' && validateVariables().length > 0)}
            className="flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                Processing...
              </>
            ) : currentStep === 'preview' ? (
              <>
                Draft Document
                <FileText className="h-4 w-4 ml-2" />
              </>
            ) : (
              <>
                Next
                <ChevronRight className="h-4 w-4 ml-2" />
              </>
            )}
          </button>
        )}

        {currentStep === 'result' && (
          <button
            type="button"
            onClick={() => navigate('/search')}
            className="flex items-center px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700"
          >
            Start New Document
            <FileText className="h-4 w-4 ml-2" />
          </button>
        )}
      </div>
    </div>
  );
};

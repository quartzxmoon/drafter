// Step components for the drafting wizard

import React from 'react';
import { FileText, Building, Calendar, Download, Eye, AlertTriangle } from 'lucide-react';

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

interface DraftResult {
  pdf_path?: string;
  docx_path?: string;
  manifest_path: string;
  validation_errors: string[];
  warnings: string[];
}

// Template Selection Step
export const TemplateSelectionStep: React.FC<{
  templates: TemplateInfo[];
  loading: boolean;
  onSelect: (template: TemplateInfo) => void;
}> = ({ templates, loading, onSelect }) => (
  <div className="p-6">
    <h2 className="text-xl font-semibold text-gray-900 mb-4">Select Document Template</h2>
    <p className="text-gray-600 mb-6">Choose a template to start drafting your document.</p>
    
    {loading ? (
      <div className="flex items-center justify-center h-32">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600">Loading templates...</span>
      </div>
    ) : (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {templates.map((template) => (
          <div
            key={template.id}
            onClick={() => onSelect(template)}
            className="border border-gray-200 rounded-lg p-4 hover:border-blue-500 hover:shadow-md cursor-pointer transition-all"
          >
            <div className="flex items-start">
              <FileText className="h-6 w-6 text-blue-600 mt-1" />
              <div className="ml-3 flex-1">
                <h3 className="text-lg font-medium text-gray-900">{template.name}</h3>
                <p className="text-sm text-gray-500 mb-2">{template.category}</p>
                <p className="text-sm text-gray-700 mb-3">{template.description}</p>
                
                <div className="flex items-center justify-between text-xs text-gray-500">
                  <span>{template.variable_count} variables</span>
                  <span className="bg-gray-100 px-2 py-1 rounded">{template.document_type}</span>
                </div>
                
                {template.court_types.length > 0 && (
                  <div className="mt-2">
                    <span className="text-xs text-gray-500">Courts: </span>
                    {template.court_types.map((court, index) => (
                      <span key={court} className="text-xs text-blue-600">
                        {court.toUpperCase()}{index < template.court_types.length - 1 ? ', ' : ''}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    )}
  </div>
);

// Variables Input Step
export const VariablesStep: React.FC<{
  template: TemplateInfo;
  variables: TemplateVariable[];
  values: Record<string, string>;
  onChange: (name: string, value: string) => void;
}> = ({ template, variables, values, onChange }) => (
  <div className="p-6">
    <h2 className="text-xl font-semibold text-gray-900 mb-4">Fill Document Variables</h2>
    <p className="text-gray-600 mb-6">
      Enter the information for <strong>{template.name}</strong>. Required fields are marked with *.
    </p>
    
    <div className="space-y-6">
      {variables.map((variable) => (
        <div key={variable.name}>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            {variable.description}
            {variable.required && <span className="text-red-500 ml-1">*</span>}
          </label>
          
          {variable.options ? (
            <select
              value={values[variable.name] || ''}
              onChange={(e) => onChange(variable.name, e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="">Select an option...</option>
              {variable.options.map((option) => (
                <option key={option} value={option}>{option}</option>
              ))}
            </select>
          ) : variable.var_type === 'textarea' ? (
            <textarea
              value={values[variable.name] || ''}
              onChange={(e) => onChange(variable.name, e.target.value)}
              rows={4}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder={variable.default_value || `Enter ${variable.description.toLowerCase()}`}
            />
          ) : (
            <input
              type={variable.var_type === 'email' ? 'email' : variable.var_type === 'phone' ? 'tel' : 'text'}
              value={values[variable.name] || ''}
              onChange={(e) => onChange(variable.name, e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder={variable.default_value || `Enter ${variable.description.toLowerCase()}`}
            />
          )}
          
          {variable.required && !values[variable.name] && (
            <p className="mt-1 text-sm text-red-600">This field is required</p>
          )}
        </div>
      ))}
    </div>
  </div>
);

// Court Selection Step
export const CourtSelectionStep: React.FC<{
  selectedCourt: string;
  onSelect: (court: string) => void;
}> = ({ selectedCourt, onSelect }) => {
  const courts = [
    { id: '', name: 'No specific court (generic formatting)', type: 'Generic' },
    { id: 'philadelphia-cp', name: 'Philadelphia County Court of Common Pleas', type: 'Common Pleas' },
    { id: 'allegheny-cp', name: 'Allegheny County Court of Common Pleas', type: 'Common Pleas' },
    { id: 'montgomery-cp', name: 'Montgomery County Court of Common Pleas', type: 'Common Pleas' },
    { id: 'bucks-cp', name: 'Bucks County Court of Common Pleas', type: 'Common Pleas' },
    { id: 'chester-cp', name: 'Chester County Court of Common Pleas', type: 'Common Pleas' },
    { id: 'delaware-cp', name: 'Delaware County Court of Common Pleas', type: 'Common Pleas' },
  ];

  return (
    <div className="p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">Select Court Rules</h2>
      <p className="text-gray-600 mb-6">
        Choose the court to apply specific formatting rules, or use generic formatting.
      </p>
      
      <div className="space-y-3">
        {courts.map((court) => (
          <div
            key={court.id}
            onClick={() => onSelect(court.id)}
            className={`border rounded-lg p-4 cursor-pointer transition-all ${
              selectedCourt === court.id
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <div className="flex items-center">
              <div className={`w-4 h-4 rounded-full border-2 mr-3 ${
                selectedCourt === court.id
                  ? 'border-blue-500 bg-blue-500'
                  : 'border-gray-300'
              }`}>
                {selectedCourt === court.id && (
                  <div className="w-2 h-2 bg-white rounded-full mx-auto mt-0.5"></div>
                )}
              </div>
              <div className="flex-1">
                <h3 className="font-medium text-gray-900">{court.name}</h3>
                <p className="text-sm text-gray-500">{court.type}</p>
              </div>
              <Building className="h-5 w-5 text-gray-400" />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

// Preview Step
export const PreviewStep: React.FC<{
  template: TemplateInfo;
  variables: Record<string, string>;
  court: string;
}> = ({ template, variables, court }) => (
  <div className="p-6">
    <h2 className="text-xl font-semibold text-gray-900 mb-4">Review Document Details</h2>
    <p className="text-gray-600 mb-6">
      Please review the information below before generating your document.
    </p>
    
    <div className="space-y-6">
      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="font-medium text-gray-900 mb-3">Template Information</h3>
        <dl className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <dt className="text-sm font-medium text-gray-500">Template</dt>
            <dd className="text-sm text-gray-900">{template.name}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Document Type</dt>
            <dd className="text-sm text-gray-900">{template.document_type}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Category</dt>
            <dd className="text-sm text-gray-900">{template.category}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Court Formatting</dt>
            <dd className="text-sm text-gray-900">
              {court ? court.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase()) : 'Generic'}
            </dd>
          </div>
        </dl>
      </div>
      
      <div className="bg-gray-50 rounded-lg p-4">
        <h3 className="font-medium text-gray-900 mb-3">Document Variables</h3>
        <dl className="space-y-3">
          {Object.entries(variables).map(([key, value]) => (
            <div key={key} className="grid grid-cols-1 md:grid-cols-3 gap-2">
              <dt className="text-sm font-medium text-gray-500 capitalize">
                {key.replace(/_/g, ' ')}
              </dt>
              <dd className="text-sm text-gray-900 md:col-span-2">{value || '(not provided)'}</dd>
            </div>
          ))}
        </dl>
      </div>
    </div>
  </div>
);

// Result Step
export const ResultStep: React.FC<{
  result: DraftResult;
}> = ({ result }) => (
  <div className="p-6">
    <h2 className="text-xl font-semibold text-gray-900 mb-4">Document Generated Successfully!</h2>
    
    {result.warnings.length > 0 && (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
        <div className="flex">
          <AlertTriangle className="h-5 w-5 text-yellow-400" />
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800">Warnings</h3>
            <ul className="mt-1 text-sm text-yellow-700 list-disc list-inside">
              {result.warnings.map((warning, index) => (
                <li key={index}>{warning}</li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    )}
    
    <div className="space-y-4">
      {result.docx_path && (
        <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
          <div className="flex items-center">
            <FileText className="h-8 w-8 text-blue-600" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-gray-900">Word Document</h3>
              <p className="text-sm text-gray-500">Editable DOCX format</p>
            </div>
          </div>
          <div className="flex space-x-2">
            <button
              type="button"
              onClick={() => window.open(result.docx_path)}
              className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              <Eye className="h-4 w-4 mr-2" />
              View
            </button>
            <button
              type="button"
              onClick={() => {
                const link = document.createElement('a');
                link.href = result.docx_path!;
                link.download = '';
                link.click();
              }}
              className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
            >
              <Download className="h-4 w-4 mr-2" />
              Download
            </button>
          </div>
        </div>
      )}
      
      {result.pdf_path && (
        <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
          <div className="flex items-center">
            <FileText className="h-8 w-8 text-red-600" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-gray-900">PDF Document</h3>
              <p className="text-sm text-gray-500">Print-ready PDF format</p>
            </div>
          </div>
          <div className="flex space-x-2">
            <button
              type="button"
              onClick={() => window.open(result.pdf_path)}
              className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              <Eye className="h-4 w-4 mr-2" />
              View
            </button>
            <button
              type="button"
              onClick={() => {
                const link = document.createElement('a');
                link.href = result.pdf_path!;
                link.download = '';
                link.click();
              }}
              className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700"
            >
              <Download className="h-4 w-4 mr-2" />
              Download
            </button>
          </div>
        </div>
      )}
    </div>
  </div>
);

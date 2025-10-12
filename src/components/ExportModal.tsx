// Export modal component for PA eDocket Desktop

import React, { useState } from 'react';
import { X, Download, FileText, Database, Archive } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import type { SearchResult, ExportFormat, ExportOptions } from '../types/domain';

interface ExportModalProps {
  data: SearchResult[];
  onClose: () => void;
  type: 'search' | 'docket';
}

export const ExportModal: React.FC<ExportModalProps> = ({
  data,
  onClose,
  type,
}) => {
  const [selectedFormat, setSelectedFormat] = useState<ExportFormat>('JSON');
  const [isExporting, setIsExporting] = useState(false);
  const [exportOptions, setExportOptions] = useState({
    includeAttachments: false,
    includeFinancials: true,
    includeEvents: true,
    includeParties: true,
  });
  
  const formats = [
    {
      value: 'JSON' as ExportFormat,
      label: 'JSON',
      description: 'Structured data format for developers',
      icon: Database,
    },
    {
      value: 'CSV' as ExportFormat,
      label: 'CSV',
      description: 'Spreadsheet-compatible format',
      icon: FileText,
    },
    {
      value: 'PDF' as ExportFormat,
      label: 'PDF',
      description: 'Formatted document for printing',
      icon: FileText,
    },
    {
      value: 'ZIP' as ExportFormat,
      label: 'ZIP Archive',
      description: 'Complete package with all files',
      icon: Archive,
    },
  ];
  
  const handleExport = async () => {
    setIsExporting(true);
    
    try {
      const options: ExportOptions = {
        format: selectedFormat,
        ...exportOptions,
      };
      
      console.log('Exporting data:', { data, options });
      
      // Call Tauri command to export data
      const result = await invoke<{ path: string; manifest: any }>('export_data', {
        data,
        options,
        type,
      });
      
      console.log('Export completed:', result);
      
      // Show success message or open file location
      // This would typically show a notification or open the file explorer
      alert(`Export completed successfully!\nFile saved to: ${result.path}`);
      
      onClose();
    } catch (error) {
      console.error('Export error:', error);
      alert('Export failed. Please try again.');
    } finally {
      setIsExporting(false);
    }
  };
  
  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex min-h-screen items-center justify-center p-4">
        {/* Backdrop */}
        <div 
          className="fixed inset-0 bg-black bg-opacity-25 transition-opacity"
          onClick={onClose}
        />
        
        {/* Modal */}
        <div className="relative bg-white rounded-lg shadow-xl max-w-md w-full">
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b">
            <h3 className="text-lg font-medium text-gray-900">
              Export {type === 'search' ? 'Search Results' : 'Docket Data'}
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <X className="h-6 w-6" />
            </button>
          </div>
          
          {/* Content */}
          <div className="p-6 space-y-6">
            {/* Export Summary */}
            <div className="bg-gray-50 rounded-lg p-4">
              <h4 className="text-sm font-medium text-gray-900 mb-2">Export Summary</h4>
              <div className="text-sm text-gray-600">
                <p>{data.length} {type === 'search' ? 'search results' : 'docket records'} selected</p>
                <p>Format: {selectedFormat}</p>
              </div>
            </div>
            
            {/* Format Selection */}
            <div>
              <h4 className="text-sm font-medium text-gray-900 mb-3">Export Format</h4>
              <div className="space-y-2">
                {formats.map((format) => {
                  const Icon = format.icon;
                  return (
                    <label
                      key={format.value}
                      className={`flex items-center p-3 border rounded-lg cursor-pointer transition-colors ${
                        selectedFormat === format.value
                          ? 'border-blue-500 bg-blue-50'
                          : 'border-gray-200 hover:bg-gray-50'
                      }`}
                    >
                      <input
                        type="radio"
                        name="format"
                        value={format.value}
                        checked={selectedFormat === format.value}
                        onChange={(e) => setSelectedFormat(e.target.value as ExportFormat)}
                        className="sr-only"
                      />
                      <Icon className="h-5 w-5 text-gray-400 mr-3" />
                      <div className="flex-1">
                        <div className="text-sm font-medium text-gray-900">
                          {format.label}
                        </div>
                        <div className="text-xs text-gray-500">
                          {format.description}
                        </div>
                      </div>
                    </label>
                  );
                })}
              </div>
            </div>
            
            {/* Export Options */}
            <div>
              <h4 className="text-sm font-medium text-gray-900 mb-3">Include in Export</h4>
              <div className="space-y-2">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={exportOptions.includeParties}
                    onChange={(e) => setExportOptions(prev => ({
                      ...prev,
                      includeParties: e.target.checked
                    }))}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700">Party information</span>
                </label>
                
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={exportOptions.includeEvents}
                    onChange={(e) => setExportOptions(prev => ({
                      ...prev,
                      includeEvents: e.target.checked
                    }))}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700">Case events</span>
                </label>
                
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={exportOptions.includeFinancials}
                    onChange={(e) => setExportOptions(prev => ({
                      ...prev,
                      includeFinancials: e.target.checked
                    }))}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700">Financial information</span>
                </label>
                
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={exportOptions.includeAttachments}
                    onChange={(e) => setExportOptions(prev => ({
                      ...prev,
                      includeAttachments: e.target.checked
                    }))}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <span className="ml-2 text-sm text-gray-700">Document attachments</span>
                </label>
              </div>
            </div>
          </div>
          
          {/* Footer */}
          <div className="flex items-center justify-end space-x-3 p-6 border-t">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              onClick={handleExport}
              disabled={isExporting}
              className="flex items-center px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isExporting ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  Exporting...
                </>
              ) : (
                <>
                  <Download className="h-4 w-4 mr-2" />
                  Export
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

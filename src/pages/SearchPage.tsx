// Search page for PA eDocket Desktop

import React, { useState, useCallback } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Search, Filter, Download, Eye, Calendar, MapPin } from 'lucide-react';
import { SearchForm } from '../components/SearchForm';
import { SearchResults } from '../components/SearchResults';
import { ExportModal } from '../components/ExportModal';
import { useSearch } from '../hooks/useSearch';
import type { SearchParams, SearchResult } from '../types/domain';

const searchSchema = z.object({
  term: z.string().optional(),
  court: z.enum(['MDJ', 'CP', 'APP']).optional(),
  county: z.string().optional(),
  from: z.string().optional(),
  to: z.string().optional(),
  docket: z.string().optional(),
  otn: z.string().optional(),
  sid: z.string().optional(),
  page: z.number().min(1).default(1),
  limit: z.number().min(1).max(100).default(25),
});

type SearchFormData = z.infer<typeof searchSchema>;

export const SearchPage: React.FC = () => {
  const [showFilters, setShowFilters] = useState(false);
  const [showExportModal, setShowExportModal] = useState(false);
  const [selectedResults, setSelectedResults] = useState<SearchResult[]>([]);
  
  const {
    data: searchResults,
    isLoading,
    error,
    search,
    pagination,
  } = useSearch();
  
  const form = useForm<SearchFormData>({
    resolver: zodResolver(searchSchema),
    defaultValues: {
      page: 1,
      limit: 25,
    },
  });
  
  const onSubmit = useCallback((data: SearchFormData) => {
    const searchParams: SearchParams = {
      ...data,
      // Convert empty strings to undefined
      term: data.term || undefined,
      county: data.county || undefined,
      from: data.from || undefined,
      to: data.to || undefined,
      docket: data.docket || undefined,
      otn: data.otn || undefined,
      sid: data.sid || undefined,
    };
    
    search(searchParams);
  }, [search]);
  
  const handlePageChange = useCallback((page: number) => {
    const currentValues = form.getValues();
    form.setValue('page', page);
    onSubmit({ ...currentValues, page });
  }, [form, onSubmit]);
  
  const handleExport = useCallback(() => {
    if (selectedResults.length === 0 && searchResults) {
      setSelectedResults(searchResults);
    }
    setShowExportModal(true);
  }, [selectedResults, searchResults]);
  
  const handleSelectResult = useCallback((result: SearchResult, selected: boolean) => {
    setSelectedResults(prev => {
      if (selected) {
        return [...prev, result];
      } else {
        return prev.filter(r => r.id !== result.id);
      }
    });
  }, []);
  
  const handleSelectAll = useCallback((selected: boolean) => {
    if (selected && searchResults) {
      setSelectedResults(searchResults);
    } else {
      setSelectedResults([]);
    }
  }, [searchResults]);
  
  return (
    <div className="space-y-6">
      {/* Search Form */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-medium text-gray-900">Search Pennsylvania Court Records</h2>
          <button
            type="button"
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
          >
            <Filter className="h-4 w-4 mr-2" />
            {showFilters ? 'Hide Filters' : 'Show Filters'}
          </button>
        </div>
        
        <SearchForm
          form={form}
          onSubmit={onSubmit}
          showAdvanced={showFilters}
          isLoading={isLoading}
        />
      </div>
      
      {/* Search Results */}
      {(searchResults || isLoading || error) && (
        <div className="bg-white rounded-lg shadow">
          {/* Results Header */}
          <div className="px-6 py-4 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-4">
                <h3 className="text-lg font-medium text-gray-900">Search Results</h3>
                {pagination && (
                  <span className="text-sm text-gray-500">
                    {pagination.total} results found
                  </span>
                )}
              </div>
              
              <div className="flex items-center space-x-2">
                {searchResults && searchResults.length > 0 && (
                  <>
                    <span className="text-sm text-gray-500">
                      {selectedResults.length} selected
                    </span>
                    <button
                      onClick={handleExport}
                      className="flex items-center px-3 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700"
                    >
                      <Download className="h-4 w-4 mr-2" />
                      Export
                    </button>
                  </>
                )}
              </div>
            </div>
          </div>
          
          {/* Results Content */}
          <div className="p-6">
            {isLoading && (
              <div className="flex items-center justify-center py-12">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                <span className="ml-3 text-gray-600">Searching court records...</span>
              </div>
            )}
            
            {error && (
              <div className="bg-red-50 border border-red-200 rounded-md p-4">
                <div className="flex">
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-red-800">Search Error</h3>
                    <div className="mt-2 text-sm text-red-700">
                      {error.message || 'An error occurred while searching. Please try again.'}
                    </div>
                  </div>
                </div>
              </div>
            )}
            
            {searchResults && searchResults.length === 0 && !isLoading && (
              <div className="text-center py-12">
                <Search className="mx-auto h-12 w-12 text-gray-400" />
                <h3 className="mt-2 text-sm font-medium text-gray-900">No results found</h3>
                <p className="mt-1 text-sm text-gray-500">
                  Try adjusting your search criteria or check your spelling.
                </p>
              </div>
            )}
            
            {searchResults && searchResults.length > 0 && (
              <SearchResults
                results={searchResults}
                selectedResults={selectedResults}
                onSelectResult={handleSelectResult}
                onSelectAll={handleSelectAll}
                pagination={pagination}
                onPageChange={handlePageChange}
              />
            )}
          </div>
        </div>
      )}
      
      {/* Export Modal */}
      {showExportModal && (
        <ExportModal
          data={selectedResults.length > 0 ? selectedResults : searchResults || []}
          onClose={() => setShowExportModal(false)}
          type="search"
        />
      )}
    </div>
  );
};

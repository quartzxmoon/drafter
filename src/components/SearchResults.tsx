// Search results component for PA eDocket Desktop
import React from 'react';
import { Link } from 'react-router-dom';
import { 
  Eye, 
  Download, 
  Calendar, 
  MapPin, 
  Hash, 
  User,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight
} from 'lucide-react';
import type { SearchResult } from '../types/domain';

// Local PaginationInfo to replace missing domain.ts export
interface PaginationInfo {
  currentPage: number;
  totalPages: number;
  pageSize: number;
  total: number;
}

interface PartyInfo {
  name: string;
  role?: string;
}

interface SearchResultsProps {
  results: SearchResult[];
  selectedResults: SearchResult[];
  onSelectResult: (result: SearchResult, selected: boolean) => void;
  onSelectAll: (selected: boolean) => void;
  pagination?: PaginationInfo;
  onPageChange: (page: number) => void;
}

export const SearchResults: React.FC<SearchResultsProps> = ({
  results,
  selectedResults,
  onSelectResult,
  onSelectAll,
  pagination,
  onPageChange,
}) => {
  const isSelected = (result: SearchResult): boolean => {
    return selectedResults.some(r => r.id === result.id);
  };
  
  const allSelected: boolean = results.length > 0 && results.every(result => isSelected(result));
  const someSelected: boolean = selectedResults.length > 0 && !allSelected;
  
  const formatDate = (dateString?: string): string => {
    if (!dateString) return 'N/A';
    try {
      return new Date(dateString).toLocaleDateString();
    } catch {
      return dateString;
    }
  };
  
  const getCourtDisplayName = (court: string): string => {
    const courtMap: Record<string, string> = {
      'MDJ': 'Magisterial District Court',
      'CP': 'Court of Common Pleas',
      'APP': 'Appellate Court',
    };
    return courtMap[court] || court;
  };
  
  const getCaseTypeColor = (caseType?: string): string => {
    const colorMap: Record<string, string> = {
      'Criminal': 'bg-red-100 text-red-800',
      'Civil': 'bg-blue-100 text-blue-800',
      'Family': 'bg-green-100 text-green-800',
      'Orphans': 'bg-purple-100 text-purple-800',
      'Traffic': 'bg-yellow-100 text-yellow-800',
    };
    return colorMap[caseType || ''] || 'bg-gray-100 text-gray-800';
  };
  
  return (
    <div className="space-y-4">
      {/* Results Table */}
      <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
        <table className="min-w-full divide-y divide-gray-300">
          <thead className="bg-gray-50">
            <tr>
              <th scope="col" className="relative w-12 px-6 sm:w-16 sm:px-8">
                <input
                  type="checkbox"
                  className="absolute left-4 top-1/2 -mt-2 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 sm:left-6"
                  checked={allSelected}
                  ref={(input) => {
                    if (input) input.indeterminate = someSelected;
                  }}
                  onChange={(e) => onSelectAll(e.target.checked)}
                  aria-label="Select all results"
                  title="Select all results"
                />
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wide">
                Case Information
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wide">
                Parties
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wide">
                Court & Date
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wide">
                Status
              </th>
              <th scope="col" className="relative px-6 py-3">
                <span className="sr-only">Actions</span>
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 bg-white">
            {results.map((result) => (
              <tr key={result.id} className={isSelected(result) ? 'bg-blue-50' : 'hover:bg-gray-50'}>
                <td className="relative w-12 px-6 sm:w-16 sm:px-8">
                  <input
                    type="checkbox"
                    className="absolute left-4 top-1/2 -mt-2 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 sm:left-6"
                    checked={isSelected(result)}
                    onChange={(e) => onSelectResult(result, e.target.checked)}
                    aria-label={`Select result ${result.docketNumber}`}
                    title={`Select result ${result.docketNumber}`}
                  />
                </td>
                
                {/* Case Information */}
                <td className="px-6 py-4">
                  <div className="space-y-1">
                    <div className="flex items-center">
                      <Hash className="h-4 w-4 text-gray-400 mr-1" />
                      <span className="text-sm font-medium text-gray-900">
                        {result.docketNumber}
                      </span>
                    </div>
                    {result.caseType && (
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getCaseTypeColor(result.caseType)}`}>
                        {result.caseType}
                      </span>
                    )}
                    {result.otn && (
                      <div className="text-xs text-gray-500">
                        OTN: {result.otn}
                      </div>
                    )}
                  </div>
                </td>
                
                {/* Parties */}
                <td className="px-6 py-4">
                  <div className="space-y-1">
                    {(() => {
                      const partiesList: PartyInfo[] = Array.isArray(result.parties) ? result.parties : [];
                      if (partiesList.length > 0) {
                        return partiesList.slice(0, 2).map((party: PartyInfo, index: number) => (
                          <div key={index} className="flex items-center text-sm">
                            <User className="h-3 w-3 text-gray-400 mr-1" />
                            <span className="text-gray-900">{party.name}</span>
                            {party.role && (
                              <span className="ml-2 text-xs text-gray-500">({party.role})</span>
                            )}
                          </div>
                        ));
                      }
                      return <span className="text-sm text-gray-500">No parties listed</span>;
                    })()}
                    {Array.isArray(result.parties) && result.parties.length > 2 && (
                      <div className="text-xs text-gray-500">
                        +{result.parties.length - 2} more
                      </div>
                    )}
                  </div>
                </td>
                
                {/* Court & Date */}
                <td className="px-6 py-4">
                  <div className="space-y-1">
                    <div className="flex items-center text-sm">
                      <MapPin className="h-4 w-4 text-gray-400 mr-1" />
                      <span className="text-gray-900">
                        {getCourtDisplayName(result.court)}
                      </span>
                    </div>
                    {result.county && (
                      <div className="text-xs text-gray-500">
                        {result.county} County
                      </div>
                    )}
                    <div className="flex items-center text-xs text-gray-500">
                      <Calendar className="h-3 w-3 mr-1" />
                      Filed: {formatDate(result.filedDate)}
                    </div>
                  </div>
                </td>
                
                {/* Status */}
                <td className="px-6 py-4">
                  <div className="space-y-1">
                    {result.status && (
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                        {result.status}
                      </span>
                    )}
                    {result.lastActivity && (
                      <div className="text-xs text-gray-500">
                        Last: {formatDate(result.lastActivity)}
                      </div>
                    )}
                  </div>
                </td>
                
                {/* Actions */}
                <td className="px-6 py-4 text-right text-sm font-medium">
                  <div className="flex items-center justify-end space-x-2">
                    <Link
                      to={`/docket/${result.id}`}
                      className="text-blue-600 hover:text-blue-900 p-1 rounded"
                      title="View Docket"
                    >
                      <Eye className="h-4 w-4" />
                    </Link>
                    <button
                      className="text-gray-400 hover:text-gray-600 p-1 rounded"
                      aria-label="Download"
                      title="Download"
                    >
                      <Download className="h-4 w-4" />
                      <span className="sr-only">Download</span>
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      
      {/* Pagination */}
      {pagination && pagination.totalPages > 1 && (
        <div className="flex items-center justify-between border-t border-gray-200 bg-white px-4 py-3 sm:px-6">
          <div className="flex flex-1 justify-between sm:hidden">
            <button
              onClick={() => onPageChange(pagination.currentPage - 1)}
              disabled={pagination.currentPage <= 1}
              className="relative inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Previous
            </button>
            <button
              onClick={() => onPageChange(pagination.currentPage + 1)}
              disabled={pagination.currentPage >= pagination.totalPages}
              className="relative ml-3 inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Next
            </button>
          </div>
          
          <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
            <div>
              <p className="text-sm text-gray-700">
                Showing{' '}
                <span className="font-medium">
                  {(pagination.currentPage - 1) * pagination.pageSize + 1}
                </span>{' '}
                to{' '}
                <span className="font-medium">
                  {Math.min(pagination.currentPage * pagination.pageSize, pagination.total)}
                </span>{' '}
                of{' '}
                <span className="font-medium">{pagination.total}</span> results
              </p>
            </div>
            
            <div>
              <nav className="isolate inline-flex -space-x-px rounded-md shadow-sm" aria-label="Pagination">
                <button
                  onClick={() => onPageChange(1)}
                  disabled={pagination.currentPage <= 1}
                  title="First page"
                  aria-label="First page"
                  className="relative inline-flex items-center rounded-l-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <ChevronsLeft className="h-5 w-5" />
                </button>
                <button
                  onClick={() => onPageChange(pagination.currentPage - 1)}
                  disabled={pagination.currentPage <= 1}
                  title="Previous page"
                  aria-label="Previous page"
                  className="relative inline-flex items-center px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <ChevronLeft className="h-5 w-5" />
                </button>
                
                {/* Page numbers */}
                {Array.from({ length: Math.min(5, pagination.totalPages) }, (_, i) => {
                  const page = Math.max(1, pagination.currentPage - 2) + i;
                  if

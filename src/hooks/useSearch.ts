// Search hook for PA eDocket Desktop

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { SearchParams, SearchResult } from '../types/domain';

// Local fallback for pagination info to avoid dependency on domain export
type PaginationInfo = {
  page?: number;
  perPage?: number;
  total?: number;
  totalPages?: number;
  [key: string]: any;
};

interface UseSearchReturn {
  data: SearchResult[] | null;
  isLoading: boolean;
  error: Error | null;
  search: (params: SearchParams) => Promise<void>;
  pagination: PaginationInfo | null;
}

export const useSearch = (): UseSearchReturn => {
  const [data, setData] = useState<SearchResult[] | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [pagination, setPagination] = useState<PaginationInfo | null>(null);
  
  const search = useCallback(async (params: SearchParams) => {
    setIsLoading(true);
    setError(null);
    
    try {
      console.log('Searching with params:', params);
      
      // Call Tauri command to search
      const response = await invoke<{
        results: SearchResult[];
        pagination: PaginationInfo;
      }>('cmd_search', { params });
      
      console.log('Search response:', response);
      
      setData(response.results);
      setPagination(response.pagination);
    } catch (err) {
      console.error('Search error:', err);
      setError(err instanceof Error ? err : new Error('Search failed'));
      setData(null);
      setPagination(null);
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  return {
    data,
    isLoading,
    error,
    search,
    pagination,
  };
};

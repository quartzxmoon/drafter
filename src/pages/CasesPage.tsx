// Case Management Page with Hierarchical Organization
// Professional case/matter management with folders, practice areas, and quick actions

import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import {
  Folder,
  FolderOpen,
  ChevronRight,
  ChevronDown,
  Plus,
  Search,
  Filter,
  MoreVertical,
  FileText,
  Calendar,
  Clock,
  DollarSign,
  AlertCircle,
  CheckCircle,
  User,
  Scale,
  Building,
  Tag,
} from 'lucide-react';

interface Matter {
  id: string;
  client_id: string;
  matter_number: string;
  title: string;
  matter_type: string;
  status: string;
  court_name?: string;
  docket_number?: string;
  next_deadline?: string;
  total_time: number;
  total_expenses: number;
}

interface CaseFolder {
  id: string;
  name: string;
  color: string;
  icon: string;
  matter_count: number;
  parent_folder_id?: string;
  children?: CaseFolder[];
}

interface PracticeArea {
  id: string;
  name: string;
  parent_area_id?: string;
  matter_count: number;
  children?: PracticeArea[];
}

export const CasesPage: React.FC = () => {
  const navigate = useNavigate();
  const [matters, setMatters] = useState<Matter[]>([]);
  const [folders, setFolders] = useState<CaseFolder[]>([]);
  const [practiceAreas, setPracticeAreas] = useState<PracticeArea[]>([]);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [selectedPracticeArea, setSelectedPracticeArea] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState<'list' | 'grid' | 'timeline'>('list');
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [expandedAreas, setExpandedAreas] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [selectedFolder, selectedPracticeArea]);

  const loadData = async () => {
    setLoading(true);
    try {
      // Load matters
      const mattersData = await invoke<Matter[]>('cmd_list_matters', {
        folderId: selectedFolder,
        practiceAreaId: selectedPracticeArea,
      });
      setMatters(mattersData);

      // Load folders hierarchy
      const foldersData = await invoke<CaseFolder[]>('cmd_get_case_folders');
      setFolders(buildHierarchy(foldersData));

      // Load practice areas hierarchy
      const areasData = await invoke<PracticeArea[]>('cmd_get_practice_areas');
      setPracticeAreas(buildHierarchy(areasData));
    } catch (error) {
      console.error('Failed to load data:', error);
    } finally {
      setLoading(false);
    }
  };

  const buildHierarchy = <T extends { id: string; parent_folder_id?: string; parent_area_id?: string }>(
    items: T[]
  ): T[] => {
    const map = new Map<string, T & { children?: T[] }>();
    const roots: (T & { children?: T[] })[] = [];

    items.forEach(item => {
      map.set(item.id, { ...item, children: [] });
    });

    items.forEach(item => {
      const parentId = (item as any).parent_folder_id || (item as any).parent_area_id;
      if (parentId && map.has(parentId)) {
        map.get(parentId)!.children!.push(map.get(item.id)!);
      } else {
        roots.push(map.get(item.id)!);
      }
    });

    return roots;
  };

  const toggleFolder = (folderId: string) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId);
    } else {
      newExpanded.add(folderId);
    }
    setExpandedFolders(newExpanded);
  };

  const togglePracticeArea = (areaId: string) => {
    const newExpanded = new Set(expandedAreas);
    if (newExpanded.has(areaId)) {
      newExpanded.delete(areaId);
    } else {
      newExpanded.add(areaId);
    }
    setExpandedAreas(newExpanded);
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active': return 'bg-green-100 text-green-800';
      case 'pending': return 'bg-yellow-100 text-yellow-800';
      case 'closed': return 'bg-gray-100 text-gray-800';
      default: return 'bg-blue-100 text-blue-800';
    }
  };

  const getMatterTypeIcon = (type: string) => {
    switch (type.toLowerCase()) {
      case 'civil': return <Scale className="w-4 h-4" />;
      case 'criminal': return <AlertCircle className="w-4 h-4" />;
      case 'family': return <User className="w-4 h-4" />;
      case 'business': return <Building className="w-4 h-4" />;
      default: return <FileText className="w-4 h-4" />;
    }
  };

  const renderFolderTree = (folder: CaseFolder, level: number = 0) => {
    const isExpanded = expandedFolders.has(folder.id);
    const hasChildren = folder.children && folder.children.length > 0;

    return (
      <div key={folder.id}>
        <div
          className={`flex items-center gap-2 px-3 py-2 hover:bg-gray-100 rounded cursor-pointer ${
            selectedFolder === folder.id ? 'bg-blue-50' : ''
          }`}
          style={{ paddingLeft: `${level * 16 + 12}px` }}
          onClick={() => {
            setSelectedFolder(folder.id);
            if (hasChildren) toggleFolder(folder.id);
          }}
        >
          {hasChildren && (
            <button onClick={(e) => { e.stopPropagation(); toggleFolder(folder.id); }}>
              {isExpanded ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
            </button>
          )}
          {isExpanded ? <FolderOpen className="w-4 h-4" style={{ color: folder.color }} /> : <Folder className="w-4 h-4" style={{ color: folder.color }} />}
          <span className="flex-1 text-sm">{folder.name}</span>
          <span className="text-xs text-gray-500">{folder.matter_count}</span>
        </div>
        {isExpanded && hasChildren && folder.children!.map(child => renderFolderTree(child, level + 1))}
      </div>
    );
  };

  const renderPracticeAreaTree = (area: PracticeArea, level: number = 0) => {
    const isExpanded = expandedAreas.has(area.id);
    const hasChildren = area.children && area.children.length > 0;

    return (
      <div key={area.id}>
        <div
          className={`flex items-center gap-2 px-3 py-2 hover:bg-gray-100 rounded cursor-pointer ${
            selectedPracticeArea === area.id ? 'bg-purple-50' : ''
          }`}
          style={{ paddingLeft: `${level * 16 + 12}px` }}
          onClick={() => {
            setSelectedPracticeArea(area.id);
            if (hasChildren) togglePracticeArea(area.id);
          }}
        >
          {hasChildren && (
            <button onClick={(e) => { e.stopPropagation(); togglePracticeArea(area.id); }}>
              {isExpanded ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
            </button>
          )}
          <Tag className="w-4 h-4 text-purple-600" />
          <span className="flex-1 text-sm">{area.name}</span>
          <span className="text-xs text-gray-500">{area.matter_count}</span>
        </div>
        {isExpanded && hasChildren && area.children!.map(child => renderPracticeAreaTree(child, level + 1))}
      </div>
    );
  };

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Left Sidebar - Folders & Practice Areas */}
      <div className="w-80 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b">
          <button
            onClick={() => navigate('/cases/new')}
            className="w-full bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 flex items-center justify-center gap-2"
          >
            <Plus className="w-4 h-4" />
            New Case
          </button>
        </div>

        <div className="flex-1 overflow-y-auto">
          {/* Folders Section */}
          <div className="p-4">
            <h3 className="text-xs font-semibold text-gray-500 uppercase mb-2">Case Folders</h3>
            <div className="space-y-1">
              {folders.map(folder => renderFolderTree(folder))}
            </div>
          </div>

          {/* Practice Areas Section */}
          <div className="p-4 border-t">
            <h3 className="text-xs font-semibold text-gray-500 uppercase mb-2">Practice Areas</h3>
            <div className="space-y-1">
              {practiceAreas.map(area => renderPracticeAreaTree(area))}
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="bg-white border-b border-gray-200 p-4">
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search cases..."
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md"
                />
              </div>
            </div>
            <button className="p-2 hover:bg-gray-100 rounded">
              <Filter className="w-5 h-5" />
            </button>
            <div className="flex gap-1 border rounded">
              <button
                onClick={() => setViewMode('list')}
                className={`px-3 py-1 ${viewMode === 'list' ? 'bg-gray-100' : ''}`}
              >
                List
              </button>
              <button
                onClick={() => setViewMode('grid')}
                className={`px-3 py-1 ${viewMode === 'grid' ? 'bg-gray-100' : ''}`}
              >
                Grid
              </button>
              <button
                onClick={() => setViewMode('timeline')}
                className={`px-3 py-1 ${viewMode === 'timeline' ? 'bg-gray-100' : ''}`}
              >
                Timeline
              </button>
            </div>
          </div>
        </div>

        {/* Matters List */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading ? (
            <div className="text-center text-gray-500">Loading cases...</div>
          ) : matters.length === 0 ? (
            <div className="text-center text-gray-500">No cases found</div>
          ) : viewMode === 'list' ? (
            <div className="space-y-3">
              {matters.map(matter => (
                <div
                  key={matter.id}
                  className="bg-white p-4 rounded-lg border border-gray-200 hover:shadow-md transition-shadow cursor-pointer"
                  onClick={() => navigate(`/cases/${matter.id}`)}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        {getMatterTypeIcon(matter.matter_type)}
                        <h3 className="font-semibold">{matter.title}</h3>
                        <span className={`px-2 py-1 rounded-full text-xs ${getStatusColor(matter.status)}`}>
                          {matter.status}
                        </span>
                      </div>
                      <div className="grid grid-cols-4 gap-4 text-sm text-gray-600">
                        <div>
                          <span className="text-gray-500">Matter #:</span> {matter.matter_number}
                        </div>
                        {matter.docket_number && (
                          <div>
                            <span className="text-gray-500">Docket:</span> {matter.docket_number}
                          </div>
                        )}
                        {matter.court_name && (
                          <div>
                            <span className="text-gray-500">Court:</span> {matter.court_name}
                          </div>
                        )}
                        {matter.next_deadline && (
                          <div className="flex items-center gap-1">
                            <Clock className="w-4 h-4" />
                            {matter.next_deadline}
                          </div>
                        )}
                      </div>
                      <div className="flex gap-4 mt-2 text-sm">
                        <div className="flex items-center gap-1">
                          <Calendar className="w-4 h-4 text-gray-400" />
                          {matter.total_time}h
                        </div>
                        <div className="flex items-center gap-1">
                          <DollarSign className="w-4 h-4 text-gray-400" />
                          ${matter.total_expenses.toFixed(2)}
                        </div>
                      </div>
                    </div>
                    <button className="p-2 hover:bg-gray-100 rounded">
                      <MoreVertical className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ) : viewMode === 'grid' ? (
            <div className="grid grid-cols-3 gap-4">
              {matters.map(matter => (
                <div
                  key={matter.id}
                  className="bg-white p-4 rounded-lg border border-gray-200 hover:shadow-md transition-shadow cursor-pointer"
                  onClick={() => navigate(`/cases/${matter.id}`)}
                >
                  <div className="flex items-center gap-2 mb-3">
                    {getMatterTypeIcon(matter.matter_type)}
                    <span className={`px-2 py-1 rounded-full text-xs ${getStatusColor(matter.status)}`}>
                      {matter.status}
                    </span>
                  </div>
                  <h3 className="font-semibold mb-2">{matter.title}</h3>
                  <p className="text-sm text-gray-600">{matter.matter_number}</p>
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-2">
              <div className="text-center text-gray-500">Timeline view coming soon</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

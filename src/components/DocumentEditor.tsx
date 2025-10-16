// Microsoft Word-like Document Editor with Legal Tools
// Advanced rich text editor with citation integration, pleading formatting, and AI assistance

import React, { useState, useEffect, useCallback } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import TextAlign from '@tiptap/extension-text-align';
import Underline from '@tiptap/extension-underline';
import Highlight from '@tiptap/extension-highlight';
import Table from '@tiptap/extension-table';
import TableRow from '@tiptap/extension-table-row';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import TextStyle from '@tiptap/extension-text-style';
import Color from '@tiptap/extension-color';
import FontFamily from '@tiptap/extension-font-family';
import Subscript from '@tiptap/extension-subscript';
import Superscript from '@tiptap/extension-superscript';
import {
  Bold,
  Italic,
  Underline as UnderlineIcon,
  AlignLeft,
  AlignCenter,
  AlignRight,
  AlignJustify,
  List,
  ListOrdered,
  Quote,
  Undo,
  Redo,
  Link as LinkIcon,
  Image as ImageIcon,
  Table as TableIcon,
  FileText,
  Save,
  Download,
  Eye,
  Settings,
  Search,
  BookOpen,
  Sparkles,
  Scale,
  FileSignature,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface DocumentEditorProps {
  matterId: string;
  documentId?: string;
  initialContent?: string;
  onSave?: (content: string) => void;
  onCitationInsert?: (citation: any) => void;
}

export const DocumentEditor: React.FC<DocumentEditorProps> = ({
  matterId,
  documentId,
  initialContent = '',
  onSave,
  onCitationInsert,
}) => {
  const [showCitationPanel, setShowCitationPanel] = useState(false);
  const [showAIAssistant, setShowAIAssistant] = useState(false);
  const [showOutline, setShowOutline] = useState(true);
  const [fontSize, setFontSize] = useState(12);
  const [fontFamily, setFontFamily] = useState('Times New Roman');
  const [lineNumbers, setLineNumbers] = useState(true);
  const [documentType, setDocumentType] = useState<string>('motion');
  const [savingStatus, setSavingStatus] = useState<'saved' | 'saving' | 'unsaved'>('saved');

  const editor = useEditor({
    extensions: [
      StarterKit,
      Underline,
      TextAlign.configure({
        types: ['heading', 'paragraph'],
      }),
      Highlight.configure({
        multicolor: true,
      }),
      Table.configure({
        resizable: true,
      }),
      TableRow,
      TableHeader,
      TableCell,
      TextStyle,
      Color,
      FontFamily,
      Subscript,
      Superscript,
    ],
    content: initialContent,
    editorProps: {
      attributes: {
        class: 'prose prose-sm sm:prose lg:prose-lg xl:prose-2xl focus:outline-none min-h-[800px] p-8',
        style: `font-family: ${fontFamily}; font-size: ${fontSize}pt; line-height: 2.0;`,
      },
    },
    onUpdate: ({ editor }) => {
      setSavingStatus('unsaved');
      // Auto-save after 2 seconds of inactivity
      setTimeout(() => handleAutoSave(), 2000);
    },
  });

  useEffect(() => {
    if (editor) {
      editor.commands.setContent(initialContent);
    }
  }, [initialContent, editor]);

  const handleAutoSave = useCallback(async () => {
    if (!editor) return;

    setSavingStatus('saving');
    const content = editor.getHTML();

    try {
      if (documentId) {
        await invoke('cmd_save_document', {
          documentId,
          content,
        });
      }
      setSavingStatus('saved');
      onSave?.(content);
    } catch (error) {
      console.error('Auto-save failed:', error);
      setSavingStatus('unsaved');
    }
  }, [editor, documentId, onSave]);

  const handleInsertCitation = async (citation: string) => {
    if (!editor) return;

    editor.chain().focus().insertContent(citation).run();
    onCitationInsert?.(citation);
  };

  const handleFormatAsPleading = async () => {
    if (!editor) return;

    const content = editor.getHTML();

    try {
      const formatted = await invoke<string>('cmd_format_as_pleading', {
        matterId,
        content,
        documentType,
      });

      editor.commands.setContent(formatted);
    } catch (error) {
      console.error('Formatting failed:', error);
    }
  };

  const handleGenerateTableOfAuthorities = async () => {
    if (!editor) return;

    const content = editor.getText();

    try {
      const toa = await invoke<string>('cmd_generate_toa', {
        content,
      });

      editor.chain().focus().insertContent(`<h2>TABLE OF AUTHORITIES</h2>${toa}`).run();
    } catch (error) {
      console.error('TOA generation failed:', error);
    }
  };

  const handleExport = async (format: 'pdf' | 'docx') => {
    if (!editor) return;

    const content = editor.getHTML();

    try {
      const filePath = await invoke<string>('cmd_export_document', {
        documentId,
        content,
        format,
      });

      console.log(`Document exported to: ${filePath}`);
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  if (!editor) {
    return <div>Loading editor...</div>;
  }

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Left Sidebar - Outline */}
      {showOutline && (
        <div className="w-64 bg-white border-r border-gray-200 p-4 overflow-y-auto">
          <h3 className="font-semibold mb-4 flex items-center">
            <FileText className="w-4 h-4 mr-2" />
            Document Outline
          </h3>
          <DocumentOutline editor={editor} />
        </div>
      )}

      {/* Main Editor */}
      <div className="flex-1 flex flex-col">
        {/* Toolbar */}
        <div className="bg-white border-b border-gray-200 p-2 sticky top-0 z-10">
          <div className="flex flex-wrap gap-2">
            {/* File Operations */}
            <div className="flex gap-1 border-r pr-2">
              <button
                onClick={() => handleAutoSave()}
                className="p-2 hover:bg-gray-100 rounded"
                title="Save"
              >
                <Save className="w-4 h-4" />
              </button>
              <button
                onClick={() => handleExport('pdf')}
                className="p-2 hover:bg-gray-100 rounded"
                title="Export PDF"
              >
                <Download className="w-4 h-4" />
              </button>
              <button
                onClick={() => handleFormatAsPleading()}
                className="p-2 hover:bg-gray-100 rounded text-blue-600"
                title="Format as Pleading"
              >
                <FileSignature className="w-4 h-4" />
              </button>
            </div>

            {/* Text Formatting */}
            <div className="flex gap-1 border-r pr-2">
              <button
                onClick={() => editor.chain().focus().toggleBold().run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive('bold') ? 'bg-gray-200' : ''}`}
                title="Bold"
              >
                <Bold className="w-4 h-4" />
              </button>
              <button
                onClick={() => editor.chain().focus().toggleItalic().run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive('italic') ? 'bg-gray-200' : ''}`}
                title="Italic"
              >
                <Italic className="w-4 h-4" />
              </button>
              <button
                onClick={() => editor.chain().focus().toggleUnderline().run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive('underline') ? 'bg-gray-200' : ''}`}
                title="Underline"
              >
                <UnderlineIcon className="w-4 h-4" />
              </button>
            </div>

            {/* Alignment */}
            <div className="flex gap-1 border-r pr-2">
              <button
                onClick={() => editor.chain().focus().setTextAlign('left').run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive({ textAlign: 'left' }) ? 'bg-gray-200' : ''}`}
                title="Align Left"
              >
                <AlignLeft className="w-4 h-4" />
              </button>
              <button
                onClick={() => editor.chain().focus().setTextAlign('center').run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive({ textAlign: 'center' }) ? 'bg-gray-200' : ''}`}
                title="Align Center"
              >
                <AlignCenter className="w-4 h-4" />
              </button>
              <button
                onClick={() => editor.chain().focus().setTextAlign('right').run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive({ textAlign: 'right' }) ? 'bg-gray-200' : ''}`}
                title="Align Right"
              >
                <AlignRight className="w-4 h-4" />
              </button>
              <button
                onClick={() => editor.chain().focus().setTextAlign('justify').run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive({ textAlign: 'justify' }) ? 'bg-gray-200' : ''}`}
                title="Justify"
              >
                <AlignJustify className="w-4 h-4" />
              </button>
            </div>

            {/* Lists */}
            <div className="flex gap-1 border-r pr-2">
              <button
                onClick={() => editor.chain().focus().toggleBulletList().run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive('bulletList') ? 'bg-gray-200' : ''}`}
                title="Bullet List"
              >
                <List className="w-4 h-4" />
              </button>
              <button
                onClick={() => editor.chain().focus().toggleOrderedList().run()}
                className={`p-2 hover:bg-gray-100 rounded ${editor.isActive('orderedList') ? 'bg-gray-200' : ''}`}
                title="Numbered List"
              >
                <ListOrdered className="w-4 h-4" />
              </button>
            </div>

            {/* Legal Tools */}
            <div className="flex gap-1 border-r pr-2">
              <button
                onClick={() => setShowCitationPanel(!showCitationPanel)}
                className="p-2 hover:bg-gray-100 rounded text-blue-600"
                title="Insert Citation"
              >
                <Scale className="w-4 h-4" />
              </button>
              <button
                onClick={handleGenerateTableOfAuthorities}
                className="p-2 hover:bg-gray-100 rounded text-purple-600"
                title="Generate Table of Authorities"
              >
                <BookOpen className="w-4 h-4" />
              </button>
              <button
                onClick={() => setShowAIAssistant(!showAIAssistant)}
                className="p-2 hover:bg-gray-100 rounded text-green-600"
                title="AI Assistant"
              >
                <Sparkles className="w-4 h-4" />
              </button>
            </div>

            {/* Font Controls */}
            <div className="flex gap-2 items-center">
              <select
                value={fontFamily}
                onChange={(e) => setFontFamily(e.target.value)}
                className="text-sm border rounded px-2 py-1"
              >
                <option value="Times New Roman">Times New Roman</option>
                <option value="Arial">Arial</option>
                <option value="Courier New">Courier New</option>
                <option value="Georgia">Georgia</option>
              </select>
              <select
                value={fontSize}
                onChange={(e) => setFontSize(Number(e.target.value))}
                className="text-sm border rounded px-2 py-1"
              >
                {[10, 11, 12, 13, 14, 16, 18, 20, 24].map(size => (
                  <option key={size} value={size}>{size}</option>
                ))}
              </select>
            </div>

            {/* Status */}
            <div className="ml-auto flex items-center gap-2 text-sm text-gray-600">
              <span className={`${
                savingStatus === 'saved' ? 'text-green-600' :
                savingStatus === 'saving' ? 'text-blue-600' :
                'text-orange-600'
              }`}>
                {savingStatus === 'saved' ? '✓ Saved' :
                 savingStatus === 'saving' ? 'Saving...' :
                 'Unsaved changes'}
              </span>
            </div>
          </div>
        </div>

        {/* Editor Content */}
        <div className="flex-1 overflow-y-auto bg-white">
          <div className="max-w-[8.5in] mx-auto relative">
            {lineNumbers && <LineNumbers editor={editor} />}
            <EditorContent editor={editor} />
          </div>
        </div>
      </div>

      {/* Right Sidebar - Citation Panel or AI Assistant */}
      {(showCitationPanel || showAIAssistant) && (
        <div className="w-96 bg-white border-l border-gray-200 p-4 overflow-y-auto">
          {showCitationPanel && <CitationPanel onInsert={handleInsertCitation} />}
          {showAIAssistant && <AIAssistant editor={editor} matterId={matterId} />}
        </div>
      )}
    </div>
  );
};

// Document Outline Component
const DocumentOutline: React.FC<{ editor: any }> = ({ editor }) => {
  const [headings, setHeadings] = useState<any[]>([]);

  useEffect(() => {
    const updateHeadings = () => {
      const json = editor.getJSON();
      const extractedHeadings: any[] = [];

      json.content?.forEach((node: any, index: number) => {
        if (node.type === 'heading') {
          extractedHeadings.push({
            level: node.attrs.level,
            text: node.content?.[0]?.text || '',
            id: index,
          });
        }
      });

      setHeadings(extractedHeadings);
    };

    editor.on('update', updateHeadings);
    updateHeadings();

    return () => {
      editor.off('update', updateHeadings);
    };
  }, [editor]);

  return (
    <div className="space-y-1">
      {headings.length === 0 ? (
        <p className="text-sm text-gray-500">No headings yet</p>
      ) : (
        headings.map((heading) => (
          <div
            key={heading.id}
            className={`text-sm cursor-pointer hover:bg-gray-100 p-1 rounded`}
            style={{ paddingLeft: `${heading.level * 12}px` }}
            onClick={() => {
              // Scroll to heading
            }}
          >
            {heading.text}
          </div>
        ))
      )}
    </div>
  );
};

// Line Numbers Component
const LineNumbers: React.FC<{ editor: any }> = ({ editor }) => {
  const [lineCount, setLineCount] = useState(0);

  useEffect(() => {
    const updateLineCount = () => {
      const text = editor.getText();
      setLineCount(text.split('\n').length);
    };

    editor.on('update', updateLineCount);
    updateLineCount();

    return () => {
      editor.off('update', updateLineCount);
    };
  }, [editor]);

  return (
    <div className="absolute left-0 top-0 w-12 text-right pr-2 text-sm text-gray-400 select-none pointer-events-none">
      {Array.from({ length: lineCount }, (_, i) => (
        <div key={i} style={{ height: '28px' }}>
          {i + 1}
        </div>
      ))}
    </div>
  );
};

// Citation Panel Component
const CitationPanel: React.FC<{ onInsert: (citation: string) => void }> = ({ onInsert }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [searching, setSearching] = useState(false);

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;

    setSearching(true);
    try {
      const results = await invoke('cmd_search_case_law', {
        query: searchQuery,
      });
      setSearchResults(results as any[]);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setSearching(false);
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="font-semibold flex items-center">
        <Scale className="w-4 h-4 mr-2" />
        Insert Citation
      </h3>

      <div className="space-y-2">
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
          placeholder="Search case law..."
          className="w-full px-3 py-2 border rounded-md"
        />
        <button
          onClick={handleSearch}
          disabled={searching}
          className="w-full bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 disabled:opacity-50"
        >
          {searching ? 'Searching...' : 'Search'}
        </button>
      </div>

      <div className="space-y-2 max-h-[600px] overflow-y-auto">
        {searchResults.map((result, index) => (
          <div
            key={index}
            className="p-3 border rounded-md hover:bg-gray-50 cursor-pointer"
            onClick={() => onInsert(result.citation)}
          >
            <div className="font-medium text-sm">{result.case_name}</div>
            <div className="text-xs text-gray-600 mt-1">{result.citation}</div>
          </div>
        ))}
      </div>
    </div>
  );
};

// AI Assistant Component
const AIAssistant: React.FC<{ editor: any; matterId: string }> = ({ editor, matterId }) => {
  const [suggestions, setSuggestions] = useState<string[]>([]);

  useEffect(() => {
    const getSuggestions = async () => {
      const content = editor.getText().slice(-500); // Last 500 chars

      try {
        const aiSuggestions = await invoke('cmd_get_ai_suggestions', {
          matterId,
          context: content,
        });
        setSuggestions(aiSuggestions as string[]);
      } catch (error) {
        console.error('AI suggestions failed:', error);
      }
    };

    const debounce = setTimeout(getSuggestions, 1000);
    return () => clearTimeout(debounce);
  }, [editor, matterId]);

  return (
    <div className="space-y-4">
      <h3 className="font-semibold flex items-center">
        <Sparkles className="w-4 h-4 mr-2" />
        AI Assistant
      </h3>

      <div className="space-y-2">
        {suggestions.map((suggestion, index) => (
          <div key={index} className="p-3 bg-blue-50 rounded-md text-sm">
            {suggestion}
          </div>
        ))}
      </div>
    </div>
  );
};

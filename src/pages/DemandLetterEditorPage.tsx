// Demand Letter Editor - Professional Rich Text Editor
// Generate, edit, and send demand letters with professional templates

import React, { useEffect, useState, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import {
  Send,
  Save,
  Download,
  Eye,
  ArrowLeft,
  Bold,
  Italic,
  Underline,
  AlignLeft,
  AlignCenter,
  AlignRight,
  List,
  ListOrdered,
  Plus,
  X,
  FileText,
  Mail,
  Printer,
} from 'lucide-react';

interface DemandLetter {
  id: string;
  settlement_calculation_id: string;
  matter_id: string;
  recipient_name: string;
  recipient_address: string;
  subject: string;
  opening_paragraph: string;
  facts_section: string;
  liability_section: string;
  damages_section: string;
  settlement_demand: number;
  deadline: string;
  closing_paragraph: string;
  exhibits: Exhibit[];
  letter_html: string;
  letter_pdf_path: string | null;
  created_at: string;
  created_by: string;
  sent_at: string | null;
}

interface Exhibit {
  exhibit_letter: string;
  description: string;
  file_path: string;
}

export default function DemandLetterEditorPage() {
  const { calcId } = useParams<{ calcId: string }>();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPreview, setShowPreview] = useState(false);

  // Letter Content
  const [recipientName, setRecipientName] = useState('');
  const [recipientAddress, setRecipientAddress] = useState('');
  const [subject, setSubject] = useState('');
  const [opening, setOpening] = useState('');
  const [facts, setFacts] = useState('');
  const [liability, setLiability] = useState('');
  const [damages, setDamages] = useState('');
  const [demandAmount, setDemandAmount] = useState(0);
  const [deadline, setDeadline] = useState('');
  const [closing, setClosing] = useState('');
  const [exhibits, setExhibits] = useState<Exhibit[]>([]);

  // Editor state
  const [activeEditor, setActiveEditor] = useState<string>('opening');
  const editorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadCalculationAndGenerateLetter();
  }, [calcId]);

  const loadCalculationAndGenerateLetter = async () => {
    try {
      setLoading(true);

      // Load settlement calculation
      const calculation = await invoke('cmd_get_settlement_calculation', { calcId });

      // Set default values
      setDemandAmount((calculation as any).recommended_demand);

      // Calculate deadline (30 days from now)
      const deadlineDate = new Date();
      deadlineDate.setDate(deadlineDate.getDate() + 30);
      setDeadline(deadlineDate.toISOString().split('T')[0]);

      // Generate default content
      setSubject(`Settlement Demand - Matter ${(calculation as any).matter_id}`);

      setOpening(
        `Dear ${recipientName || '[Recipient Name]'}:\n\n` +
        `This office represents ${(calculation as any).plaintiff_name} in connection with injuries sustained ` +
        `as a result of the negligence of ${(calculation as any).defendant_name}. ` +
        `We write to demand settlement of this claim.`
      );

      setFacts(
        `STATEMENT OF FACTS\n\n` +
        `On [DATE], ${(calculation as any).plaintiff_name} sustained serious injuries as a direct ` +
        `and proximate result of the negligence and carelessness of ${(calculation as any).defendant_name}. ` +
        `[Provide detailed factual narrative here.]`
      );

      setLiability(
        `LIABILITY\n\n` +
        `The liability of ${(calculation as any).defendant_name} is clear and well-established. ` +
        `[Detail basis for liability, including applicable law and supporting evidence.]`
      );

      setDamages(
        `DAMAGES\n\n` +
        `As a direct and proximate result of the aforementioned incident, ` +
        `${(calculation as any).plaintiff_name} has sustained the following damages:\n\n` +
        `Economic Damages:\n` +
        `- Past Medical Expenses: $${(calculation as any).economic_damages.past_medical_expenses.toLocaleString()}\n` +
        `- Future Medical Expenses: $${(calculation as any).economic_damages.future_medical_expenses.toLocaleString()}\n` +
        `- Past Lost Wages: $${(calculation as any).economic_damages.past_lost_wages.toLocaleString()}\n` +
        `- Future Lost Earning Capacity: $${(calculation as any).economic_damages.future_lost_earning_capacity.toLocaleString()}\n` +
        `- Total Economic Damages: $${(calculation as any).economic_damages.total_economic.toLocaleString()}\n\n` +
        `Non-Economic Damages:\n` +
        `- Pain and Suffering: $${(calculation as any).non_economic_damages.pain_and_suffering.toLocaleString()}\n` +
        `- Emotional Distress: $${(calculation as any).non_economic_damages.emotional_distress.toLocaleString()}\n` +
        `- Loss of Enjoyment of Life: $${(calculation as any).non_economic_damages.loss_of_enjoyment_of_life.toLocaleString()}\n` +
        `- Total Non-Economic Damages: $${(calculation as any).non_economic_damages.total_non_economic.toLocaleString()}\n\n` +
        `TOTAL DAMAGES: $${(calculation as any).total_damages.toLocaleString()}`
      );

      setClosing(
        `SETTLEMENT DEMAND\n\n` +
        `Based on the foregoing, we demand settlement in the amount of $${demandAmount.toLocaleString()}. ` +
        `This offer expires on ${new Date(deadline).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}. ` +
        `If we do not receive a satisfactory response by that date, we will proceed with litigation without further notice.\n\n` +
        `Please direct all communications regarding this matter to the undersigned.\n\n` +
        `Very truly yours,\n\n[Your Name]\n[Your Title]\n[Law Firm Name]`
      );

    } catch (err) {
      setError(err as string);
      console.error('Failed to load calculation:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      await invoke('cmd_generate_demand_letter', {
        calcId,
        recipientName,
        recipientAddress,
        facts,
        createdBy: '[Current User]', // Would come from auth
      });
      alert('Demand letter saved successfully!');
    } catch (err) {
      setError(err as string);
    } finally {
      setSaving(false);
    }
  };

  const handleExportPDF = async () => {
    try {
      const pdfPath = await invoke<string>('cmd_export_settlement_report', {
        calcId,
        format: 'pdf',
        outputPath: 'demand_letter.pdf',
      });
      alert(`PDF exported to: ${pdfPath}`);
    } catch (err) {
      alert(`Export failed: ${err}`);
    }
  };

  const handleSendEmail = () => {
    alert('Email functionality would integrate with email client here');
  };

  const addExhibit = () => {
    const nextLetter = String.fromCharCode(65 + exhibits.length); // A, B, C, etc.
    setExhibits([
      ...exhibits,
      {
        exhibit_letter: nextLetter,
        description: '',
        file_path: '',
      },
    ]);
  };

  const removeExhibit = (index: number) => {
    setExhibits(exhibits.filter((_, i) => i !== index));
  };

  const updateExhibit = (index: number, field: string, value: string) => {
    const updated = [...exhibits];
    updated[index] = { ...updated[index], [field]: value };
    setExhibits(updated);
  };

  const formatCurrency = (amount: number) =>
    new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
    }).format(amount);

  // Rich text formatting functions
  const execCommand = (command: string, value: string | undefined = undefined) => {
    document.execCommand(command, false, value);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-navy-600 mx-auto mb-4"></div>
          <p className="text-slate-600 text-lg">Loading editor...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header */}
      <div className="bg-gradient-to-r from-navy-900 to-navy-800 text-white shadow-xl">
        <div className="max-w-7xl mx-auto px-8 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <button
                type="button"
                onClick={() => navigate(`/settlement/analysis/${calcId}`)}
                className="p-2 hover:bg-navy-700 rounded-lg transition"
                aria-label="Back to analysis"
              >
                <ArrowLeft size={24} />
              </button>
              <div>
                <h1 className="text-3xl font-bold">Demand Letter Editor</h1>
                <p className="text-navy-200 mt-1">Professional settlement demand letter</p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={() => setShowPreview(!showPreview)}
                className="btn-secondary-white flex items-center gap-2"
              >
                <Eye size={18} />
                {showPreview ? 'Edit' : 'Preview'}
              </button>
              <button
                type="button"
                onClick={handleExportPDF}
                className="btn-secondary-white flex items-center gap-2"
              >
                <Download size={18} />
                Export PDF
              </button>
              <button
                type="button"
                onClick={handleSave}
                disabled={saving}
                className="btn-primary-gold flex items-center gap-2"
              >
                <Save size={18} />
                {saving ? 'Saving...' : 'Save Letter'}
              </button>
              <button
                type="button"
                onClick={handleSendEmail}
                className="btn-primary flex items-center gap-2"
              >
                <Send size={18} />
                Send
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-8 py-8">
        {error && (
          <div className="mb-6 bg-red-50 border-l-4 border-red-500 p-4 rounded">
            <p className="text-red-800">{error}</p>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Editor Side */}
          <div className="lg:col-span-2 space-y-6">
            {/* Recipient Information */}
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h2 className="text-xl font-bold text-navy-900 mb-6">Recipient Information</h2>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-semibold text-navy-900 mb-2">
                    Recipient Name <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="text"
                    value={recipientName}
                    onChange={(e) => setRecipientName(e.target.value)}
                    className="form-input"
                    placeholder="e.g., John Smith, Claims Adjuster"
                  />
                </div>

                <div>
                  <label className="block text-sm font-semibold text-navy-900 mb-2">
                    Settlement Demand Amount <span className="text-red-500">*</span>
                  </label>
                  <div className="relative">
                    <span className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 font-semibold">
                      $
                    </span>
                    <input
                      type="number"
                      value={demandAmount}
                      onChange={(e) => setDemandAmount(parseFloat(e.target.value) || 0)}
                      className="form-input pl-8"
                      placeholder="0.00"
                    />
                  </div>
                </div>

                <div className="md:col-span-2">
                  <label className="block text-sm font-semibold text-navy-900 mb-2">
                    Recipient Address <span className="text-red-500">*</span>
                  </label>
                  <textarea
                    value={recipientAddress}
                    onChange={(e) => setRecipientAddress(e.target.value)}
                    className="form-input"
                    rows={3}
                    placeholder="Street Address&#10;City, State ZIP"
                  />
                </div>

                <div className="md:col-span-2">
                  <label className="block text-sm font-semibold text-navy-900 mb-2">
                    Subject Line
                  </label>
                  <input
                    type="text"
                    value={subject}
                    onChange={(e) => setSubject(e.target.value)}
                    className="form-input"
                    placeholder="RE: Settlement Demand"
                  />
                </div>

                <div>
                  <label className="block text-sm font-semibold text-navy-900 mb-2">
                    Response Deadline <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="date"
                    value={deadline}
                    onChange={(e) => setDeadline(e.target.value)}
                    className="form-input"
                  />
                </div>
              </div>
            </div>

            {/* Letter Content Sections */}
            {!showPreview ? (
              <>
                {/* Opening Paragraph */}
                <EditorSection
                  title="Opening Paragraph"
                  content={opening}
                  onChange={setOpening}
                  placeholder="Dear [Recipient]: This office represents..."
                />

                {/* Facts Section */}
                <EditorSection
                  title="Statement of Facts"
                  content={facts}
                  onChange={setFacts}
                  placeholder="Provide a detailed narrative of the incident..."
                />

                {/* Liability Section */}
                <EditorSection
                  title="Liability"
                  content={liability}
                  onChange={setLiability}
                  placeholder="Explain the basis for defendant's liability..."
                />

                {/* Damages Section */}
                <EditorSection
                  title="Damages"
                  content={damages}
                  onChange={setDamages}
                  placeholder="Detail all economic and non-economic damages..."
                />

                {/* Closing Paragraph */}
                <EditorSection
                  title="Closing & Demand"
                  content={closing}
                  onChange={setClosing}
                  placeholder="Based on the foregoing, we demand..."
                />

                {/* Exhibits */}
                <div className="bg-white rounded-xl shadow-lg p-8">
                  <div className="flex items-center justify-between mb-6">
                    <h2 className="text-xl font-bold text-navy-900">Exhibits</h2>
                    <button
                      type="button"
                      onClick={addExhibit}
                      className="btn-secondary flex items-center gap-2"
                    >
                      <Plus size={16} />
                      Add Exhibit
                    </button>
                  </div>

                  {exhibits.length === 0 ? (
                    <p className="text-slate-500 text-center py-8">
                      No exhibits added yet. Click "Add Exhibit" to attach supporting documents.
                    </p>
                  ) : (
                    <div className="space-y-4">
                      {exhibits.map((exhibit, index) => (
                        <div
                          key={index}
                          className="p-4 bg-slate-50 rounded-lg border border-slate-200"
                        >
                          <div className="flex items-start justify-between mb-4">
                            <h3 className="font-semibold text-navy-900">
                              Exhibit {exhibit.exhibit_letter}
                            </h3>
                            <button
                              type="button"
                              onClick={() => removeExhibit(index)}
                              className="text-red-600 hover:text-red-700"
                              aria-label={`Remove exhibit ${exhibit.exhibit_letter}`}
                            >
                              <X size={20} />
                            </button>
                          </div>

                          <div className="grid grid-cols-1 gap-4">
                            <div>
                              <label className="block text-sm font-semibold text-navy-900 mb-2">
                                Description
                              </label>
                              <input
                                type="text"
                                value={exhibit.description}
                                onChange={(e) =>
                                  updateExhibit(index, 'description', e.target.value)
                                }
                                className="form-input"
                                placeholder="e.g., Medical Records from Dr. Smith"
                              />
                            </div>

                            <div>
                              <label className="block text-sm font-semibold text-navy-900 mb-2">
                                File Path
                              </label>
                              <input
                                type="text"
                                value={exhibit.file_path}
                                onChange={(e) =>
                                  updateExhibit(index, 'file_path', e.target.value)
                                }
                                className="form-input"
                                placeholder="/path/to/document.pdf"
                              />
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </>
            ) : (
              /* Preview Mode */
              <div className="bg-white rounded-xl shadow-2xl p-12">
                <div className="max-w-3xl mx-auto">
                  {/* Letterhead */}
                  <div className="text-center mb-12 pb-8 border-b-2 border-slate-200">
                    <h1 className="text-3xl font-bold text-navy-900 mb-2">
                      [YOUR LAW FIRM NAME]
                    </h1>
                    <p className="text-slate-600">
                      [Address] | [Phone] | [Email] | [Website]
                    </p>
                  </div>

                  {/* Date */}
                  <p className="mb-8 text-slate-700">
                    {new Date().toLocaleDateString('en-US', {
                      month: 'long',
                      day: 'numeric',
                      year: 'numeric',
                    })}
                  </p>

                  {/* Recipient */}
                  <div className="mb-8">
                    <p className="font-semibold text-navy-900">{recipientName}</p>
                    <p className="text-slate-700 whitespace-pre-line">{recipientAddress}</p>
                  </div>

                  {/* Subject */}
                  <p className="mb-8 font-semibold text-navy-900">RE: {subject}</p>

                  {/* Letter Body */}
                  <div className="space-y-6 text-slate-700 leading-relaxed">
                    <div className="whitespace-pre-line">{opening}</div>
                    <div className="whitespace-pre-line">{facts}</div>
                    <div className="whitespace-pre-line">{liability}</div>
                    <div className="whitespace-pre-line">{damages}</div>
                    <div className="whitespace-pre-line">{closing}</div>
                  </div>

                  {/* Exhibits List */}
                  {exhibits.length > 0 && (
                    <div className="mt-12 pt-8 border-t-2 border-slate-200">
                      <h3 className="font-bold text-navy-900 mb-4">Enclosures:</h3>
                      <ul className="list-disc list-inside space-y-2 text-slate-700">
                        {exhibits.map((exhibit) => (
                          <li key={exhibit.exhibit_letter}>
                            Exhibit {exhibit.exhibit_letter}: {exhibit.description}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Quick Actions */}
            <div className="bg-white rounded-xl shadow-lg p-6">
              <h3 className="text-lg font-bold text-navy-900 mb-4">Quick Actions</h3>
              <div className="space-y-3">
                <button
                  type="button"
                  onClick={handleSave}
                  className="w-full btn-secondary flex items-center justify-center gap-2"
                >
                  <Save size={18} />
                  Save Draft
                </button>
                <button
                  type="button"
                  onClick={handleExportPDF}
                  className="w-full btn-secondary flex items-center justify-center gap-2"
                >
                  <Download size={18} />
                  Export PDF
                </button>
                <button
                  type="button"
                  onClick={() => window.print()}
                  className="w-full btn-secondary flex items-center justify-center gap-2"
                >
                  <Printer size={18} />
                  Print
                </button>
                <button
                  type="button"
                  onClick={handleSendEmail}
                  className="w-full btn-primary flex items-center justify-center gap-2"
                >
                  <Mail size={18} />
                  Send via Email
                </button>
              </div>
            </div>

            {/* Templates */}
            <div className="bg-white rounded-xl shadow-lg p-6">
              <h3 className="text-lg font-bold text-navy-900 mb-4">Templates</h3>
              <div className="space-y-2">
                <button
                  type="button"
                  className="w-full text-left px-4 py-3 bg-slate-50 hover:bg-slate-100 rounded-lg transition"
                >
                  <p className="font-semibold text-navy-900 text-sm">Personal Injury Standard</p>
                  <p className="text-xs text-slate-600">Traditional format for PI cases</p>
                </button>
                <button
                  type="button"
                  className="w-full text-left px-4 py-3 bg-slate-50 hover:bg-slate-100 rounded-lg transition"
                >
                  <p className="font-semibold text-navy-900 text-sm">Medical Malpractice</p>
                  <p className="text-xs text-slate-600">Specialized for med mal cases</p>
                </button>
                <button
                  type="button"
                  className="w-full text-left px-4 py-3 bg-slate-50 hover:bg-slate-100 rounded-lg transition"
                >
                  <p className="font-semibold text-navy-900 text-sm">Employment Law</p>
                  <p className="text-xs text-slate-600">For employment disputes</p>
                </button>
              </div>
            </div>

            {/* Tips */}
            <div className="bg-blue-50 rounded-xl border-2 border-blue-200 p-6">
              <h3 className="text-lg font-bold text-blue-900 mb-4 flex items-center gap-2">
                <FileText size={20} />
                Writing Tips
              </h3>
              <ul className="space-y-2 text-sm text-blue-800">
                <li className="flex items-start gap-2">
                  <span className="text-blue-600 mt-1">•</span>
                  <span>Be professional and courteous in tone</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-600 mt-1">•</span>
                  <span>Cite specific evidence and medical records</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-600 mt-1">•</span>
                  <span>Set a reasonable deadline (30 days standard)</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-600 mt-1">•</span>
                  <span>Attach all supporting exhibits</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-600 mt-1">•</span>
                  <span>Proofread carefully before sending</span>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Editor Section Component
interface EditorSectionProps {
  title: string;
  content: string;
  onChange: (value: string) => void;
  placeholder: string;
}

function EditorSection({ title, content, onChange, placeholder }: EditorSectionProps) {
  return (
    <div className="bg-white rounded-xl shadow-lg p-8">
      <h2 className="text-xl font-bold text-navy-900 mb-4">{title}</h2>

      <textarea
        value={content}
        onChange={(e) => onChange(e.target.value)}
        className="w-full min-h-[200px] p-4 border-2 border-slate-200 rounded-lg focus:border-navy-500 focus:ring-2 focus:ring-navy-200 transition font-serif text-slate-700 leading-relaxed"
        placeholder={placeholder}
      />
    </div>
  );
}

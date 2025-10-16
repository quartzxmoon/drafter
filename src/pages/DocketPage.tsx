// Docket detail page for PA eDocket Desktop

import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import {
  FileText,
  Users,
  Calendar,
  DollarSign,
  Paperclip,
  Scale,
  Download,
  ExternalLink,
  AlertCircle,
  Clock,
  User,
  Building
} from 'lucide-react';
import { Docket, Party, Event, Filing, Financial, Attachment, Charge } from '../types/domain';
import { invoke } from '@tauri-apps/api/core';

type TabType = 'overview' | 'parties' | 'charges' | 'events' | 'filings' | 'financials' | 'attachments';

export const DocketPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [docket, setDocket] = useState<Docket | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [attachments, setAttachments] = useState<Attachment[]>([]);

  useEffect(() => {
    if (id) {
      loadDocket(id);
      loadAttachments(id);
    }
  }, [id]);

  const loadDocket = async (docketId: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<Docket>('cmd_get_docket', { id: docketId });
      setDocket(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load docket');
    } finally {
      setLoading(false);
    }
  };

  const loadAttachments = async (docketId: string) => {
    try {
      const result = await invoke<Attachment[]>('cmd_get_attachments', { docketId });
      setAttachments(result);
    } catch (err) {
      console.warn('Failed to load attachments:', err);
    }
  };

  const handleExport = async (format: 'json' | 'pdf') => {
    if (!docket) return;

    try {
      await invoke('cmd_export', {
        exportType: format,
        payload: format === 'json' ? docket : { docket_id: docket.id }
      });
    } catch (err) {
      console.error('Export failed:', err);
    }
  };

  const tabs = [
    { id: 'overview', label: 'Overview', icon: FileText },
    { id: 'parties', label: 'Parties', icon: Users, count: docket?.parties.length },
    { id: 'charges', label: 'Charges', icon: Scale, count: docket?.charges.length },
    { id: 'events', label: 'Events', icon: Calendar, count: docket?.events.length },
    { id: 'filings', label: 'Filings', icon: FileText, count: docket?.filings.length },
    { id: 'financials', label: 'Financials', icon: DollarSign, count: docket?.financials.length },
    { id: 'attachments', label: 'Attachments', icon: Paperclip, count: attachments.length },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600">Loading docket...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-6">
        <div className="flex items-center">
          <AlertCircle className="h-5 w-5 text-red-500 mr-2" />
          <h3 className="text-red-800 font-medium">Error Loading Docket</h3>
        </div>
        <p className="text-red-700 mt-2">{error}</p>
        <button
          onClick={() => id && loadDocket(id)}
          className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!docket) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
        <div className="flex items-center">
          <AlertCircle className="h-5 w-5 text-yellow-500 mr-2" />
          <h3 className="text-yellow-800 font-medium">Docket Not Found</h3>
        </div>
        <p className="text-yellow-700 mt-2">The requested docket could not be found.</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">{docket.caption}</h1>
              <p className="text-gray-600 mt-1">Docket No. {docket.docketNumber}</p>
            </div>
            <div className="flex space-x-2">
              <button
                onClick={() => handleExport('json')}
                className="flex items-center px-3 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
              >
                <Download className="h-4 w-4 mr-2" />
                Export JSON
              </button>
              <button
                onClick={() => handleExport('pdf')}
                className="flex items-center px-3 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
              >
                <Download className="h-4 w-4 mr-2" />
                Export PDF
              </button>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8 px-6">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              const isActive = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as TabType)}
                  className={`py-4 px-1 border-b-2 font-medium text-sm flex items-center ${
                    isActive
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  <Icon className="h-4 w-4 mr-2" />
                  {tab.label}
                  {tab.count !== undefined && tab.count > 0 && (
                    <span className={`ml-2 py-0.5 px-2 rounded-full text-xs ${
                      isActive ? 'bg-blue-100 text-blue-600' : 'bg-gray-100 text-gray-600'
                    }`}>
                      {tab.count}
                    </span>
                  )}
                </button>
              );
            })}
          </nav>
        </div>
      </div>

      {/* Tab Content */}
      <div className="bg-white rounded-lg shadow">
        {activeTab === 'overview' && <OverviewTab docket={docket} />}
        {activeTab === 'parties' && <PartiesTab parties={docket.parties} />}
        {activeTab === 'charges' && <ChargesTab charges={docket.charges} />}
        {activeTab === 'events' && <EventsTab events={docket.events} />}
        {activeTab === 'filings' && <FilingsTab filings={docket.filings} />}
        {activeTab === 'financials' && <FinancialsTab financials={docket.financials} />}
        {activeTab === 'attachments' && <AttachmentsTab attachments={attachments} />}
      </div>
    </div>
  );
};

// Tab Components
const OverviewTab: React.FC<{ docket: Docket }> = ({ docket }) => (
  <div className="p-6">
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-4">Case Information</h3>
        <dl className="space-y-3">
          <div>
            <dt className="text-sm font-medium text-gray-500">Docket Number</dt>
            <dd className="text-sm text-gray-900">{docket.docketNumber}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Court</dt>
            <dd className="text-sm text-gray-900">{docket.court}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Status</dt>
            <dd className="text-sm text-gray-900">
              <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                docket.status === 'Active' ? 'bg-green-100 text-green-800' :
                docket.status === 'Closed' ? 'bg-gray-100 text-gray-800' :
                'bg-yellow-100 text-yellow-800'
              }`}>
                {docket.status}
              </span>
            </dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Filed Date</dt>
            <dd className="text-sm text-gray-900">{new Date(docket.filed).toLocaleDateString()}</dd>
          </div>
          {docket.judge && (
            <div>
              <dt className="text-sm font-medium text-gray-500">Judge</dt>
              <dd className="text-sm text-gray-900">{docket.judge}</dd>
            </div>
          )}
        </dl>
      </div>

      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-4">Summary</h3>
        <dl className="space-y-3">
          <div>
            <dt className="text-sm font-medium text-gray-500">Parties</dt>
            <dd className="text-sm text-gray-900">{docket.parties.length}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Charges</dt>
            <dd className="text-sm text-gray-900">{docket.charges.length}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Events</dt>
            <dd className="text-sm text-gray-900">{docket.events.length}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Filings</dt>
            <dd className="text-sm text-gray-900">{docket.filings.length}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Last Updated</dt>
            <dd className="text-sm text-gray-900">{docket.lastUpdated ? new Date(docket.lastUpdated).toLocaleString() : 'Unknown'}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-500">Source</dt>
            <dd className="text-sm text-gray-900">{docket.source}</dd>
          </div>
        </dl>
      </div>
    </div>
  </div>
);

const PartiesTab: React.FC<{ parties: Party[] }> = ({ parties }) => (
  <div className="p-6">
    <h3 className="text-lg font-medium text-gray-900 mb-4">Parties</h3>
    {parties.length === 0 ? (
      <p className="text-gray-500">No parties found.</p>
    ) : (
      <div className="space-y-4">
        {parties.map((party, index) => (
          <div key={index} className="border border-gray-200 rounded-lg p-4">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center">
                  <User className="h-5 w-5 text-gray-400 mr-2" />
                  <h4 className="text-lg font-medium text-gray-900">{party.name}</h4>
                  <span className={`ml-3 inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                    party.role === 'Plaintiff' ? 'bg-blue-100 text-blue-800' :
                    party.role === 'Defendant' ? 'bg-red-100 text-red-800' :
                    party.role === 'Petitioner' ? 'bg-green-100 text-green-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {party.role}
                  </span>
                </div>

                <dl className="mt-3 grid grid-cols-1 md:grid-cols-2 gap-4">
                  {party.address && (
                    <>
                      <dt className="text-sm font-medium text-gray-500">Address</dt>
                      <dd className="text-sm text-gray-900">
                        {party.address}
                        {party.city ? `, ${party.city}` : ''}
                        {party.state ? `, ${party.state}` : ''}
                        {party.zipCode ? ` ${party.zipCode}` : ''}
                      </dd>
                    </>
                  )}
                  {party.phone && (
                    <>
                      <dt className="text-sm font-medium text-gray-500">Phone</dt>
                      <dd className="text-sm text-gray-900">{party.phone}</dd>
                    </>
                  )}
                  {party.email && (
                    <>
                      <dt className="text-sm font-medium text-gray-500">Email</dt>
                      <dd className="text-sm text-gray-900">{party.email}</dd>
                    </>
                  )}
                  {party.attorney && (
                    <>
                      <dt className="text-sm font-medium text-gray-500">Attorney</dt>
                      <dd className="text-sm text-gray-900">
                        {party.attorney}
                        {party.attorneyId && ` (ID: ${party.attorneyId})`}
                        {party.attorneyPhone && (
                          <>
                            <br />
                            {party.attorneyPhone}
                          </>
                        )}
                        {party.attorneyEmail && (
                          <>
                            <br />
                            {party.attorneyEmail}
                          </>
                        )}
                      </dd>
                    </>
                  )}
                </dl>
               </div>
             </div>
           </div>
         ))}
       </div>
     )}
   </div>
);

const ChargesTab: React.FC<{ charges: Charge[] }> = ({ charges }) => (
  <div className="p-6">
    <h3 className="text-lg font-medium text-gray-900 mb-4">Charges</h3>
    {charges.length === 0 ? (
      <p className="text-gray-500">No charges found.</p>
    ) : (
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Sequence
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Description
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Grade
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Disposition
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Disposition Date
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {charges.map((charge, index) => (
              <tr key={index}>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {charge.sequence}
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  <div>
                    <div className="font-medium">{charge.description}</div>
                    {charge.statute && (
                      <div className="text-gray-500">{charge.statute}</div>
                    )}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {charge.grade}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {charge.disposition || 'Pending'}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {charge.dispositionDate ? new Date(charge.dispositionDate).toLocaleDateString() : '-'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    )}
  </div>
);

const EventsTab: React.FC<{ events: Event[] }> = ({ events }) => (
  <div className="p-6">
    <h3 className="text-lg font-medium text-gray-900 mb-4">Docket Events</h3>
    {events.length === 0 ? (
      <p className="text-gray-500">No events found.</p>
    ) : (
      <div className="space-y-4">
        {events.map((event, index) => (
          <div key={index} className="border border-gray-200 rounded-lg p-4">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                <Calendar className="h-5 w-5 text-gray-400 mt-1" />
              </div>
              <div className="ml-3 flex-1">
                <div className="flex items-center justify-between">
                  <h4 className="text-sm font-medium text-gray-900">
                    {new Date(event.date).toLocaleDateString()}
                  </h4>
                  {event.time && (
                    <span className="text-sm text-gray-500">
                      <Clock className="h-4 w-4 inline mr-1" />
                      {event.time}
                    </span>
                  )}
                </div>
                <p className="mt-1 text-sm text-gray-700">{event.description}</p>
                {event.judge && (
                  <p className="mt-1 text-sm text-gray-500">
                    <User className="h-4 w-4 inline mr-1" />
                    Judge: {event.judge}
                  </p>
                )}
                {event.courtroom && (
                  <p className="mt-1 text-sm text-gray-500">
                    <Building className="h-4 w-4 inline mr-1" />
                    Courtroom: {event.courtroom}
                  </p>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    )}
  </div>
);

const FilingsTab: React.FC<{ filings: Filing[] }> = ({ filings }) => (
  <div className="p-6">
    <h3 className="text-lg font-medium text-gray-900 mb-4">Filings</h3>
    {filings.length === 0 ? (
      <p className="text-gray-500">No filings found.</p>
    ) : (
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Date Filed
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Document
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Filed By
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Status
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {filings.map((filing, index) => (
              <tr key={index}>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {new Date(filing.filed_date).toLocaleDateString()}
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  <div>
                    <div className="font-medium">{filing.document_title}</div>
                    {filing.document_type && (
                      <div className="text-gray-500">{filing.document_type}</div>
                    )}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {filing.filed_by}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                    filing.status === 'Filed' ? 'bg-green-100 text-green-800' :
                    filing.status === 'Pending' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {filing.status}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                  {filing.document_url && (
                    <a
                      href={filing.document_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-600 hover:text-blue-900"
                    >
                      <ExternalLink className="h-4 w-4 inline mr-1" />
                      View
                    </a>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    )}
  </div>
);

const FinancialsTab: React.FC<{ financials: Financial[] }> = ({ financials }) => (
  <div className="p-6">
    <h3 className="text-lg font-medium text-gray-900 mb-4">Financial Information</h3>
    {financials.length === 0 ? (
      <p className="text-gray-500">No financial information found.</p>
    ) : (
      <div className="space-y-6">
        {financials.map((financial, index) => (
          <div key={index} className="border border-gray-200 rounded-lg p-4">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h4 className="text-lg font-medium text-gray-900 flex items-center">
                  <DollarSign className="h-5 w-5 text-gray-400 mr-2" />
                  {financial.description}
                </h4>

                <div className="mt-3 grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div>
                    <dl>
                      <dt className="text-sm font-medium text-gray-500">Amount Assessed</dt>
                      <dd className="text-sm text-gray-900 font-medium">
                        ${financial.amount_assessed.toFixed(2)}
                      </dd>
                    </dl>
                  </div>

                  <div>
                    <dl>
                      <dt className="text-sm font-medium text-gray-500">Amount Paid</dt>
                      <dd className="text-sm text-gray-900 font-medium">
                        ${financial.amount_paid.toFixed(2)}
                      </dd>
                    </dl>
                  </div>

                  <div>
                    <dl>
                      <dt className="text-sm font-medium text-gray-500">Balance Due</dt>
                      <dd className={`text-sm font-medium ${
                        financial.balance_due > 0 ? 'text-red-600' : 'text-green-600'
                      }`}>
                        ${financial.balance_due.toFixed(2)}
                      </dd>
                    </dl>
                  </div>

                  {financial.dueDate && (
                    <div>
                      <dl>
                        <dt className="text-sm font-medium text-gray-500">Due Date</dt>
                        <dd className="text-sm text-gray-900">
                          {new Date(financial.dueDate).toLocaleDateString()}
                        </dd>
                      </dl>
                    </div>
                  )}

                  {financial.payment_plan && (
                    <div>
                      <dl>
                        <dt className="text-sm font-medium text-gray-500">Payment Plan</dt>
                        <dd className="text-sm text-gray-900">Yes</dd>
                      </dl>
                    </div>
                  )}

                  <div>
                    <dl>
                      <dt className="text-sm font-medium text-gray-500">Status</dt>
                      <dd className="text-sm text-gray-900">
                        <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                          financial.balance_due === 0 ? 'bg-green-100 text-green-800' :
                          financial.balance_due > 0 ? 'bg-red-100 text-red-800' :
                          'bg-gray-100 text-gray-800'
                        }`}>
                          {financial.balance_due === 0 ? 'Paid' : 'Outstanding'}
                        </span>
                      </dd>
                    </dl>
                  </div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    )}
  </div>
);

const AttachmentsTab: React.FC<{ attachments: Attachment[] }> = ({ attachments }) => (
  <div className="p-6">
    <h3 className="text-lg font-medium text-gray-900 mb-4">Attachments</h3>
    {attachments.length === 0 ? (
      <p className="text-gray-500">No attachments found.</p>
    ) : (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {attachments.map((attachment, index) => (
          <div key={index} className="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                <Paperclip className="h-5 w-5 text-gray-400" />
              </div>
              <div className="ml-3 flex-1 min-w-0">
                <h4 className="text-sm font-medium text-gray-900 truncate">
                  {attachment.filename}
                </h4>
                <p className="text-sm text-gray-500 mt-1">
                  {attachment.description}
                </p>

                <div className="mt-2 space-y-1">
                  <div className="text-xs text-gray-500">
                    Size: {formatFileSize(attachment.file_size)}
                  </div>
                  <div className="text-xs text-gray-500">
                    Type: {attachment.file_type}
                  </div>
                  <div className="text-xs text-gray-500">
                    Uploaded: {attachment.uploadDate ? new Date(attachment.uploadDate).toLocaleDateString() : 'Unknown'}
                  </div>
                </div>

                <div className="mt-3 flex space-x-2">
                  {attachment.download_url && (
                    <a
                      href={attachment.download_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center px-2 py-1 border border-transparent text-xs font-medium rounded text-blue-700 bg-blue-100 hover:bg-blue-200"
                    >
                      <Download className="h-3 w-3 mr-1" />
                      Download
                    </a>
                  )}
                  {attachment.view_url && (
                    <a
                      href={attachment.view_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center px-2 py-1 border border-transparent text-xs font-medium rounded text-gray-700 bg-gray-100 hover:bg-gray-200"
                    >
                      <ExternalLink className="h-3 w-3 mr-1" />
                      View
                    </a>
                  )}
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    )}
  </div>
);

// Helper function
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

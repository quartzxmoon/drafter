// Negotiation Timeline Component
// Interactive timeline visualization for settlement offer/counteroffer tracking

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  TrendingUp,
  TrendingDown,
  Minus,
  CheckCircle,
  XCircle,
  Clock,
  Send,
  ArrowRight,
  DollarSign,
  Calendar,
  User,
  MessageSquare,
  AlertCircle,
} from 'lucide-react';

interface OfferTimelineEvent {
  id: string;
  type: 'offer' | 'counteroffer' | 'rejection' | 'acceptance';
  from: 'Plaintiff' | 'Defendant';
  amount: number;
  date: string;
  status: 'Pending' | 'Accepted' | 'Rejected' | 'Countered' | 'Expired';
  response?: string;
  terms?: string[];
  round: number;
}

interface NegotiationTimelineProps {
  calcId: string;
  events: OfferTimelineEvent[];
  onRecordOffer: () => void;
  onGenerateCounter: () => void;
}

export default function NegotiationTimeline({
  calcId,
  events,
  onRecordOffer,
  onGenerateCounter,
}: NegotiationTimelineProps) {
  const [selectedEvent, setSelectedEvent] = useState<OfferTimelineEvent | null>(null);
  const [showDetails, setShowDetails] = useState(false);

  const formatCurrency = (amount: number) =>
    new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
    }).format(amount);

  const formatDate = (dateString: string) =>
    new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });

  const getEventIcon = (type: string, status: string) => {
    if (status === 'Accepted') return <CheckCircle className="text-green-600" size={24} />;
    if (status === 'Rejected') return <XCircle className="text-red-600" size={24} />;
    if (status === 'Expired') return <Clock className="text-slate-400" size={24} />;

    switch (type) {
      case 'offer':
        return <Send className="text-blue-600" size={24} />;
      case 'counteroffer':
        return <ArrowRight className="text-amber-600" size={24} />;
      default:
        return <MessageSquare className="text-slate-600" size={24} />;
    }
  };

  const getEventColor = (from: string, status: string) => {
    if (status === 'Accepted') return 'green';
    if (status === 'Rejected') return 'red';
    if (status === 'Expired') return 'slate';
    return from === 'Plaintiff' ? 'blue' : 'amber';
  };

  const getTrendIcon = (currentAmount: number, previousAmount: number) => {
    if (currentAmount > previousAmount) {
      return <TrendingUp className="text-green-600" size={16} />;
    } else if (currentAmount < previousAmount) {
      return <TrendingDown className="text-red-600" size={16} />;
    } else {
      return <Minus className="text-slate-400" size={16} />;
    }
  };

  const calculateGap = (index: number) => {
    if (index === 0) return null;
    const current = events[index];
    const previous = events[index - 1];
    const gap = current.amount - previous.amount;
    const percentage = ((gap / previous.amount) * 100).toFixed(1);
    return { gap, percentage };
  };

  return (
    <div className="bg-white rounded-xl shadow-lg p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-2xl font-bold text-navy-900 mb-2">Negotiation Timeline</h2>
          <p className="text-slate-600">
            {events.length} offer{events.length !== 1 ? 's' : ''} •{' '}
            {events.filter((e) => e.status === 'Pending').length} pending
          </p>
        </div>

        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={onRecordOffer}
            className="btn-secondary flex items-center gap-2"
          >
            <Send size={18} />
            Record Offer
          </button>
          <button
            type="button"
            onClick={onGenerateCounter}
            className="btn-primary flex items-center gap-2"
          >
            <ArrowRight size={18} />
            Generate Counter
          </button>
        </div>
      </div>

      {events.length === 0 ? (
        <div className="text-center py-16">
          <div className="inline-flex items-center justify-center w-16 h-16 bg-slate-100 rounded-full mb-4">
            <MessageSquare className="text-slate-400" size={32} />
          </div>
          <h3 className="text-xl font-semibold text-slate-700 mb-2">No Offers Yet</h3>
          <p className="text-slate-500 mb-6">
            Start tracking settlement negotiations by recording the first offer.
          </p>
          <button
            type="button"
            onClick={onRecordOffer}
            className="btn-primary flex items-center gap-2 mx-auto"
          >
            <Send size={18} />
            Record First Offer
          </button>
        </div>
      ) : (
        <div className="space-y-6">
          {/* Timeline Visualization */}
          <div className="relative">
            {/* Vertical Line */}
            <div className="absolute left-8 top-0 bottom-0 w-0.5 bg-slate-200" />

            {/* Timeline Events */}
            <div className="space-y-8">
              {events.map((event, index) => {
                const color = getEventColor(event.from, event.status);
                const gap = calculateGap(index);

                return (
                  <div key={event.id} className="relative">
                    {/* Timeline Dot */}
                    <div className={`absolute left-0 w-16 h-16 bg-${color}-100 border-4 border-${color}-500 rounded-full flex items-center justify-center z-10`}>
                      {getEventIcon(event.type, event.status)}
                    </div>

                    {/* Event Card */}
                    <div className="ml-24 relative">
                      <div
                        className={`bg-${color}-50 border-2 border-${color}-200 rounded-xl p-6 hover:shadow-lg transition cursor-pointer`}
                        onClick={() => {
                          setSelectedEvent(event);
                          setShowDetails(true);
                        }}
                      >
                        {/* Event Header */}
                        <div className="flex items-start justify-between mb-4">
                          <div>
                            <div className="flex items-center gap-3 mb-2">
                              <span className={`px-3 py-1 bg-${color}-200 text-${color}-900 rounded-full text-sm font-bold`}>
                                Round {event.round}
                              </span>
                              <span className={`px-3 py-1 bg-white border border-${color}-300 text-${color}-900 rounded-full text-sm font-semibold`}>
                                {event.status}
                              </span>
                            </div>
                            <h3 className={`text-xl font-bold text-${color}-900`}>
                              {event.type === 'offer' ? 'Offer' : 'Counter-Offer'} from{' '}
                              {event.from}
                            </h3>
                          </div>

                          <div className="text-right">
                            <p className="text-sm text-slate-600 mb-1 flex items-center gap-2">
                              <Calendar size={14} />
                              {formatDate(event.date)}
                            </p>
                            {gap && (
                              <div className="flex items-center gap-2 text-sm">
                                {getTrendIcon(event.amount, events[index - 1].amount)}
                                <span className={`font-semibold ${gap.gap > 0 ? 'text-green-600' : 'text-red-600'}`}>
                                  {gap.gap > 0 ? '+' : ''}
                                  {formatCurrency(gap.gap)} ({gap.percentage}%)
                                </span>
                              </div>
                            )}
                          </div>
                        </div>

                        {/* Amount */}
                        <div className="mb-4">
                          <div className="flex items-baseline gap-3">
                            <DollarSign className={`text-${color}-600`} size={32} />
                            <span className={`text-4xl font-bold text-${color}-900`}>
                              {formatCurrency(event.amount)}
                            </span>
                          </div>
                        </div>

                        {/* Response */}
                        {event.response && (
                          <div className="mt-4 p-4 bg-white rounded-lg border border-slate-200">
                            <p className="text-sm font-semibold text-slate-700 mb-1">Response:</p>
                            <p className="text-slate-600 text-sm">{event.response}</p>
                          </div>
                        )}

                        {/* Terms */}
                        {event.terms && event.terms.length > 0 && (
                          <div className="mt-4">
                            <p className="text-sm font-semibold text-slate-700 mb-2">Terms:</p>
                            <div className="flex flex-wrap gap-2">
                              {event.terms.map((term, i) => (
                                <span
                                  key={i}
                                  className="px-3 py-1 bg-white border border-slate-300 rounded-full text-xs text-slate-700"
                                >
                                  {term}
                                </span>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>

                      {/* Arrow connecting to next event */}
                      {index < events.length - 1 && (
                        <div className="absolute left-1/2 -bottom-4 transform -translate-x-1/2">
                          <ArrowRight className="text-slate-300" size={24} />
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Summary Statistics */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 pt-8 border-t-2 border-slate-200">
            <StatCard
              label="Total Rounds"
              value={events.length.toString()}
              icon={<MessageSquare size={20} />}
            />
            <StatCard
              label="First Offer"
              value={formatCurrency(events[0]?.amount || 0)}
              icon={<Send size={20} />}
            />
            <StatCard
              label="Latest Offer"
              value={formatCurrency(events[events.length - 1]?.amount || 0)}
              icon={<ArrowRight size={20} />}
            />
            <StatCard
              label="Total Movement"
              value={formatCurrency(
                (events[events.length - 1]?.amount || 0) - (events[0]?.amount || 0)
              )}
              icon={<TrendingUp size={20} />}
            />
          </div>
        </div>
      )}

      {/* Event Details Modal */}
      {showDetails && selectedEvent && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
          onClick={() => setShowDetails(false)}
        >
          <div
            className="bg-white rounded-xl shadow-2xl max-w-2xl w-full p-8"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-2xl font-bold text-navy-900">Offer Details</h3>
              <button
                type="button"
                onClick={() => setShowDetails(false)}
                className="text-slate-400 hover:text-slate-600"
                aria-label="Close details"
              >
                <XCircle size={28} />
              </button>
            </div>

            <div className="space-y-6">
              <DetailRow label="Round" value={`Round ${selectedEvent.round}`} />
              <DetailRow label="Type" value={selectedEvent.type} />
              <DetailRow label="From" value={selectedEvent.from} />
              <DetailRow label="Amount" value={formatCurrency(selectedEvent.amount)} />
              <DetailRow label="Date" value={formatDate(selectedEvent.date)} />
              <DetailRow label="Status" value={selectedEvent.status} />

              {selectedEvent.response && (
                <div>
                  <p className="text-sm font-semibold text-slate-700 mb-2">Response:</p>
                  <p className="text-slate-600 p-4 bg-slate-50 rounded-lg">
                    {selectedEvent.response}
                  </p>
                </div>
              )}

              {selectedEvent.terms && selectedEvent.terms.length > 0 && (
                <div>
                  <p className="text-sm font-semibold text-slate-700 mb-2">Terms:</p>
                  <ul className="list-disc list-inside space-y-1 text-slate-600">
                    {selectedEvent.terms.map((term, i) => (
                      <li key={i}>{term}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>

            <div className="mt-8 flex gap-3">
              <button
                type="button"
                onClick={() => setShowDetails(false)}
                className="flex-1 btn-secondary"
              >
                Close
              </button>
              {selectedEvent.status === 'Pending' && (
                <button
                  type="button"
                  onClick={onGenerateCounter}
                  className="flex-1 btn-primary"
                >
                  Generate Counter-Offer
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// Helper Components
interface StatCardProps {
  label: string;
  value: string;
  icon: React.ReactNode;
}

function StatCard({ label, value, icon }: StatCardProps) {
  return (
    <div className="p-4 bg-slate-50 rounded-lg border border-slate-200">
      <div className="flex items-center gap-2 mb-2 text-slate-600">
        {icon}
        <span className="text-sm font-medium">{label}</span>
      </div>
      <p className="text-2xl font-bold text-navy-900">{value}</p>
    </div>
  );
}

interface DetailRowProps {
  label: string;
  value: string;
}

function DetailRow({ label, value }: DetailRowProps) {
  return (
    <div className="flex justify-between items-center py-3 border-b border-slate-200">
      <span className="font-semibold text-slate-700">{label}:</span>
      <span className="text-navy-900">{value}</span>
    </div>
  );
}

// Settlement Dashboard - Executive Overview Page
// Provides high-level analytics and access to settlement calculator tools

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  BarChart3,
  TrendingUp,
  DollarSign,
  FileText,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Calculator,
  PlusCircle,
} from 'lucide-react';
import { Link } from 'react-router-dom';

interface DashboardStats {
  total_calculations: number;
  active_negotiations: number;
  total_settlement_value: number;
  average_settlement_ratio: number;
  offers_pending: number;
  offers_accepted: number;
  offers_rejected: number;
  average_negotiation_rounds: number;
  total_estimated_fees: number;
  total_net_to_client: number;
}

interface CaseTypeDistribution {
  case_type: string;
  count: number;
  total_value: number;
  average_value: number;
}

interface JurisdictionStats {
  jurisdiction: string;
  total_cases: number;
  average_settlement: number;
  median_settlement: number;
  success_rate: number;
}

export default function SettlementDashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [caseDistribution, setCaseDistribution] = useState<CaseTypeDistribution[]>([]);
  const [jurisdictionStats, setJurisdictionStats] = useState<JurisdictionStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    try {
      setLoading(true);

      const [dashboardStats, caseTypes, jurisdictions] = await Promise.all([
        invoke<DashboardStats>('cmd_get_settlement_dashboard_stats', {
          matterId: null,
          dateFrom: null,
          dateTo: null,
        }),
        invoke<CaseTypeDistribution[]>('cmd_get_case_type_distribution', {
          dateFrom: null,
          dateTo: null,
        }),
        invoke<JurisdictionStats[]>('cmd_get_jurisdiction_statistics'),
      ]);

      setStats(dashboardStats);
      setCaseDistribution(caseTypes);
      setJurisdictionStats(jurisdictions);
    } catch (err) {
      setError(err as string);
      console.error('Failed to load dashboard data:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  const formatPercentage = (value: number) => {
    return `${(value * 100).toFixed(1)}%`;
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-navy-600 mx-auto mb-4"></div>
          <p className="text-slate-600 text-lg">Loading dashboard...</p>
        </div>
      </div>
    );
  }

  if (error || !stats) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-8">
        <div className="max-w-2xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <h3 className="text-red-800 font-semibold text-lg mb-2">Error Loading Dashboard</h3>
            <p className="text-red-600">{error || 'Unknown error occurred'}</p>
            <button
              onClick={loadDashboardData}
              className="mt-4 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header */}
      <div className="bg-gradient-to-r from-navy-900 to-navy-800 text-white shadow-xl">
        <div className="max-w-7xl mx-auto px-8 py-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-4xl font-bold mb-2">Settlement Calculator Dashboard</h1>
              <p className="text-navy-200 text-lg">
                Comprehensive settlement analysis and negotiation tracking
              </p>
            </div>
            <Link
              to="/settlement/new"
              className="flex items-center gap-2 px-6 py-3 bg-gold-500 hover:bg-gold-600 text-navy-900 font-semibold rounded-lg transition-all shadow-lg hover:shadow-xl transform hover:scale-105"
            >
              <PlusCircle size={20} />
              New Calculation
            </Link>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-8 py-8">
        {/* Key Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {/* Total Calculations */}
          <MetricCard
            icon={<Calculator className="text-navy-600" size={28} />}
            title="Total Calculations"
            value={stats.total_calculations.toString()}
            subtitle="Settlement analyses"
            color="navy"
          />

          {/* Active Negotiations */}
          <MetricCard
            icon={<Clock className="text-amber-600" size={28} />}
            title="Active Negotiations"
            value={stats.active_negotiations.toString()}
            subtitle="Ongoing settlement talks"
            color="amber"
          />

          {/* Total Portfolio Value */}
          <MetricCard
            icon={<DollarSign className="text-green-600" size={28} />}
            title="Total Portfolio Value"
            value={formatCurrency(stats.total_settlement_value)}
            subtitle="Combined settlement value"
            color="green"
          />

          {/* Average Settlement Ratio */}
          <MetricCard
            icon={<TrendingUp className="text-blue-600" size={28} />}
            title="Avg Settlement Ratio"
            value={formatPercentage(stats.average_settlement_ratio)}
            subtitle="Of calculated value"
            color="blue"
          />
        </div>

        {/* Negotiation Status Row */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <StatusCard
            icon={<AlertCircle className="text-yellow-600" size={24} />}
            title="Pending Offers"
            value={stats.offers_pending}
            color="yellow"
          />
          <StatusCard
            icon={<CheckCircle className="text-green-600" size={24} />}
            title="Accepted Offers"
            value={stats.offers_accepted}
            color="green"
          />
          <StatusCard
            icon={<XCircle className="text-red-600" size={24} />}
            title="Rejected Offers"
            value={stats.offers_rejected}
            color="red"
          />
        </div>

        {/* Financial Summary */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <FinancialCard
            title="Avg Negotiation Rounds"
            value={stats.average_negotiation_rounds.toFixed(1)}
            description="Typical rounds to settlement"
          />
          <FinancialCard
            title="Total Estimated Fees"
            value={formatCurrency(stats.total_estimated_fees)}
            description="Attorney fees across portfolio"
          />
          <FinancialCard
            title="Total Net to Clients"
            value={formatCurrency(stats.total_net_to_client)}
            description="Net recovery after fees"
          />
        </div>

        {/* Case Type Distribution */}
        {caseDistribution.length > 0 && (
          <div className="bg-white rounded-xl shadow-lg p-8 mb-8">
            <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
              <BarChart3 className="text-navy-600" size={28} />
              Case Type Distribution
            </h2>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50 border-b-2 border-slate-200">
                  <tr>
                    <th className="px-6 py-4 text-left text-sm font-semibold text-slate-700">
                      Case Type
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Count
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Total Value
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Avg Value
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {caseDistribution.map((caseType, index) => (
                    <tr key={index} className="hover:bg-slate-50 transition">
                      <td className="px-6 py-4 text-navy-900 font-medium">
                        {caseType.case_type}
                      </td>
                      <td className="px-6 py-4 text-right text-slate-700">
                        {caseType.count}
                      </td>
                      <td className="px-6 py-4 text-right text-slate-700 font-mono">
                        {formatCurrency(caseType.total_value)}
                      </td>
                      <td className="px-6 py-4 text-right text-slate-700 font-mono">
                        {formatCurrency(caseType.average_value)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* Jurisdiction Statistics */}
        {jurisdictionStats.length > 0 && (
          <div className="bg-white rounded-xl shadow-lg p-8">
            <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
              <FileText className="text-navy-600" size={28} />
              Jurisdiction Statistics
            </h2>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50 border-b-2 border-slate-200">
                  <tr>
                    <th className="px-6 py-4 text-left text-sm font-semibold text-slate-700">
                      Jurisdiction
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Total Cases
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Avg Settlement
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Median Settlement
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Success Rate
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {jurisdictionStats.map((jurisdiction, index) => (
                    <tr key={index} className="hover:bg-slate-50 transition">
                      <td className="px-6 py-4 text-navy-900 font-medium">
                        {jurisdiction.jurisdiction}
                      </td>
                      <td className="px-6 py-4 text-right text-slate-700">
                        {jurisdiction.total_cases}
                      </td>
                      <td className="px-6 py-4 text-right text-slate-700 font-mono">
                        {formatCurrency(jurisdiction.average_settlement)}
                      </td>
                      <td className="px-6 py-4 text-right text-slate-700 font-mono">
                        {formatCurrency(jurisdiction.median_settlement)}
                      </td>
                      <td className="px-6 py-4 text-right">
                        <span
                          className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                            jurisdiction.success_rate >= 0.7
                              ? 'bg-green-100 text-green-800'
                              : jurisdiction.success_rate >= 0.5
                              ? 'bg-yellow-100 text-yellow-800'
                              : 'bg-red-100 text-red-800'
                          }`}
                        >
                          {formatPercentage(jurisdiction.success_rate)}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ============= Component Library =============

interface MetricCardProps {
  icon: React.ReactNode;
  title: string;
  value: string;
  subtitle: string;
  color: 'navy' | 'amber' | 'green' | 'blue';
}

function MetricCard({ icon, title, value, subtitle, color }: MetricCardProps) {
  const colorClasses = {
    navy: 'from-navy-500 to-navy-700',
    amber: 'from-amber-500 to-amber-700',
    green: 'from-green-500 to-green-700',
    blue: 'from-blue-500 to-blue-700',
  };

  return (
    <div className="bg-white rounded-xl shadow-lg p-6 border-t-4 border-navy-600 hover:shadow-xl transition-shadow">
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 bg-slate-50 rounded-lg">{icon}</div>
      </div>
      <h3 className="text-slate-600 text-sm font-medium mb-2">{title}</h3>
      <p className="text-3xl font-bold text-navy-900 mb-1">{value}</p>
      <p className="text-slate-500 text-sm">{subtitle}</p>
    </div>
  );
}

interface StatusCardProps {
  icon: React.ReactNode;
  title: string;
  value: number;
  color: 'yellow' | 'green' | 'red';
}

function StatusCard({ icon, title, value, color }: StatusCardProps) {
  const colorClasses = {
    yellow: 'bg-yellow-50 border-yellow-200',
    green: 'bg-green-50 border-green-200',
    red: 'bg-red-50 border-red-200',
  };

  return (
    <div className={`${colorClasses[color]} rounded-xl p-6 border-2`}>
      <div className="flex items-center gap-4">
        <div className="p-3 bg-white rounded-lg shadow-sm">{icon}</div>
        <div>
          <p className="text-slate-600 text-sm font-medium mb-1">{title}</p>
          <p className="text-3xl font-bold text-slate-900">{value}</p>
        </div>
      </div>
    </div>
  );
}

interface FinancialCardProps {
  title: string;
  value: string;
  description: string;
}

function FinancialCard({ title, value, description }: FinancialCardProps) {
  return (
    <div className="bg-gradient-to-br from-navy-900 to-navy-800 rounded-xl p-6 text-white shadow-lg">
      <h3 className="text-navy-200 text-sm font-medium mb-3">{title}</h3>
      <p className="text-3xl font-bold mb-2">{value}</p>
      <p className="text-navy-300 text-sm">{description}</p>
    </div>
  );
}

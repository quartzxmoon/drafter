// Settlement Analysis Results Page
// Displays comprehensive settlement calculation with visual analytics

import React, { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import {
  TrendingUp,
  DollarSign,
  Scale,
  AlertTriangle,
  CheckCircle,
  FileText,
  Download,
  Send,
  Edit,
  ArrowLeft,
  Info,
  Brain,
  MapPin,
  Gavel,
  Building,
  Target,
  PieChart,
  BarChart3,
  Activity,
} from 'lucide-react';

// Placeholder for chart components - would use recharts or chart.js
const SettlementRangeGauge = ({ low, mid, high, current }: any) => (
  <div className="bg-gradient-to-r from-red-100 via-yellow-100 to-green-100 rounded-lg p-6 relative h-32">
    <div className="absolute inset-0 flex items-center justify-between px-6">
      <div className="text-center">
        <p className="text-sm font-semibold text-red-700">Low</p>
        <p className="text-lg font-bold text-red-900">${(low / 1000).toFixed(0)}K</p>
      </div>
      <div className="text-center">
        <p className="text-sm font-semibold text-yellow-700">Mid</p>
        <p className="text-lg font-bold text-yellow-900">${(mid / 1000).toFixed(0)}K</p>
      </div>
      <div className="text-center">
        <p className="text-sm font-semibold text-green-700">High</p>
        <p className="text-lg font-bold text-green-900">${(high / 1000).toFixed(0)}K</p>
      </div>
    </div>
  </div>
);

const DamagesBreakdownChart = ({ economic, nonEconomic, punitive }: any) => {
  const total = economic + nonEconomic + (punitive || 0);
  const economicPct = (economic / total) * 100;
  const nonEconomicPct = (nonEconomic / total) * 100;
  const punitivePct = punitive ? (punitive / total) * 100 : 0;

  return (
    <div className="space-y-4">
      <div>
        <div className="flex justify-between mb-2">
          <span className="text-sm font-semibold text-navy-900">Economic Damages</span>
          <span className="text-sm font-mono font-bold text-navy-900">
            ${economic.toLocaleString()} ({economicPct.toFixed(1)}%)
          </span>
        </div>
        <div className="w-full bg-slate-200 rounded-full h-3">
          <div
            className="bg-blue-600 h-3 rounded-full"
            style={{ width: `${economicPct}%` }}
          />
        </div>
      </div>

      <div>
        <div className="flex justify-between mb-2">
          <span className="text-sm font-semibold text-navy-900">Non-Economic Damages</span>
          <span className="text-sm font-mono font-bold text-navy-900">
            ${nonEconomic.toLocaleString()} ({nonEconomicPct.toFixed(1)}%)
          </span>
        </div>
        <div className="w-full bg-slate-200 rounded-full h-3">
          <div
            className="bg-amber-600 h-3 rounded-full"
            style={{ width: `${nonEconomicPct}%` }}
          />
        </div>
      </div>

      {punitive && punitive > 0 && (
        <div>
          <div className="flex justify-between mb-2">
            <span className="text-sm font-semibold text-navy-900">Punitive Damages</span>
            <span className="text-sm font-mono font-bold text-navy-900">
              ${punitive.toLocaleString()} ({punitivePct.toFixed(1)}%)
            </span>
          </div>
          <div className="w-full bg-slate-200 rounded-full h-3">
            <div
              className="bg-red-600 h-3 rounded-full"
              style={{ width: `${punitivePct}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default function SettlementAnalysisPage() {
  const { calcId } = useParams<{ calcId: string }>();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('overview');
  const [calculation, setCalculation] = useState<any>(null);

  useEffect(() => {
    loadCalculation();
  }, [calcId]);

  const loadCalculation = async () => {
    try {
      setLoading(true);
      const result = await invoke('cmd_get_settlement_calculation', { calcId });
      setCalculation(result);
    } catch (err) {
      setError(err as string);
      console.error('Failed to load calculation:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleExport = async (format: 'pdf' | 'excel' | 'word') => {
    try {
      const outputPath = await invoke<string>('cmd_export_settlement_report', {
        calcId,
        format,
        outputPath: `settlement_report.${format}`,
      });
      alert(`Report exported to: ${outputPath}`);
    } catch (err) {
      alert(`Export failed: ${err}`);
    }
  };

  const formatCurrency = (amount: number) =>
    new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
    }).format(amount);

  const formatPercentage = (value: number) => `${(value * 100).toFixed(1)}%`;

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-navy-600 mx-auto mb-4"></div>
          <p className="text-slate-600 text-lg">Loading settlement analysis...</p>
        </div>
      </div>
    );
  }

  if (error || !calculation) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-8">
        <div className="max-w-2xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <h3 className="text-red-800 font-semibold text-lg mb-2">Error Loading Analysis</h3>
            <p className="text-red-600">{error || 'Calculation not found'}</p>
            <button
              type="button"
              onClick={() => navigate('/settlement')}
              className="mt-4 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition"
            >
              Back to Dashboard
            </button>
          </div>
        </div>
      </div>
    );
  }

  const tabs = [
    { id: 'overview', label: 'Overview', icon: Activity },
    { id: 'damages', label: 'Damages Breakdown', icon: PieChart },
    { id: 'liability', label: 'Liability Analysis', icon: Scale },
    { id: 'risk', label: 'Risk Assessment', icon: AlertTriangle },
    { id: 'comparables', label: 'Comparable Verdicts', icon: BarChart3 },
    { id: 'ai', label: 'AI Insights', icon: Brain },
    { id: 'strategy', label: 'Negotiation Strategy', icon: Target },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header */}
      <div className="bg-gradient-to-r from-navy-900 to-navy-800 text-white shadow-xl">
        <div className="max-w-7xl mx-auto px-8 py-8">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-4">
              <button
                type="button"
                onClick={() => navigate('/settlement')}
                className="p-2 hover:bg-navy-700 rounded-lg transition"
                aria-label="Back to dashboard"
              >
                <ArrowLeft size={24} />
              </button>
              <div>
                <h1 className="text-3xl font-bold">Settlement Analysis</h1>
                <p className="text-navy-200 mt-1">
                  {calculation.plaintiff_name} v. {calculation.defendant_name}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={() => handleExport('pdf')}
                className="btn-secondary-white flex items-center gap-2"
              >
                <Download size={18} />
                Export PDF
              </button>
              <Link
                to={`/settlement/demand-letter/${calcId}`}
                className="btn-primary-gold flex items-center gap-2"
              >
                <Send size={18} />
                Generate Demand
              </Link>
            </div>
          </div>

          {/* Key Metrics Row */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <MetricCard
              label="Total Damages"
              value={formatCurrency(calculation.total_damages)}
              icon={<DollarSign size={20} />}
            />
            <MetricCard
              label="Recommended Demand"
              value={formatCurrency(calculation.recommended_demand)}
              icon={<Target size={20} />}
            />
            <MetricCard
              label="Minimum Settlement"
              value={formatCurrency(calculation.minimum_settlement)}
              icon={<AlertTriangle size={20} />}
            />
            <MetricCard
              label="Net to Client"
              value={formatCurrency(calculation.net_to_client)}
              icon={<CheckCircle size={20} />}
            />
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="bg-white border-b border-slate-200 sticky top-0 z-10 shadow-sm">
        <div className="max-w-7xl mx-auto px-8">
          <div className="flex gap-1 overflow-x-auto">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 px-6 py-4 font-semibold transition border-b-2 whitespace-nowrap ${
                  activeTab === tab.id
                    ? 'border-navy-600 text-navy-900 bg-navy-50'
                    : 'border-transparent text-slate-600 hover:text-navy-900 hover:bg-slate-50'
                }`}
              >
                <tab.icon size={18} />
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-7xl mx-auto px-8 py-8">
        {/* Overview Tab */}
        {activeTab === 'overview' && (
          <div className="space-y-8">
            {/* Settlement Range */}
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
                <TrendingUp className="text-navy-600" size={28} />
                Settlement Range
              </h2>
              <SettlementRangeGauge
                low={calculation.settlement_range.low_estimate}
                mid={calculation.settlement_range.mid_estimate}
                high={calculation.settlement_range.high_estimate}
                current={calculation.recommended_demand}
              />
              <div className="mt-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
                <p className="text-blue-900 text-sm">
                  <strong>Confidence Level:</strong>{' '}
                  {formatPercentage(calculation.settlement_range.confidence_level)}
                </p>
                <p className="text-blue-800 text-sm mt-2">
                  {calculation.settlement_range.range_explanation}
                </p>
              </div>
            </div>

            {/* Recommendations Summary */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-white rounded-xl shadow-lg p-8">
                <h3 className="text-xl font-bold text-navy-900 mb-4">
                  Settlement Recommendations
                </h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center p-4 bg-green-50 rounded-lg border-2 border-green-200">
                    <span className="font-semibold text-green-900">
                      Recommended Demand
                    </span>
                    <span className="text-2xl font-bold text-green-900">
                      {formatCurrency(calculation.recommended_demand)}
                    </span>
                  </div>
                  <div className="flex justify-between items-center p-4 bg-blue-50 rounded-lg border-2 border-blue-200">
                    <span className="font-semibold text-blue-900">
                      Target Settlement
                    </span>
                    <span className="text-2xl font-bold text-blue-900">
                      {formatCurrency(calculation.target_settlement)}
                    </span>
                  </div>
                  <div className="flex justify-between items-center p-4 bg-amber-50 rounded-lg border-2 border-amber-200">
                    <span className="font-semibold text-amber-900">
                      Minimum Acceptable
                    </span>
                    <span className="text-2xl font-bold text-amber-900">
                      {formatCurrency(calculation.minimum_settlement)}
                    </span>
                  </div>
                </div>
              </div>

              <div className="bg-white rounded-xl shadow-lg p-8">
                <h3 className="text-xl font-bold text-navy-900 mb-4">
                  Attorney Fees & Costs
                </h3>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="text-slate-600">Estimated Attorney Fees</span>
                    <span className="font-mono font-semibold text-navy-900">
                      {formatCurrency(calculation.estimated_attorney_fees)}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-600">Litigation Costs to Date</span>
                    <span className="font-mono font-semibold text-navy-900">
                      {formatCurrency(calculation.litigation_costs_to_date)}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-600">
                      Projected Additional Costs
                    </span>
                    <span className="font-mono font-semibold text-navy-900">
                      {formatCurrency(calculation.projected_additional_costs)}
                    </span>
                  </div>
                  <div className="border-t-2 border-slate-200 pt-3 mt-3">
                    <div className="flex justify-between items-center">
                      <span className="font-bold text-navy-900">
                        Net to Client
                      </span>
                      <span className="text-2xl font-bold text-green-600">
                        {formatCurrency(calculation.net_to_client)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Rationale */}
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h3 className="text-xl font-bold text-navy-900 mb-4 flex items-center gap-2">
                <FileText className="text-navy-600" size={24} />
                Settlement Rationale
              </h3>
              <p className="text-slate-700 leading-relaxed">{calculation.rationale}</p>
            </div>
          </div>
        )}

        {/* Damages Breakdown Tab */}
        {activeTab === 'damages' && (
          <div className="space-y-8">
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Damages Breakdown
              </h2>
              <DamagesBreakdownChart
                economic={calculation.economic_damages.total_economic}
                nonEconomic={calculation.non_economic_damages.total_non_economic}
                punitive={
                  calculation.punitive_damages
                    ? calculation.punitive_damages.amount
                    : 0
                }
              />
            </div>

            {/* Economic Damages Detail */}
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h3 className="text-xl font-bold text-navy-900 mb-6">
                Economic Damages Detail
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <DamageLineItem
                  label="Past Medical Expenses"
                  amount={calculation.economic_damages.past_medical_expenses}
                />
                <DamageLineItem
                  label="Future Medical Expenses"
                  amount={calculation.economic_damages.future_medical_expenses}
                />
                <DamageLineItem
                  label="Past Lost Wages"
                  amount={calculation.economic_damages.past_lost_wages}
                />
                <DamageLineItem
                  label="Future Lost Earning Capacity"
                  amount={calculation.economic_damages.future_lost_earning_capacity}
                />
                <DamageLineItem
                  label="Property Damage"
                  amount={calculation.economic_damages.property_damage}
                />
                <DamageLineItem
                  label="Other Expenses"
                  amount={calculation.economic_damages.other_expenses}
                />
              </div>
              <div className="mt-6 pt-6 border-t-2 border-slate-200">
                <div className="flex justify-between items-center">
                  <span className="text-xl font-bold text-navy-900">
                    Total Economic Damages
                  </span>
                  <span className="text-3xl font-bold text-navy-900">
                    {formatCurrency(calculation.economic_damages.total_economic)}
                  </span>
                </div>
              </div>
            </div>

            {/* Non-Economic Damages Detail */}
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h3 className="text-xl font-bold text-navy-900 mb-6">
                Non-Economic Damages Detail
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <DamageLineItem
                  label="Pain and Suffering"
                  amount={calculation.non_economic_damages.pain_and_suffering}
                />
                <DamageLineItem
                  label="Emotional Distress"
                  amount={calculation.non_economic_damages.emotional_distress}
                />
                <DamageLineItem
                  label="Loss of Enjoyment of Life"
                  amount={calculation.non_economic_damages.loss_of_enjoyment_of_life}
                />
                <DamageLineItem
                  label="Loss of Consortium"
                  amount={calculation.non_economic_damages.loss_of_consortium}
                />
              </div>
              <div className="mt-6 p-4 bg-amber-50 rounded-lg border border-amber-200">
                <p className="text-sm text-amber-900">
                  <strong>Methodology:</strong> {calculation.non_economic_damages.methodology}
                </p>
                <p className="text-sm text-amber-900 mt-1">
                  <strong>Multiplier:</strong> {calculation.non_economic_damages.multiplier}x
                </p>
              </div>
              <div className="mt-6 pt-6 border-t-2 border-slate-200">
                <div className="flex justify-between items-center">
                  <span className="text-xl font-bold text-navy-900">
                    Total Non-Economic Damages
                  </span>
                  <span className="text-3xl font-bold text-navy-900">
                    {formatCurrency(
                      calculation.non_economic_damages.total_non_economic
                    )}
                  </span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Liability Analysis Tab */}
        {activeTab === 'liability' && (
          <div className="space-y-8">
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
                <Scale className="text-navy-600" size={28} />
                Liability Analysis
              </h2>

              {/* Liability Split */}
              <div className="mb-8">
                <div className="flex items-center justify-between mb-4">
                  <div className="text-center flex-1">
                    <p className="text-sm font-semibold text-slate-600 mb-2">
                      Defendant Liability
                    </p>
                    <p className="text-5xl font-bold text-red-600">
                      {calculation.liability_analysis.defendant_liability_percentage}%
                    </p>
                  </div>
                  <div className="text-4xl font-bold text-slate-300">vs</div>
                  <div className="text-center flex-1">
                    <p className="text-sm font-semibold text-slate-600 mb-2">
                      Plaintiff Liability
                    </p>
                    <p className="text-5xl font-bold text-blue-600">
                      {calculation.liability_analysis.plaintiff_liability_percentage}%
                    </p>
                  </div>
                </div>

                <div className="w-full bg-slate-200 rounded-full h-6">
                  <div
                    className="bg-red-600 h-6 rounded-l-full flex items-center justify-end pr-2"
                    style={{
                      width: `${calculation.liability_analysis.defendant_liability_percentage}%`,
                    }}
                  >
                    <span className="text-white text-xs font-bold">Defendant</span>
                  </div>
                </div>
              </div>

              {/* Liability Strength */}
              <div className="mb-8 p-6 bg-slate-50 rounded-lg border-2 border-slate-200">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-semibold text-slate-600 mb-1">
                      Liability Strength
                    </p>
                    <p className="text-2xl font-bold text-navy-900">
                      {calculation.liability_analysis.liability_strength}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-slate-600 mb-1">
                      Jurisdiction
                    </p>
                    <p className="text-2xl font-bold text-navy-900">
                      {calculation.liability_analysis.jurisdiction}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-slate-600 mb-1">
                      Comparative Negligence
                    </p>
                    <p className="text-lg font-bold text-navy-900">
                      {calculation.liability_analysis.comparative_negligence_applies
                        ? 'Applies'
                        : 'Does Not Apply'}
                    </p>
                  </div>
                </div>
              </div>

              {/* Liability Factors */}
              <div>
                <h3 className="text-lg font-bold text-navy-900 mb-4">
                  Key Liability Factors
                </h3>
                <div className="space-y-3">
                  {calculation.liability_analysis.key_liability_factors.map(
                    (factor: any, index: number) => (
                      <div
                        key={index}
                        className={`p-4 rounded-lg border-2 ${
                          factor.favors === 'Plaintiff'
                            ? 'bg-green-50 border-green-200'
                            : 'bg-red-50 border-red-200'
                        }`}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <p className="font-semibold text-navy-900">
                              {factor.factor}
                            </p>
                            <p className="text-sm text-slate-600 mt-1">
                              Favors: <strong>{factor.favors}</strong>
                            </p>
                          </div>
                          <div className="ml-4">
                            <div className="text-right">
                              <p className="text-sm text-slate-600">Weight</p>
                              <p className="text-xl font-bold text-navy-900">
                                {(factor.weight * 100).toFixed(0)}%
                              </p>
                            </div>
                          </div>
                        </div>
                      </div>
                    )
                  )}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Risk Assessment Tab */}
        {activeTab === 'risk' && (
          <div className="space-y-8">
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
                <AlertTriangle className="text-amber-600" size={28} />
                Trial Risk Assessment
              </h2>

              {/* Risk Metrics Grid */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                <div className="p-6 bg-gradient-to-br from-red-50 to-red-100 rounded-lg border-2 border-red-200">
                  <p className="text-sm font-semibold text-red-700 mb-2">
                    Trial Risk Score
                  </p>
                  <p className="text-4xl font-bold text-red-900">
                    {(calculation.risk_assessment.trial_risk_score * 100).toFixed(0)}%
                  </p>
                  <p className="text-xs text-red-600 mt-2">Higher = More Risk</p>
                </div>

                <div className="p-6 bg-gradient-to-br from-green-50 to-green-100 rounded-lg border-2 border-green-200">
                  <p className="text-sm font-semibold text-green-700 mb-2">
                    Probability of Win
                  </p>
                  <p className="text-4xl font-bold text-green-900">
                    {(calculation.risk_assessment.probability_of_win * 100).toFixed(0)}%
                  </p>
                  <p className="text-xs text-green-600 mt-2">At trial</p>
                </div>

                <div className="p-6 bg-gradient-to-br from-blue-50 to-blue-100 rounded-lg border-2 border-blue-200">
                  <p className="text-sm font-semibold text-blue-700 mb-2">
                    Expected Trial Value
                  </p>
                  <p className="text-3xl font-bold text-blue-900">
                    {formatCurrency(calculation.risk_assessment.expected_trial_value)}
                  </p>
                  <p className="text-xs text-blue-600 mt-2">Risk-adjusted</p>
                </div>
              </div>

              {/* Trial Costs */}
              <div className="mb-8 p-6 bg-amber-50 rounded-lg border-2 border-amber-200">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-semibold text-amber-700 mb-1">
                      Estimated Trial Costs
                    </p>
                    <p className="text-3xl font-bold text-amber-900">
                      {formatCurrency(calculation.risk_assessment.trial_cost_estimate)}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-amber-700 mb-1">
                      Expected Trial Duration
                    </p>
                    <p className="text-3xl font-bold text-amber-900">
                      {calculation.risk_assessment.expected_trial_duration_months} months
                    </p>
                  </div>
                </div>
              </div>

              {/* Case Strengths */}
              <div className="mb-8">
                <h3 className="text-lg font-bold text-navy-900 mb-4 flex items-center gap-2">
                  <CheckCircle className="text-green-600" size={24} />
                  Case Strengths
                </h3>
                <div className="space-y-3">
                  {calculation.risk_assessment.strengths.map((strength: any, index: number) => (
                    <div
                      key={index}
                      className="p-4 bg-green-50 rounded-lg border-l-4 border-green-500"
                    >
                      <div className="flex items-start justify-between">
                        <p className="font-semibold text-green-900 flex-1">
                          {strength.description}
                        </p>
                        <span
                          className={`ml-4 px-3 py-1 rounded-full text-sm font-medium ${
                            strength.impact === 'Critical'
                              ? 'bg-green-200 text-green-900'
                              : strength.impact === 'Major'
                              ? 'bg-green-100 text-green-800'
                              : 'bg-green-50 text-green-700'
                          }`}
                        >
                          {strength.impact}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Case Weaknesses */}
              <div>
                <h3 className="text-lg font-bold text-navy-900 mb-4 flex items-center gap-2">
                  <AlertTriangle className="text-red-600" size={24} />
                  Case Weaknesses
                </h3>
                <div className="space-y-3">
                  {calculation.risk_assessment.weaknesses.map(
                    (weakness: any, index: number) => (
                      <div
                        key={index}
                        className="p-4 bg-red-50 rounded-lg border-l-4 border-red-500"
                      >
                        <div className="flex items-start justify-between mb-2">
                          <p className="font-semibold text-red-900 flex-1">
                            {weakness.description}
                          </p>
                          <span
                            className={`ml-4 px-3 py-1 rounded-full text-sm font-medium ${
                              weakness.impact === 'Critical'
                                ? 'bg-red-200 text-red-900'
                                : weakness.impact === 'Major'
                                ? 'bg-red-100 text-red-800'
                                : 'bg-red-50 text-red-700'
                            }`}
                          >
                            {weakness.impact}
                          </span>
                        </div>
                        {weakness.mitigation && (
                          <div className="mt-2 p-3 bg-white rounded border border-red-200">
                            <p className="text-sm text-slate-700">
                              <strong>Mitigation:</strong> {weakness.mitigation}
                            </p>
                          </div>
                        )}
                      </div>
                    )
                  )}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Comparable Verdicts Tab */}
        {activeTab === 'comparables' && (
          <div className="bg-white rounded-xl shadow-lg p-8">
            <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
              <BarChart3 className="text-navy-600" size={28} />
              Comparable Verdicts
            </h2>

            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50 border-b-2 border-slate-200">
                  <tr>
                    <th className="px-6 py-4 text-left text-sm font-semibold text-slate-700">
                      Case Name
                    </th>
                    <th className="px-6 py-4 text-left text-sm font-semibold text-slate-700">
                      Year
                    </th>
                    <th className="px-6 py-4 text-left text-sm font-semibold text-slate-700">
                      Jurisdiction
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Verdict Amount
                    </th>
                    <th className="px-6 py-4 text-right text-sm font-semibold text-slate-700">
                      Similarity
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {calculation.comparable_verdicts.map((verdict: any, index: number) => (
                    <tr key={index} className="hover:bg-slate-50 transition">
                      <td className="px-6 py-4">
                        <div>
                          <p className="font-semibold text-navy-900">{verdict.case_name}</p>
                          <p className="text-sm text-slate-600">{verdict.injury_type}</p>
                        </div>
                      </td>
                      <td className="px-6 py-4 text-slate-700">{verdict.year}</td>
                      <td className="px-6 py-4 text-slate-700">{verdict.jurisdiction}</td>
                      <td className="px-6 py-4 text-right font-mono font-semibold text-navy-900">
                        {formatCurrency(verdict.verdict_amount)}
                      </td>
                      <td className="px-6 py-4 text-right">
                        <span
                          className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                            verdict.similarity_score >= 0.8
                              ? 'bg-green-100 text-green-800'
                              : verdict.similarity_score >= 0.6
                              ? 'bg-yellow-100 text-yellow-800'
                              : 'bg-red-100 text-red-800'
                          }`}
                        >
                          {(verdict.similarity_score * 100).toFixed(0)}%
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* AI Insights Tab */}
        {activeTab === 'ai' && calculation.ai_analysis && (
          <div className="space-y-8">
            <div className="bg-gradient-to-r from-purple-600 to-blue-600 rounded-xl p-8 text-white shadow-xl">
              <div className="flex items-center gap-4 mb-4">
                <Brain size={40} />
                <div>
                  <h2 className="text-3xl font-bold">AI-Powered Settlement Analysis</h2>
                  <p className="text-purple-100">
                    Advanced predictive modeling based on {calculation.ai_analysis.similar_cases_analyzed} similar cases
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
                <div className="bg-white/10 backdrop-blur rounded-lg p-6">
                  <p className="text-purple-100 text-sm mb-2">Predicted Settlement Value</p>
                  <p className="text-4xl font-bold">
                    {formatCurrency(calculation.ai_analysis.predicted_settlement_value)}
                  </p>
                </div>
                <div className="bg-white/10 backdrop-blur rounded-lg p-6">
                  <p className="text-purple-100 text-sm mb-2">Confidence Score</p>
                  <p className="text-4xl font-bold">
                    {(calculation.ai_analysis.confidence_score * 100).toFixed(1)}%
                  </p>
                </div>
              </div>
            </div>

            {/* AI Factors */}
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h3 className="text-xl font-bold text-navy-900 mb-6">
                AI Factor Analysis
              </h3>
              <div className="space-y-4">
                {calculation.ai_analysis.factors_considered.map((factor: any, index: number) => (
                  <div key={index} className="border-l-4 border-blue-500 p-4 bg-blue-50 rounded-r-lg">
                    <div className="flex items-start justify-between mb-2">
                      <div className="flex-1">
                        <p className="font-semibold text-navy-900">{factor.factor_name}</p>
                        <p className="text-sm text-slate-600 mt-1">{factor.description}</p>
                      </div>
                      <div className="ml-4 text-right">
                        <p className="text-sm text-slate-600">Importance</p>
                        <p className="text-xl font-bold text-blue-600">
                          {(factor.importance * 100).toFixed(0)}%
                        </p>
                      </div>
                    </div>
                    <div className="w-full bg-slate-200 rounded-full h-2 mt-2">
                      <div
                        className={`h-2 rounded-full ${
                          factor.impact_direction === 'Positive'
                            ? 'bg-green-500'
                            : factor.impact_direction === 'Negative'
                            ? 'bg-red-500'
                            : 'bg-slate-400'
                        }`}
                        style={{ width: `${factor.importance * 100}%` }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Judge History */}
            {calculation.ai_analysis.judge_history && (
              <div className="bg-white rounded-xl shadow-lg p-8">
                <h3 className="text-xl font-bold text-navy-900 mb-6 flex items-center gap-2">
                  <Gavel className="text-navy-600" size={24} />
                  Judge History: {calculation.ai_analysis.judge_history.judge_name}
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div className="p-4 bg-slate-50 rounded-lg">
                    <p className="text-sm text-slate-600 mb-1">Avg Plaintiff Verdict</p>
                    <p className="text-2xl font-bold text-navy-900">
                      {formatCurrency(calculation.ai_analysis.judge_history.average_plaintiff_verdict)}
                    </p>
                  </div>
                  <div className="p-4 bg-slate-50 rounded-lg">
                    <p className="text-sm text-slate-600 mb-1">Plaintiff Win Rate</p>
                    <p className="text-2xl font-bold text-navy-900">
                      {formatPercentage(calculation.ai_analysis.judge_history.plaintiff_win_rate)}
                    </p>
                  </div>
                  <div className="p-4 bg-slate-50 rounded-lg">
                    <p className="text-sm text-slate-600 mb-1">Settlement Tendency</p>
                    <p className="text-2xl font-bold text-navy-900">
                      {calculation.ai_analysis.judge_history.settlement_encouragement}
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Venue Statistics */}
            {calculation.ai_analysis.venue_statistics && (
              <div className="bg-white rounded-xl shadow-lg p-8">
                <h3 className="text-xl font-bold text-navy-900 mb-6 flex items-center gap-2">
                  <MapPin className="text-navy-600" size={24} />
                  Venue Statistics: {calculation.ai_analysis.venue_statistics.county}
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div className="p-4 bg-slate-50 rounded-lg">
                    <p className="text-sm text-slate-600 mb-1">Avg Plaintiff Verdict</p>
                    <p className="text-2xl font-bold text-navy-900">
                      {formatCurrency(calculation.ai_analysis.venue_statistics.average_plaintiff_verdict)}
                    </p>
                  </div>
                  <div className="p-4 bg-slate-50 rounded-lg">
                    <p className="text-sm text-slate-600 mb-1">Plaintiff Win Rate</p>
                    <p className="text-2xl font-bold text-navy-900">
                      {formatPercentage(calculation.ai_analysis.venue_statistics.plaintiff_win_rate)}
                    </p>
                  </div>
                  <div className="p-4 bg-slate-50 rounded-lg">
                    <p className="text-sm text-slate-600 mb-1">Tort Reform Climate</p>
                    <p className="text-xl font-bold text-navy-900">
                      {calculation.ai_analysis.venue_statistics.tort_reform_climate}
                    </p>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Negotiation Strategy Tab */}
        {activeTab === 'strategy' && (
          <div className="bg-white rounded-xl shadow-lg p-8">
            <h2 className="text-2xl font-bold text-navy-900 mb-6 flex items-center gap-3">
              <Target className="text-navy-600" size={28} />
              Negotiation Strategy
            </h2>

            <div className="space-y-4">
              {calculation.negotiation_strategy.map((strategy: string, index: number) => (
                <div
                  key={index}
                  className="flex items-start gap-4 p-4 bg-blue-50 rounded-lg border-l-4 border-blue-500"
                >
                  <div className="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold">
                    {index + 1}
                  </div>
                  <p className="flex-1 text-navy-900 leading-relaxed">{strategy}</p>
                </div>
              ))}
            </div>

            <div className="mt-8 p-6 bg-gradient-to-r from-navy-900 to-navy-800 rounded-lg text-white">
              <h3 className="text-xl font-bold mb-3">Next Steps</h3>
              <ul className="space-y-2 text-navy-100">
                <li className="flex items-center gap-2">
                  <CheckCircle size={18} />
                  Generate and send demand letter
                </li>
                <li className="flex items-center gap-2">
                  <CheckCircle size={18} />
                  Prepare client for settlement negotiations
                </li>
                <li className="flex items-center gap-2">
                  <CheckCircle size={18} />
                  Document all offers and counteroffers
                </li>
                <li className="flex items-center gap-2">
                  <CheckCircle size={18} />
                  Monitor approaching statute of limitations
                </li>
              </ul>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Helper Components
function MetricCard({ label, value, icon }: any) {
  return (
    <div className="bg-white/10 backdrop-blur rounded-lg p-4">
      <div className="flex items-center gap-2 mb-2">
        {icon}
        <p className="text-sm text-navy-100">{label}</p>
      </div>
      <p className="text-2xl font-bold">{value}</p>
    </div>
  );
}

function DamageLineItem({ label, amount }: { label: string; amount: number }) {
  return (
    <div className="flex justify-between items-center p-4 bg-slate-50 rounded-lg">
      <span className="text-slate-700 font-medium">{label}</span>
      <span className="text-navy-900 font-mono font-semibold">
        {new Intl.NumberFormat('en-US', {
          style: 'currency',
          currency: 'USD',
          minimumFractionDigits: 0,
        }).format(amount)}
      </span>
    </div>
  );
}

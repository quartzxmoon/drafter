// Settlement Calculator Wizard - Multi-Step Form
// Comprehensive 6-step wizard for creating settlement calculations

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import {
  ChevronRight,
  ChevronLeft,
  Save,
  Calculator,
  DollarSign,
  Heart,
  Scale,
  Brain,
  CheckCircle,
  AlertTriangle,
  Info,
  Plus,
  X,
} from 'lucide-react';

// ============= TYPE DEFINITIONS =============

interface EconomicDamages {
  past_medical_expenses: number;
  future_medical_expenses: number;
  past_lost_wages: number;
  future_lost_earning_capacity: number;
  lost_benefits: number;
  property_damage: number;
  rehabilitation_costs: number;
  home_modification_costs: number;
  assistive_device_costs: number;
  transportation_costs: number;
  other_expenses: number;
  discount_rate: number;
  total_past_economic: number;
  total_future_economic: number;
  total_economic: number;
  present_value_future_damages: number;
  medical_expense_details: MedicalExpense[];
}

interface MedicalExpense {
  date: string;
  provider: string;
  description: string;
  amount: number;
  category: string;
  is_future: boolean;
}

interface PersonalInjuryDetails {
  injury_type: string;
  injury_severity: string;
  permanent_disability: boolean;
  disability_percentage: number | null;
  scarring_disfigurement: boolean;
  treatment_ongoing: boolean;
  full_recovery_expected: boolean;
  life_expectancy_impact: number | null;
}

interface LiabilityFactor {
  factor: string;
  favors: string;
  weight: number;
}

// ============= MAIN WIZARD COMPONENT =============

export default function SettlementCalculatorWizard() {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Step 1: Case Information
  const [matterId, setMatterId] = useState('');
  const [caseType, setCaseType] = useState('PersonalInjury');
  const [plaintiffName, setPlaintiffName] = useState('');
  const [defendantName, setDefendantName] = useState('');
  const [incidentDate, setIncidentDate] = useState('');
  const [jurisdiction, setJurisdiction] = useState('PA');

  // Step 2: Economic Damages
  const [economicDamages, setEconomicDamages] = useState<EconomicDamages>({
    past_medical_expenses: 0,
    future_medical_expenses: 0,
    past_lost_wages: 0,
    future_lost_earning_capacity: 0,
    lost_benefits: 0,
    property_damage: 0,
    rehabilitation_costs: 0,
    home_modification_costs: 0,
    assistive_device_costs: 0,
    transportation_costs: 0,
    other_expenses: 0,
    discount_rate: 0.03,
    total_past_economic: 0,
    total_future_economic: 0,
    total_economic: 0,
    present_value_future_damages: 0,
    medical_expense_details: [],
  });

  const [medicalExpenses, setMedicalExpenses] = useState<MedicalExpense[]>([]);

  // Step 3: Injury Details
  const [injuryDetails, setInjuryDetails] = useState<PersonalInjuryDetails>({
    injury_type: 'SoftTissue',
    injury_severity: 'Moderate',
    permanent_disability: false,
    disability_percentage: null,
    scarring_disfigurement: false,
    treatment_ongoing: false,
    full_recovery_expected: true,
    life_expectancy_impact: null,
  });

  // Step 4: Liability Assessment
  const [liabilityPercentage, setLiabilityPercentage] = useState(100);
  const [liabilityFactors, setLiabilityFactors] = useState<LiabilityFactor[]>([
    { factor: 'Clear causation documented', favors: 'Plaintiff', weight: 0.9 },
  ]);

  // Step 5: Advanced Options
  const [judgeName, setJudgeName] = useState('');
  const [opposingCounsel, setOpposingCounsel] = useState('');
  const [insuranceCompany, setInsuranceCompany] = useState('');
  const [calculatedBy, setCalculatedBy] = useState('');

  const steps = [
    { number: 1, title: 'Case Information', icon: Scale },
    { number: 2, title: 'Economic Damages', icon: DollarSign },
    { number: 3, title: 'Injury Details', icon: Heart },
    { number: 4, title: 'Liability', icon: Scale },
    { number: 5, title: 'Advanced Options', icon: Brain },
    { number: 6, title: 'Review & Calculate', icon: Calculator },
  ];

  const caseTypes = [
    'PersonalInjury',
    'MedicalMalpractice',
    'Employment',
    'ContractBreach',
    'RealEstate',
    'IntellectualProperty',
    'CommercialDispute',
    'WrongfulDeath',
    'ProductLiability',
    'ClassAction',
    'CivilRights',
    'ProfessionalMalpractice',
    'ConstructionDefect',
    'BusinessTort',
    'InsuranceBadFaith',
    'WorkersCompensation',
    'SocialSecurityDisability',
    'ToxicTort',
    'MassTort',
    'Antitrust',
  ];

  const jurisdictions = [
    { code: 'PA', name: 'Pennsylvania' },
    { code: 'NY', name: 'New York' },
    { code: 'CA', name: 'California' },
    { code: 'TX', name: 'Texas' },
    { code: 'FL', name: 'Florida' },
    { code: 'IL', name: 'Illinois' },
    { code: 'OH', name: 'Ohio' },
    { code: 'NJ', name: 'New Jersey' },
  ];

  const injuryTypes = [
    'TraumaticBrainInjury',
    'SpinalCordInjury',
    'Amputation',
    'Burns',
    'Fractures',
    'SoftTissue',
    'Whiplash',
    'Organ_damage',
    'Psychological',
    'Multiple',
  ];

  const injurySeverities = ['Catastrophic', 'Severe', 'Moderate', 'Minor'];

  const medicalCategories = [
    'Emergency',
    'Hospital',
    'Surgery',
    'Physician',
    'Specialist',
    'PhysicalTherapy',
    'Medication',
    'MedicalEquipment',
    'HomeCare',
    'Diagnostic',
  ];

  // Calculate totals whenever economic damages change
  React.useEffect(() => {
    calculateEconomicTotals();
  }, [
    economicDamages.past_medical_expenses,
    economicDamages.future_medical_expenses,
    economicDamages.past_lost_wages,
    economicDamages.future_lost_earning_capacity,
    economicDamages.property_damage,
    economicDamages.other_expenses,
    economicDamages.rehabilitation_costs,
    economicDamages.home_modification_costs,
    economicDamages.assistive_device_costs,
    economicDamages.transportation_costs,
  ]);

  const calculateEconomicTotals = () => {
    const totalPast =
      economicDamages.past_medical_expenses +
      economicDamages.past_lost_wages +
      economicDamages.property_damage +
      economicDamages.other_expenses;

    const totalFuture =
      economicDamages.future_medical_expenses +
      economicDamages.future_lost_earning_capacity +
      economicDamages.rehabilitation_costs +
      economicDamages.home_modification_costs +
      economicDamages.assistive_device_costs +
      economicDamages.transportation_costs;

    // Present value calculation: PV = FV / (1 + r)^n
    const years = 30; // Assume 30 year period
    const presentValue =
      totalFuture / Math.pow(1 + economicDamages.discount_rate, years);

    const totalEconomic = totalPast + presentValue;

    setEconomicDamages({
      ...economicDamages,
      total_past_economic: totalPast,
      total_future_economic: totalFuture,
      present_value_future_damages: presentValue,
      total_economic: totalEconomic,
    });
  };

  const addMedicalExpense = () => {
    setMedicalExpenses([
      ...medicalExpenses,
      {
        date: new Date().toISOString().split('T')[0],
        provider: '',
        description: '',
        amount: 0,
        category: 'Physician',
        is_future: false,
      },
    ]);
  };

  const removeMedicalExpense = (index: number) => {
    setMedicalExpenses(medicalExpenses.filter((_, i) => i !== index));
  };

  const updateMedicalExpense = (index: number, field: string, value: any) => {
    const updated = [...medicalExpenses];
    updated[index] = { ...updated[index], [field]: value };
    setMedicalExpenses(updated);
  };

  const addLiabilityFactor = () => {
    setLiabilityFactors([
      ...liabilityFactors,
      { factor: '', favors: 'Plaintiff', weight: 0.5 },
    ]);
  };

  const removeLiabilityFactor = (index: number) => {
    setLiabilityFactors(liabilityFactors.filter((_, i) => i !== index));
  };

  const updateLiabilityFactor = (index: number, field: string, value: any) => {
    const updated = [...liabilityFactors];
    updated[index] = { ...updated[index], [field]: value };
    setLiabilityFactors(updated);
  };

  const handleNext = () => {
    if (currentStep < 6) {
      setCurrentStep(currentStep + 1);
      window.scrollTo({ top: 0, behavior: 'smooth' });
    }
  };

  const handleBack = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1);
      window.scrollTo({ top: 0, behavior: 'smooth' });
    }
  };

  const handleSubmit = async () => {
    try {
      setLoading(true);
      setError(null);

      // Prepare economic damages with medical expenses
      const economicDamagesPayload = {
        ...economicDamages,
        medical_expense_details: medicalExpenses,
      };

      // Call Tauri command to calculate settlement
      const result = await invoke('cmd_calculate_settlement', {
        matterId,
        caseType,
        plaintiffName,
        defendantName,
        economicDamages: economicDamagesPayload,
        injuryDetails,
        liabilityPercentage,
        jurisdiction,
        calculatedBy,
      });

      // Navigate to results page
      navigate(`/settlement/results/${result}`);
    } catch (err) {
      setError(err as string);
      console.error('Failed to calculate settlement:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
    }).format(value);
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header */}
      <div className="bg-gradient-to-r from-navy-900 to-navy-800 text-white shadow-xl">
        <div className="max-w-7xl mx-auto px-8 py-6">
          <h1 className="text-3xl font-bold">Settlement Calculator</h1>
          <p className="text-navy-200 mt-1">
            Comprehensive settlement analysis wizard
          </p>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-8 py-8">
        {/* Progress Steps */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            {steps.map((step, index) => (
              <React.Fragment key={step.number}>
                <div className="flex flex-col items-center">
                  <div
                    className={`w-12 h-12 rounded-full flex items-center justify-center font-bold transition-all ${
                      currentStep === step.number
                        ? 'bg-navy-600 text-white scale-110 shadow-lg'
                        : currentStep > step.number
                        ? 'bg-green-500 text-white'
                        : 'bg-slate-200 text-slate-500'
                    }`}
                  >
                    {currentStep > step.number ? (
                      <CheckCircle size={24} />
                    ) : (
                      <step.icon size={24} />
                    )}
                  </div>
                  <p
                    className={`mt-2 text-sm font-medium ${
                      currentStep === step.number
                        ? 'text-navy-900'
                        : 'text-slate-500'
                    }`}
                  >
                    {step.title}
                  </p>
                </div>
                {index < steps.length - 1 && (
                  <div
                    className={`flex-1 h-1 mx-4 ${
                      currentStep > step.number
                        ? 'bg-green-500'
                        : 'bg-slate-200'
                    }`}
                  />
                )}
              </React.Fragment>
            ))}
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <div className="mb-6 bg-red-50 border-l-4 border-red-500 p-4 rounded">
            <div className="flex items-center">
              <AlertTriangle className="text-red-500 mr-3" size={24} />
              <div>
                <p className="text-red-800 font-medium">Error</p>
                <p className="text-red-600 text-sm">{error}</p>
              </div>
            </div>
          </div>
        )}

        {/* Step Content */}
        <div className="bg-white rounded-xl shadow-lg p-8">
          {/* Step 1: Case Information */}
          {currentStep === 1 && (
            <div>
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Case Information
              </h2>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <FormField label="Matter ID" required>
                  <input
                    type="text"
                    value={matterId}
                    onChange={(e) => setMatterId(e.target.value)}
                    className="form-input"
                    placeholder="Enter matter ID"
                  />
                </FormField>

                <FormField label="Case Type" required>
                  <select
                    value={caseType}
                    onChange={(e) => setCaseType(e.target.value)}
                    className="form-select"
                  >
                    {caseTypes.map((type) => (
                      <option key={type} value={type}>
                        {type.replace(/([A-Z])/g, ' $1').trim()}
                      </option>
                    ))}
                  </select>
                </FormField>

                <FormField label="Plaintiff Name" required>
                  <input
                    type="text"
                    value={plaintiffName}
                    onChange={(e) => setPlaintiffName(e.target.value)}
                    className="form-input"
                    placeholder="Enter plaintiff name"
                  />
                </FormField>

                <FormField label="Defendant Name" required>
                  <input
                    type="text"
                    value={defendantName}
                    onChange={(e) => setDefendantName(e.target.value)}
                    className="form-input"
                    placeholder="Enter defendant name"
                  />
                </FormField>

                <FormField label="Incident Date">
                  <input
                    type="date"
                    value={incidentDate}
                    onChange={(e) => setIncidentDate(e.target.value)}
                    className="form-input"
                  />
                </FormField>

                <FormField label="Jurisdiction" required>
                  <select
                    value={jurisdiction}
                    onChange={(e) => setJurisdiction(e.target.value)}
                    className="form-select"
                  >
                    {jurisdictions.map((j) => (
                      <option key={j.code} value={j.code}>
                        {j.name}
                      </option>
                    ))}
                  </select>
                </FormField>
              </div>
            </div>
          )}

          {/* Step 2: Economic Damages */}
          {currentStep === 2 && (
            <div>
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Economic Damages
              </h2>

              <div className="space-y-8">
                {/* Medical Expenses */}
                <div>
                  <h3 className="text-lg font-semibold text-navy-800 mb-4">
                    Medical Expenses
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <FormField label="Past Medical Expenses">
                      <CurrencyInput
                        value={economicDamages.past_medical_expenses}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            past_medical_expenses: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Future Medical Expenses">
                      <CurrencyInput
                        value={economicDamages.future_medical_expenses}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            future_medical_expenses: value,
                          })
                        }
                      />
                    </FormField>
                  </div>
                </div>

                {/* Lost Income */}
                <div>
                  <h3 className="text-lg font-semibold text-navy-800 mb-4">
                    Lost Income
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <FormField label="Past Lost Wages">
                      <CurrencyInput
                        value={economicDamages.past_lost_wages}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            past_lost_wages: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Future Lost Earning Capacity">
                      <CurrencyInput
                        value={economicDamages.future_lost_earning_capacity}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            future_lost_earning_capacity: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Lost Benefits">
                      <CurrencyInput
                        value={economicDamages.lost_benefits}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            lost_benefits: value,
                          })
                        }
                      />
                    </FormField>
                  </div>
                </div>

                {/* Other Economic Losses */}
                <div>
                  <h3 className="text-lg font-semibold text-navy-800 mb-4">
                    Other Economic Losses
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <FormField label="Property Damage">
                      <CurrencyInput
                        value={economicDamages.property_damage}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            property_damage: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Rehabilitation Costs">
                      <CurrencyInput
                        value={economicDamages.rehabilitation_costs}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            rehabilitation_costs: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Home Modification Costs">
                      <CurrencyInput
                        value={economicDamages.home_modification_costs}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            home_modification_costs: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Assistive Device Costs">
                      <CurrencyInput
                        value={economicDamages.assistive_device_costs}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            assistive_device_costs: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Transportation Costs">
                      <CurrencyInput
                        value={economicDamages.transportation_costs}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            transportation_costs: value,
                          })
                        }
                      />
                    </FormField>

                    <FormField label="Other Expenses">
                      <CurrencyInput
                        value={economicDamages.other_expenses}
                        onChange={(value) =>
                          setEconomicDamages({
                            ...economicDamages,
                            other_expenses: value,
                          })
                        }
                      />
                    </FormField>
                  </div>
                </div>

                {/* Discount Rate */}
                <div>
                  <FormField
                    label="Discount Rate (for present value)"
                    helpText="Typically 2-5% for future damages"
                  >
                    <div className="flex items-center gap-4">
                      <input
                        type="range"
                        min="0"
                        max="0.10"
                        step="0.001"
                        value={economicDamages.discount_rate}
                        onChange={(e) =>
                          setEconomicDamages({
                            ...economicDamages,
                            discount_rate: parseFloat(e.target.value),
                          })
                        }
                        className="flex-1"
                      />
                      <span className="font-mono font-semibold text-navy-900 w-20">
                        {(economicDamages.discount_rate * 100).toFixed(1)}%
                      </span>
                    </div>
                  </FormField>
                </div>

                {/* Totals Summary */}
                <div className="bg-slate-50 rounded-lg p-6 border-2 border-slate-200">
                  <h3 className="text-lg font-semibold text-navy-800 mb-4">
                    Economic Damages Summary
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <p className="text-sm text-slate-600">
                        Total Past Economic
                      </p>
                      <p className="text-2xl font-bold text-navy-900">
                        {formatCurrency(economicDamages.total_past_economic)}
                      </p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-600">
                        Total Future Economic
                      </p>
                      <p className="text-2xl font-bold text-navy-900">
                        {formatCurrency(economicDamages.total_future_economic)}
                      </p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-600">
                        Present Value (Future)
                      </p>
                      <p className="text-2xl font-bold text-green-600">
                        {formatCurrency(
                          economicDamages.present_value_future_damages
                        )}
                      </p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-600">
                        Total Economic Damages
                      </p>
                      <p className="text-3xl font-bold text-navy-900">
                        {formatCurrency(economicDamages.total_economic)}
                      </p>
                    </div>
                  </div>
                </div>

                {/* Itemized Medical Expenses */}
                <div>
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-semibold text-navy-800">
                      Itemized Medical Expenses (Optional)
                    </h3>
                    <button
                      onClick={addMedicalExpense}
                      className="btn-secondary flex items-center gap-2"
                    >
                      <Plus size={16} />
                      Add Expense
                    </button>
                  </div>

                  {medicalExpenses.map((expense, index) => (
                    <div
                      key={index}
                      className="bg-slate-50 rounded-lg p-4 mb-4 border border-slate-200"
                    >
                      <div className="flex items-start justify-between mb-4">
                        <h4 className="font-semibold text-navy-800">
                          Expense #{index + 1}
                        </h4>
                        <button
                          onClick={() => removeMedicalExpense(index)}
                          className="text-red-600 hover:text-red-700"
                        >
                          <X size={20} />
                        </button>
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <FormField label="Date">
                          <input
                            type="date"
                            value={expense.date}
                            onChange={(e) =>
                              updateMedicalExpense(index, 'date', e.target.value)
                            }
                            className="form-input"
                          />
                        </FormField>

                        <FormField label="Provider">
                          <input
                            type="text"
                            value={expense.provider}
                            onChange={(e) =>
                              updateMedicalExpense(
                                index,
                                'provider',
                                e.target.value
                              )
                            }
                            className="form-input"
                            placeholder="Doctor, Hospital, etc."
                          />
                        </FormField>

                        <FormField label="Category">
                          <select
                            value={expense.category}
                            onChange={(e) =>
                              updateMedicalExpense(
                                index,
                                'category',
                                e.target.value
                              )
                            }
                            className="form-select"
                          >
                            {medicalCategories.map((cat) => (
                              <option key={cat} value={cat}>
                                {cat}
                              </option>
                            ))}
                          </select>
                        </FormField>

                        <FormField label="Description" className="md:col-span-2">
                          <input
                            type="text"
                            value={expense.description}
                            onChange={(e) =>
                              updateMedicalExpense(
                                index,
                                'description',
                                e.target.value
                              )
                            }
                            className="form-input"
                            placeholder="Brief description"
                          />
                        </FormField>

                        <FormField label="Amount">
                          <CurrencyInput
                            value={expense.amount}
                            onChange={(value) =>
                              updateMedicalExpense(index, 'amount', value)
                            }
                          />
                        </FormField>
                      </div>

                      <div className="mt-4">
                        <label className="flex items-center gap-2">
                          <input
                            type="checkbox"
                            checked={expense.is_future}
                            onChange={(e) =>
                              updateMedicalExpense(
                                index,
                                'is_future',
                                e.target.checked
                              )
                            }
                            className="form-checkbox"
                          />
                          <span className="text-sm text-slate-700">
                            Future expense
                          </span>
                        </label>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Step 3: Injury Details */}
          {currentStep === 3 && (
            <div>
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Injury Details
              </h2>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <FormField label="Injury Type" required>
                  <select
                    value={injuryDetails.injury_type}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        injury_type: e.target.value,
                      })
                    }
                    className="form-select"
                  >
                    {injuryTypes.map((type) => (
                      <option key={type} value={type}>
                        {type.replace(/([A-Z])/g, ' $1').trim()}
                      </option>
                    ))}
                  </select>
                </FormField>

                <FormField label="Injury Severity" required>
                  <select
                    value={injuryDetails.injury_severity}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        injury_severity: e.target.value,
                      })
                    }
                    className="form-select"
                  >
                    {injurySeverities.map((severity) => (
                      <option key={severity} value={severity}>
                        {severity}
                      </option>
                    ))}
                  </select>
                </FormField>

                <FormField label="Disability Percentage (if applicable)">
                  <div className="flex items-center gap-4">
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={injuryDetails.disability_percentage || 0}
                      onChange={(e) =>
                        setInjuryDetails({
                          ...injuryDetails,
                          disability_percentage: parseInt(e.target.value),
                        })
                      }
                      className="flex-1"
                      disabled={!injuryDetails.permanent_disability}
                    />
                    <span className="font-mono font-semibold text-navy-900 w-16">
                      {injuryDetails.disability_percentage || 0}%
                    </span>
                  </div>
                </FormField>

                <FormField label="Life Expectancy Impact (years reduced)">
                  <input
                    type="number"
                    value={injuryDetails.life_expectancy_impact || ''}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        life_expectancy_impact: e.target.value
                          ? parseInt(e.target.value)
                          : null,
                      })
                    }
                    className="form-input"
                    placeholder="0"
                    min="0"
                  />
                </FormField>
              </div>

              <div className="mt-6 space-y-4">
                <label className="flex items-center gap-3 p-4 bg-slate-50 rounded-lg border-2 border-slate-200 cursor-pointer hover:bg-slate-100 transition">
                  <input
                    type="checkbox"
                    checked={injuryDetails.permanent_disability}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        permanent_disability: e.target.checked,
                      })
                    }
                    className="form-checkbox"
                  />
                  <div>
                    <p className="font-semibold text-navy-900">
                      Permanent Disability
                    </p>
                    <p className="text-sm text-slate-600">
                      Injury results in permanent disability
                    </p>
                  </div>
                </label>

                <label className="flex items-center gap-3 p-4 bg-slate-50 rounded-lg border-2 border-slate-200 cursor-pointer hover:bg-slate-100 transition">
                  <input
                    type="checkbox"
                    checked={injuryDetails.scarring_disfigurement}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        scarring_disfigurement: e.target.checked,
                      })
                    }
                    className="form-checkbox"
                  />
                  <div>
                    <p className="font-semibold text-navy-900">
                      Scarring or Disfigurement
                    </p>
                    <p className="text-sm text-slate-600">
                      Visible scarring or disfigurement present
                    </p>
                  </div>
                </label>

                <label className="flex items-center gap-3 p-4 bg-slate-50 rounded-lg border-2 border-slate-200 cursor-pointer hover:bg-slate-100 transition">
                  <input
                    type="checkbox"
                    checked={injuryDetails.treatment_ongoing}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        treatment_ongoing: e.target.checked,
                      })
                    }
                    className="form-checkbox"
                  />
                  <div>
                    <p className="font-semibold text-navy-900">
                      Treatment Ongoing
                    </p>
                    <p className="text-sm text-slate-600">
                      Patient is still receiving treatment
                    </p>
                  </div>
                </label>

                <label className="flex items-center gap-3 p-4 bg-slate-50 rounded-lg border-2 border-slate-200 cursor-pointer hover:bg-slate-100 transition">
                  <input
                    type="checkbox"
                    checked={injuryDetails.full_recovery_expected}
                    onChange={(e) =>
                      setInjuryDetails({
                        ...injuryDetails,
                        full_recovery_expected: e.target.checked,
                      })
                    }
                    className="form-checkbox"
                  />
                  <div>
                    <p className="font-semibold text-navy-900">
                      Full Recovery Expected
                    </p>
                    <p className="text-sm text-slate-600">
                      Medical providers expect full recovery
                    </p>
                  </div>
                </label>
              </div>
            </div>
          )}

          {/* Step 4: Liability Assessment */}
          {currentStep === 4 && (
            <div>
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Liability Assessment
              </h2>

              <div className="mb-8">
                <FormField
                  label="Defendant Liability Percentage"
                  helpText="How much fault does the defendant bear?"
                >
                  <div className="flex items-center gap-6">
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={liabilityPercentage}
                      onChange={(e) =>
                        setLiabilityPercentage(parseInt(e.target.value))
                      }
                      className="flex-1"
                    />
                    <div className="w-32 text-center">
                      <p className="text-4xl font-bold text-navy-900">
                        {liabilityPercentage}%
                      </p>
                      <p className="text-sm text-slate-600">Defendant fault</p>
                    </div>
                  </div>
                </FormField>

                <div className="mt-4 flex items-center gap-2 text-sm">
                  <Info size={16} className="text-blue-500" />
                  <p className="text-slate-600">
                    Plaintiff fault: {100 - liabilityPercentage}%
                  </p>
                </div>
              </div>

              {/* Liability Factors */}
              <div>
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-semibold text-navy-800">
                    Liability Factors
                  </h3>
                  <button
                    onClick={addLiabilityFactor}
                    className="btn-secondary flex items-center gap-2"
                  >
                    <Plus size={16} />
                    Add Factor
                  </button>
                </div>

                <div className="space-y-4">
                  {liabilityFactors.map((factor, index) => (
                    <div
                      key={index}
                      className="bg-slate-50 rounded-lg p-4 border border-slate-200"
                    >
                      <div className="flex items-start justify-between mb-4">
                        <h4 className="font-semibold text-navy-800">
                          Factor #{index + 1}
                        </h4>
                        <button
                          onClick={() => removeLiabilityFactor(index)}
                          className="text-red-600 hover:text-red-700"
                        >
                          <X size={20} />
                        </button>
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <FormField label="Description" className="md:col-span-2">
                          <input
                            type="text"
                            value={factor.factor}
                            onChange={(e) =>
                              updateLiabilityFactor(
                                index,
                                'factor',
                                e.target.value
                              )
                            }
                            className="form-input"
                            placeholder="e.g., Clear causation documented"
                          />
                        </FormField>

                        <FormField label="Favors">
                          <select
                            value={factor.favors}
                            onChange={(e) =>
                              updateLiabilityFactor(
                                index,
                                'favors',
                                e.target.value
                              )
                            }
                            className="form-select"
                          >
                            <option value="Plaintiff">Plaintiff</option>
                            <option value="Defendant">Defendant</option>
                          </select>
                        </FormField>

                        <FormField label="Weight (Importance)" className="md:col-span-3">
                          <div className="flex items-center gap-4">
                            <input
                              type="range"
                              min="0"
                              max="1"
                              step="0.1"
                              value={factor.weight}
                              onChange={(e) =>
                                updateLiabilityFactor(
                                  index,
                                  'weight',
                                  parseFloat(e.target.value)
                                )
                              }
                              className="flex-1"
                            />
                            <span className="font-mono font-semibold text-navy-900 w-20">
                              {(factor.weight * 100).toFixed(0)}%
                            </span>
                          </div>
                        </FormField>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Step 5: Advanced Options */}
          {currentStep === 5 && (
            <div>
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Advanced Options
              </h2>

              <div className="bg-blue-50 border-l-4 border-blue-500 p-4 mb-6 rounded">
                <div className="flex items-start gap-3">
                  <Info className="text-blue-600 flex-shrink-0 mt-1" size={20} />
                  <div>
                    <p className="text-blue-900 font-medium">AI-Enhanced Analysis</p>
                    <p className="text-blue-700 text-sm mt-1">
                      Providing judge, counsel, and insurance information enables AI-powered
                      settlement predictions based on historical patterns and behaviors.
                    </p>
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <FormField label="Assigned Judge (Optional)">
                  <input
                    type="text"
                    value={judgeName}
                    onChange={(e) => setJudgeName(e.target.value)}
                    className="form-input"
                    placeholder="e.g., Hon. John Smith"
                  />
                </FormField>

                <FormField label="Opposing Counsel (Optional)">
                  <input
                    type="text"
                    value={opposingCounsel}
                    onChange={(e) => setOpposingCounsel(e.target.value)}
                    className="form-input"
                    placeholder="e.g., Jane Doe, Esq."
                  />
                </FormField>

                <FormField label="Insurance Company (Optional)">
                  <input
                    type="text"
                    value={insuranceCompany}
                    onChange={(e) => setInsuranceCompany(e.target.value)}
                    className="form-input"
                    placeholder="e.g., State Farm"
                  />
                </FormField>

                <FormField label="Calculated By" required>
                  <input
                    type="text"
                    value={calculatedBy}
                    onChange={(e) => setCalculatedBy(e.target.value)}
                    className="form-input"
                    placeholder="Your name"
                  />
                </FormField>
              </div>
            </div>
          )}

          {/* Step 6: Review & Calculate */}
          {currentStep === 6 && (
            <div>
              <h2 className="text-2xl font-bold text-navy-900 mb-6">
                Review & Calculate
              </h2>

              <div className="space-y-6">
                {/* Case Information Summary */}
                <ReviewSection title="Case Information">
                  <ReviewItem label="Matter ID" value={matterId} />
                  <ReviewItem
                    label="Case Type"
                    value={caseType.replace(/([A-Z])/g, ' $1').trim()}
                  />
                  <ReviewItem label="Plaintiff" value={plaintiffName} />
                  <ReviewItem label="Defendant" value={defendantName} />
                  <ReviewItem
                    label="Jurisdiction"
                    value={
                      jurisdictions.find((j) => j.code === jurisdiction)?.name ||
                      jurisdiction
                    }
                  />
                  {incidentDate && (
                    <ReviewItem
                      label="Incident Date"
                      value={new Date(incidentDate).toLocaleDateString()}
                    />
                  )}
                </ReviewSection>

                {/* Economic Damages Summary */}
                <ReviewSection title="Economic Damages">
                  <ReviewItem
                    label="Total Past Economic"
                    value={formatCurrency(economicDamages.total_past_economic)}
                    highlight
                  />
                  <ReviewItem
                    label="Total Future Economic"
                    value={formatCurrency(economicDamages.total_future_economic)}
                    highlight
                  />
                  <ReviewItem
                    label="Present Value (Future)"
                    value={formatCurrency(
                      economicDamages.present_value_future_damages
                    )}
                    highlight
                  />
                  <ReviewItem
                    label="Total Economic Damages"
                    value={formatCurrency(economicDamages.total_economic)}
                    highlight
                    large
                  />
                </ReviewSection>

                {/* Injury Details Summary */}
                <ReviewSection title="Injury Details">
                  <ReviewItem
                    label="Injury Type"
                    value={injuryDetails.injury_type.replace(/([A-Z])/g, ' $1').trim()}
                  />
                  <ReviewItem
                    label="Severity"
                    value={injuryDetails.injury_severity}
                  />
                  <ReviewItem
                    label="Permanent Disability"
                    value={injuryDetails.permanent_disability ? 'Yes' : 'No'}
                  />
                  {injuryDetails.disability_percentage && (
                    <ReviewItem
                      label="Disability Percentage"
                      value={`${injuryDetails.disability_percentage}%`}
                    />
                  )}
                </ReviewSection>

                {/* Liability Summary */}
                <ReviewSection title="Liability">
                  <ReviewItem
                    label="Defendant Liability"
                    value={`${liabilityPercentage}%`}
                    highlight
                  />
                  <ReviewItem
                    label="Plaintiff Liability"
                    value={`${100 - liabilityPercentage}%`}
                  />
                  <ReviewItem
                    label="Liability Factors"
                    value={`${liabilityFactors.length} factors documented`}
                  />
                </ReviewSection>

                {/* Advanced Options Summary */}
                {(judgeName || opposingCounsel || insuranceCompany) && (
                  <ReviewSection title="Advanced Options">
                    {judgeName && <ReviewItem label="Judge" value={judgeName} />}
                    {opposingCounsel && (
                      <ReviewItem label="Opposing Counsel" value={opposingCounsel} />
                    )}
                    {insuranceCompany && (
                      <ReviewItem
                        label="Insurance Company"
                        value={insuranceCompany}
                      />
                    )}
                  </ReviewSection>
                )}
              </div>

              <div className="mt-8 bg-gradient-to-r from-navy-900 to-navy-800 rounded-xl p-8 text-white">
                <h3 className="text-2xl font-bold mb-2">Ready to Calculate</h3>
                <p className="text-navy-200 mb-6">
                  Click the button below to generate your comprehensive settlement
                  analysis with AI-powered insights, jurisdiction-specific rules, and
                  detailed recommendations.
                </p>
                <button
                  onClick={handleSubmit}
                  disabled={loading}
                  className="btn-primary-large flex items-center gap-3"
                >
                  {loading ? (
                    <>
                      <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
                      Calculating...
                    </>
                  ) : (
                    <>
                      <Calculator size={24} />
                      Calculate Settlement
                    </>
                  )}
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Navigation Buttons */}
        <div className="mt-8 flex items-center justify-between">
          <button
            onClick={handleBack}
            disabled={currentStep === 1}
            className="btn-secondary flex items-center gap-2"
          >
            <ChevronLeft size={20} />
            Previous
          </button>

          <div className="flex items-center gap-4">
            <button className="btn-secondary flex items-center gap-2">
              <Save size={20} />
              Save Draft
            </button>

            {currentStep < 6 && (
              <button onClick={handleNext} className="btn-primary flex items-center gap-2">
                Next
                <ChevronRight size={20} />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// ============= HELPER COMPONENTS =============

interface FormFieldProps {
  label: string;
  required?: boolean;
  helpText?: string;
  children: React.ReactNode;
  className?: string;
}

function FormField({
  label,
  required,
  helpText,
  children,
  className = '',
}: FormFieldProps) {
  return (
    <div className={className}>
      <label className="block text-sm font-semibold text-navy-900 mb-2">
        {label}
        {required && <span className="text-red-500 ml-1">*</span>}
      </label>
      {children}
      {helpText && <p className="mt-1 text-sm text-slate-500">{helpText}</p>}
    </div>
  );
}

interface CurrencyInputProps {
  value: number;
  onChange: (value: number) => void;
}

function CurrencyInput({ value, onChange }: CurrencyInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const numValue = parseFloat(e.target.value) || 0;
    onChange(numValue);
  };

  return (
    <div className="relative">
      <span className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 font-semibold">
        $
      </span>
      <input
        type="number"
        value={value}
        onChange={handleChange}
        className="form-input pl-8"
        placeholder="0.00"
        min="0"
        step="0.01"
      />
    </div>
  );
}

interface ReviewSectionProps {
  title: string;
  children: React.ReactNode;
}

function ReviewSection({ title, children }: ReviewSectionProps) {
  return (
    <div className="bg-slate-50 rounded-lg p-6 border-2 border-slate-200">
      <h3 className="text-lg font-bold text-navy-900 mb-4">{title}</h3>
      <div className="space-y-3">{children}</div>
    </div>
  );
}

interface ReviewItemProps {
  label: string;
  value: string;
  highlight?: boolean;
  large?: boolean;
}

function ReviewItem({ label, value, highlight, large }: ReviewItemProps) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-slate-600 font-medium">{label}:</span>
      <span
        className={`font-semibold ${
          large ? 'text-2xl' : 'text-base'
        } ${
          highlight ? 'text-navy-900' : 'text-slate-900'
        }`}
      >
        {value}
      </span>
    </div>
  );
}

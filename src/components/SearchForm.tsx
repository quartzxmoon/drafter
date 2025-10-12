// Search form component for PA eDocket Desktop

import React from 'react';
import { UseFormReturn } from 'react-hook-form';
import { Search, Calendar, MapPin, Hash, User } from 'lucide-react';

interface SearchFormProps {
  form: UseFormReturn<any>;
  onSubmit: (data: any) => void;
  showAdvanced: boolean;
  isLoading: boolean;
}

export const SearchForm: React.FC<SearchFormProps> = ({
  form,
  onSubmit,
  showAdvanced,
  isLoading,
}) => {
  const { register, handleSubmit, formState: { errors }, watch, setValue } = form;
  
  const courtLevels = [
    { value: '', label: 'All Courts' },
    { value: 'MDJ', label: 'Magisterial District Courts' },
    { value: 'CP', label: 'Court of Common Pleas' },
    { value: 'APP', label: 'Appellate Courts' },
  ];
  
  const counties = [
    { value: '', label: 'All Counties' },
    { value: 'Adams', label: 'Adams' },
    { value: 'Allegheny', label: 'Allegheny' },
    { value: 'Armstrong', label: 'Armstrong' },
    { value: 'Beaver', label: 'Beaver' },
    { value: 'Bedford', label: 'Bedford' },
    { value: 'Berks', label: 'Berks' },
    { value: 'Blair', label: 'Blair' },
    { value: 'Bradford', label: 'Bradford' },
    { value: 'Bucks', label: 'Bucks' },
    { value: 'Butler', label: 'Butler' },
    { value: 'Cambria', label: 'Cambria' },
    { value: 'Cameron', label: 'Cameron' },
    { value: 'Carbon', label: 'Carbon' },
    { value: 'Centre', label: 'Centre' },
    { value: 'Chester', label: 'Chester' },
    { value: 'Clarion', label: 'Clarion' },
    { value: 'Clearfield', label: 'Clearfield' },
    { value: 'Clinton', label: 'Clinton' },
    { value: 'Columbia', label: 'Columbia' },
    { value: 'Crawford', label: 'Crawford' },
    { value: 'Cumberland', label: 'Cumberland' },
    { value: 'Dauphin', label: 'Dauphin' },
    { value: 'Delaware', label: 'Delaware' },
    { value: 'Elk', label: 'Elk' },
    { value: 'Erie', label: 'Erie' },
    { value: 'Fayette', label: 'Fayette' },
    { value: 'Forest', label: 'Forest' },
    { value: 'Franklin', label: 'Franklin' },
    { value: 'Fulton', label: 'Fulton' },
    { value: 'Greene', label: 'Greene' },
    { value: 'Huntingdon', label: 'Huntingdon' },
    { value: 'Indiana', label: 'Indiana' },
    { value: 'Jefferson', label: 'Jefferson' },
    { value: 'Juniata', label: 'Juniata' },
    { value: 'Lackawanna', label: 'Lackawanna' },
    { value: 'Lancaster', label: 'Lancaster' },
    { value: 'Lawrence', label: 'Lawrence' },
    { value: 'Lebanon', label: 'Lebanon' },
    { value: 'Lehigh', label: 'Lehigh' },
    { value: 'Luzerne', label: 'Luzerne' },
    { value: 'Lycoming', label: 'Lycoming' },
    { value: 'McKean', label: 'McKean' },
    { value: 'Mercer', label: 'Mercer' },
    { value: 'Mifflin', label: 'Mifflin' },
    { value: 'Monroe', label: 'Monroe' },
    { value: 'Montgomery', label: 'Montgomery' },
    { value: 'Montour', label: 'Montour' },
    { value: 'Northampton', label: 'Northampton' },
    { value: 'Northumberland', label: 'Northumberland' },
    { value: 'Perry', label: 'Perry' },
    { value: 'Philadelphia', label: 'Philadelphia' },
    { value: 'Pike', label: 'Pike' },
    { value: 'Potter', label: 'Potter' },
    { value: 'Schuylkill', label: 'Schuylkill' },
    { value: 'Snyder', label: 'Snyder' },
    { value: 'Somerset', label: 'Somerset' },
    { value: 'Sullivan', label: 'Sullivan' },
    { value: 'Susquehanna', label: 'Susquehanna' },
    { value: 'Tioga', label: 'Tioga' },
    { value: 'Union', label: 'Union' },
    { value: 'Venango', label: 'Venango' },
    { value: 'Warren', label: 'Warren' },
    { value: 'Washington', label: 'Washington' },
    { value: 'Wayne', label: 'Wayne' },
    { value: 'Westmoreland', label: 'Westmoreland' },
    { value: 'Wyoming', label: 'Wyoming' },
    { value: 'York', label: 'York' },
  ];
  
  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
      {/* Basic Search */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {/* Party Name / General Search */}
        <div>
          <label htmlFor="term" className="block text-sm font-medium text-gray-700 mb-1">
            <User className="inline h-4 w-4 mr-1" />
            Party Name or General Search
          </label>
          <input
            {...register('term')}
            type="text"
            id="term"
            placeholder="Enter party name or search term"
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
          />
          {errors.term && (
            <p className="mt-1 text-sm text-red-600">{errors.term.message}</p>
          )}
        </div>
        
        {/* Court Level */}
        <div>
          <label htmlFor="court" className="block text-sm font-medium text-gray-700 mb-1">
            Court Level
          </label>
          <select
            {...register('court')}
            id="court"
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
          >
            {courtLevels.map(court => (
              <option key={court.value} value={court.value}>
                {court.label}
              </option>
            ))}
          </select>
        </div>
        
        {/* County */}
        <div>
          <label htmlFor="county" className="block text-sm font-medium text-gray-700 mb-1">
            <MapPin className="inline h-4 w-4 mr-1" />
            County
          </label>
          <select
            {...register('county')}
            id="county"
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
          >
            {counties.map(county => (
              <option key={county.value} value={county.value}>
                {county.label}
              </option>
            ))}
          </select>
        </div>
      </div>
      
      {/* Advanced Filters */}
      {showAdvanced && (
        <div className="border-t pt-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">Advanced Search</h3>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {/* Docket Number */}
            <div>
              <label htmlFor="docket" className="block text-sm font-medium text-gray-700 mb-1">
                <Hash className="inline h-4 w-4 mr-1" />
                Docket Number
              </label>
              <input
                {...register('docket')}
                type="text"
                id="docket"
                placeholder="CP-51-CR-1234567-2024"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            
            {/* OTN */}
            <div>
              <label htmlFor="otn" className="block text-sm font-medium text-gray-700 mb-1">
                OTN (Originating Tracking Number)
              </label>
              <input
                {...register('otn')}
                type="text"
                id="otn"
                placeholder="A 12345678-9"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            
            {/* SID */}
            <div>
              <label htmlFor="sid" className="block text-sm font-medium text-gray-700 mb-1">
                SID (State ID Number)
              </label>
              <input
                {...register('sid')}
                type="text"
                id="sid"
                placeholder="A1234567"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            
            {/* Date Range */}
            <div>
              <label htmlFor="from" className="block text-sm font-medium text-gray-700 mb-1">
                <Calendar className="inline h-4 w-4 mr-1" />
                Filed From
              </label>
              <input
                {...register('from')}
                type="date"
                id="from"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            
            <div>
              <label htmlFor="to" className="block text-sm font-medium text-gray-700 mb-1">
                <Calendar className="inline h-4 w-4 mr-1" />
                Filed To
              </label>
              <input
                {...register('to')}
                type="date"
                id="to"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
          </div>
        </div>
      )}
      
      {/* Submit Button */}
      <div className="flex justify-end">
        <button
          type="submit"
          disabled={isLoading}
          className="flex items-center px-6 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
              Searching...
            </>
          ) : (
            <>
              <Search className="h-4 w-4 mr-2" />
              Search
            </>
          )}
        </button>
      </div>
    </form>
  );
};

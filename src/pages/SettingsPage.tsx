// Settings page for PA eDocket Desktop

import React from 'react';

export const SettingsPage: React.FC = () => {
  return (
    <div className="space-y-6">
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">
          Settings
        </h2>
        <p className="text-gray-500">
          This page will provide application settings including API credentials, court preferences, and export options.
        </p>
      </div>
    </div>
  );
};

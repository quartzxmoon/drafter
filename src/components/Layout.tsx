// Layout component for PA eDocket Desktop

import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  Search, 
  FileText, 
  Edit, 
  Upload, 
  Settings, 
  Scale,
  Bell,
  Download
} from 'lucide-react';

interface LayoutProps {
  children: React.ReactNode;
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const location = useLocation();
  
  const navigation = [
    { name: 'Search', href: '/search', icon: Search },
    { name: 'Drafting', href: '/drafting', icon: Edit },
    { name: 'E-Filing', href: '/efiling', icon: Upload },
    { name: 'Settings', href: '/settings', icon: Settings },
  ];
  
  const isActive = (href: string) => {
    return location.pathname === href || (href === '/search' && location.pathname === '/');
  };
  
  return (
    <div className="min-h-screen bg-gray-50 flex">
      {/* Sidebar */}
      <div className="w-64 bg-white shadow-lg">
        <div className="flex items-center px-6 py-4 border-b">
          <Scale className="h-8 w-8 text-blue-600" />
          <span className="ml-2 text-xl font-bold text-gray-900">PA eDocket</span>
        </div>
        
        <nav className="mt-6">
          {navigation.map((item) => {
            const Icon = item.icon;
            return (
              <Link
                key={item.name}
                to={item.href}
                className={`flex items-center px-6 py-3 text-sm font-medium transition-colors ${
                  isActive(item.href)
                    ? 'bg-blue-50 text-blue-700 border-r-2 border-blue-700'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                }`}
              >
                <Icon className="h-5 w-5 mr-3" />
                {item.name}
              </Link>
            );
          })}
        </nav>
        
        {/* Quick Actions */}
        <div className="mt-8 px-6">
          <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
            Quick Actions
          </h3>
          <div className="mt-3 space-y-2">
            <button className="flex items-center w-full px-3 py-2 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-md">
              <Bell className="h-4 w-4 mr-2" />
              Watchlist
            </button>
            <button className="flex items-center w-full px-3 py-2 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-md">
              <Download className="h-4 w-4 mr-2" />
              Recent Exports
            </button>
            <button className="flex items-center w-full px-3 py-2 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-50 rounded-md">
              <FileText className="h-4 w-4 mr-2" />
              Templates
            </button>
          </div>
        </div>
      </div>
      
      {/* Main content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <header className="bg-white shadow-sm border-b">
          <div className="px-6 py-4">
            <div className="flex items-center justify-between">
              <h1 className="text-2xl font-semibold text-gray-900">
                {getPageTitle(location.pathname)}
              </h1>
              
              <div className="flex items-center space-x-4">
                {/* Status indicator */}
                <div className="flex items-center">
                  <div className="h-2 w-2 bg-green-400 rounded-full"></div>
                  <span className="ml-2 text-sm text-gray-600">Connected</span>
                </div>
                
                {/* User menu placeholder */}
                <div className="h-8 w-8 bg-gray-300 rounded-full"></div>
              </div>
            </div>
          </div>
        </header>
        
        {/* Page content */}
        <main className="flex-1 overflow-auto">
          <div className="p-6">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
};

function getPageTitle(pathname: string): string {
  switch (pathname) {
    case '/':
    case '/search':
      return 'Search Dockets';
    case '/drafting':
      return 'Document Drafting';
    case '/efiling':
      return 'E-Filing';
    case '/settings':
      return 'Settings';
    default:
      if (pathname.startsWith('/docket/')) {
        return 'Docket Details';
      }
      return 'PA eDocket Desktop';
  }
}

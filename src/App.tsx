import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Layout } from './components/Layout';
import { SearchPage } from './pages/SearchPage';
import { DocketPage } from './pages/DocketPage';
import { DraftingPage } from './pages/DraftingPage';
import { EFilingPage } from './pages/EFilingPage';
import { SettingsPage } from './pages/SettingsPage';
import './App.css';

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<SearchPage />} />
          <Route path="/search" element={<SearchPage />} />
          <Route path="/docket/:id" element={<DocketPage />} />
          <Route path="/drafting" element={<DraftingPage />} />
          <Route path="/efiling" element={<EFilingPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;

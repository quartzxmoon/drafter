// Services module for PA eDocket Desktop
// Contains business logic and command handlers

// Core Services
pub mod automation;
pub mod citations;
pub mod commands;
pub mod court_rules;
pub mod database;
pub mod drafting;
pub mod export;
pub mod security;
pub mod task_runner;
pub mod watchlist;
pub mod case_management;
pub mod pleading_formatter;
pub mod ai_citation_service;

// Tier 1: Core Revenue Features (10 features)
pub mod document_assembly;       // Feature #1 - AI Document Assembly
pub mod conflict_checking;       // Feature #2 - Conflict Checking
pub mod time_tracking;           // Feature #3 - Time Tracking
pub mod billing;                 // Feature #4 - Billing & Invoicing
pub mod email_integration;       // Feature #5 - Email Integration
pub mod contract_review;         // Feature #6 - Contract Review AI
pub mod legal_research;          // Feature #7 - Legal Research
pub mod settlement_calculator;   // Feature #8 - Settlement Calculator (FLAGSHIP)
pub mod export_settlement;       // Settlement export utilities
pub mod speech_to_text;          // Feature #9 - Speech-to-Text
pub mod expert_witness;          // Feature #10 - Expert Witness Management
pub mod discovery;               // Feature #11 - Discovery Management

// Tier 2: Competitive Advantage (10 features)
pub mod court_filing;            // Feature #12 - Court E-Filing
pub mod crm;                     // Feature #13 - CRM & Client Intake
pub mod marketing;               // Feature #14 - Legal Marketing Suite
// court_rules already declared above  // Feature #15 - Court Rules Database
pub mod collaboration;           // Feature #16 - Client Collaboration Portal
pub mod jury_selection;          // Feature #17 - Jury Selection AI
pub mod analytics;               // Feature #18 - Legal Analytics Dashboard
pub mod i18n;                    // Feature #19 - Multi-Language Support
pub mod compliance;              // Feature #20 - IOLTA Compliance
pub mod immigration;             // Feature #21 - Immigration Law Toolkit

// Tier 3: Innovation Features (10 features)
pub mod real_estate;             // Feature #22 - Real Estate Toolkit
pub mod estate_planning;         // Feature #23 - Estate Planning
pub mod workers_comp;            // Feature #24 - Workers' Compensation
pub mod patent;                  // Feature #25 - Patent & Trademark
pub mod mediation;               // Feature #26 - Mediation & ADR
pub mod predictive;              // Feature #27 - Predictive Analytics
pub mod blockchain;              // Feature #28 - Blockchain Smart Contracts
pub mod chatbot;                 // Feature #29 - Virtual Legal Assistant
pub mod cybersecurity;           // Feature #30 - Cybersecurity Compliance
pub mod knowledge_base;          // Feature #31 - Knowledge Management

// Production Infrastructure (3 NEW features)
pub mod bulk_data_ingestion;     // Feature #32 - Bulk Data Ingestion (CRITICAL)
pub mod ai_automation_suite;     // Feature #33 - AI Automation Suite (GAME CHANGER)
// REST API in separate api/rest_api.rs module

// Additional Support Services
pub mod bulk_import_service;
pub mod speech_recognition;
pub mod ai_research_assistant;
pub mod document_comparison;
pub mod ai_legal_research;
pub mod esignature;
pub mod calendar_sync;
pub mod client_portal;

// Re-export commonly used types
pub use commands::*;

// Billing Service - Invoice generation, payment processing, and trust accounting
// Supports Stripe/LawPay integration and IOLTA compliance

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Pending,
    Sent,
    Viewed,
    PartiallyPaid,
    Paid,
    Overdue,
    Cancelled,
    WriteOff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentMethod {
    Cash,
    Check,
    CreditCard,
    BankTransfer,
    LawPay,
    Stripe,
    Trust,             // Payment from trust account
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
    Disputed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub invoice_number: String,
    pub matter_id: String,
    pub matter_name: String,
    pub client_id: String,
    pub client_name: String,

    // Billing details
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub issue_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,

    // Line items
    pub time_entries: Vec<InvoiceTimeEntry>,
    pub expenses: Vec<InvoiceExpense>,
    pub adjustments: Vec<InvoiceAdjustment>,

    // Amounts
    pub subtotal: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub amount_paid: f64,
    pub balance: f64,

    // Status
    pub status: InvoiceStatus,
    pub sent_at: Option<DateTime<Utc>>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,

    // Metadata
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceTimeEntry {
    pub time_entry_id: String,
    pub date: DateTime<Utc>,
    pub attorney_name: String,
    pub activity_description: String,
    pub hours: f64,
    pub rate: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceExpense {
    pub expense_id: String,
    pub date: DateTime<Utc>,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub is_reimbursable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceAdjustment {
    pub description: String,
    pub amount: f64,
    pub is_credit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub invoice_id: String,
    pub matter_id: String,
    pub client_id: String,

    // Payment details
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub payment_date: DateTime<Utc>,
    pub reference_number: Option<String>,

    // Processing
    pub status: PaymentStatus,
    pub processor_transaction_id: Option<String>,
    pub processor_fee: Option<f64>,

    // Trust account
    pub from_trust_account: bool,
    pub trust_transaction_id: Option<String>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: String,
    pub matter_id: String,
    pub attorney_id: String,

    // Expense details
    pub date: DateTime<Utc>,
    pub category: ExpenseCategory,
    pub description: String,
    pub amount: f64,
    pub is_reimbursable: bool,
    pub is_billable: bool,

    // Receipt
    pub receipt_url: Option<String>,
    pub vendor: Option<String>,

    // Status
    pub status: ExpenseStatus,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
    pub billed_at: Option<DateTime<Utc>>,
    pub invoice_id: Option<String>,
    pub reimbursed_at: Option<DateTime<Utc>>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpenseCategory {
    Travel,
    Filing_fees,
    Expert_witness,
    Court_reporter,
    Copying,
    Postage,
    Research,
    Meals,
    Parking,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpenseStatus {
    Pending,
    Approved,
    Rejected,
    Billed,
    Reimbursed,
}

// ============= Trust Accounting (IOLTA Compliance) =============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustTransactionType {
    Deposit,           // Client funds received
    Withdrawal,        // Payment made on behalf of client
    Transfer_in,       // Transfer from another trust account
    Transfer_out,      // Transfer to another trust account
    Interest,          // Interest earned (IOLTA)
    Fee_transfer,      // Transfer to operating account for earned fees
    Refund,            // Refund to client
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAccount {
    pub id: String,
    pub account_name: String,
    pub account_number: String,
    pub bank_name: String,
    pub routing_number: String,
    pub account_type: String,  // IOLTA, Non-IOLTA
    pub current_balance: f64,
    pub is_active: bool,
    pub opened_date: DateTime<Utc>,
    pub closed_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustTransaction {
    pub id: String,
    pub trust_account_id: String,
    pub matter_id: String,
    pub client_id: String,

    // Transaction details
    pub transaction_type: TrustTransactionType,
    pub transaction_date: DateTime<Utc>,
    pub amount: f64,
    pub description: String,
    pub reference_number: Option<String>,

    // Reconciliation
    pub is_reconciled: bool,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub bank_statement_date: Option<DateTime<Utc>>,

    // Related records
    pub invoice_id: Option<String>,
    pub payment_id: Option<String>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTrustBalance {
    pub client_id: String,
    pub client_name: String,
    pub matter_id: String,
    pub matter_name: String,
    pub balance: f64,
    pub last_transaction_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustReconciliation {
    pub id: String,
    pub trust_account_id: String,
    pub reconciliation_date: DateTime<Utc>,
    pub statement_date: DateTime<Utc>,

    // Balances
    pub statement_balance: f64,
    pub book_balance: f64,
    pub difference: f64,

    // Transactions
    pub unreconciled_deposits: Vec<TrustTransaction>,
    pub unreconciled_withdrawals: Vec<TrustTransaction>,

    // Status
    pub is_reconciled: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

// ============= Payment Processing Integration =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessor {
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
    pub is_test_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub invoice_id: String,
    pub amount: f64,
    pub currency: String,
    pub payment_method: PaymentMethod,
    pub processor: String,
    pub processor_intent_id: Option<String>,
    pub status: String,
    pub client_secret: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct BillingService {
    db: SqlitePool,
}

impl BillingService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ============= Invoice Management =============

    /// Create a new invoice from time entries and expenses
    pub async fn create_invoice(
        &self,
        matter_id: &str,
        client_id: &str,
        billing_period_start: DateTime<Utc>,
        billing_period_end: DateTime<Utc>,
        time_entry_ids: Vec<String>,
        expense_ids: Vec<String>,
        due_days: i64,
        created_by: &str,
    ) -> Result<Invoice> {
        let invoice_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let due_date = now + chrono::Duration::days(due_days);

        // Generate invoice number
        let invoice_number = self.generate_invoice_number().await?;

        // Get matter and client names
        let matter_name = self.get_matter_name(matter_id).await?;
        let client_name = self.get_client_name(client_id).await?;

        // Fetch time entries
        let time_entries = self.fetch_time_entries_for_invoice(&time_entry_ids).await?;

        // Fetch expenses
        let expenses = self.fetch_expenses_for_invoice(&expense_ids).await?;

        // Calculate totals
        let time_total: f64 = time_entries.iter().map(|e| e.amount).sum();
        let expense_total: f64 = expenses.iter().map(|e| e.amount).sum();
        let subtotal = time_total + expense_total;

        let invoice = Invoice {
            id: invoice_id.clone(),
            invoice_number,
            matter_id: matter_id.to_string(),
            matter_name,
            client_id: client_id.to_string(),
            client_name,
            billing_period_start,
            billing_period_end,
            issue_date: now,
            due_date,
            time_entries,
            expenses,
            adjustments: Vec::new(),
            subtotal,
            discount_amount: 0.0,
            tax_amount: 0.0,
            total: subtotal,
            amount_paid: 0.0,
            balance: subtotal,
            status: InvoiceStatus::Draft,
            sent_at: None,
            viewed_at: None,
            paid_at: None,
            notes: None,
            terms: Some("Payment due within 30 days".to_string()),
            created_at: now,
            updated_at: now,
            created_by: created_by.to_string(),
        };

        self.save_invoice(&invoice).await?;

        // Mark time entries and expenses as billed
        self.mark_time_entries_billed(&time_entry_ids, &invoice.id).await?;
        self.mark_expenses_billed(&expense_ids, &invoice.id).await?;

        Ok(invoice)
    }

    /// Update invoice
    pub async fn update_invoice(
        &self,
        invoice_id: &str,
        adjustments: Option<Vec<InvoiceAdjustment>>,
        discount_amount: Option<f64>,
        tax_amount: Option<f64>,
        notes: Option<String>,
        terms: Option<String>,
    ) -> Result<Invoice> {
        let mut invoice = self.get_invoice(invoice_id).await?;

        if let Some(adj) = adjustments {
            invoice.adjustments = adj;
        }

        if let Some(discount) = discount_amount {
            invoice.discount_amount = discount;
        }

        if let Some(tax) = tax_amount {
            invoice.tax_amount = tax;
        }

        if notes.is_some() {
            invoice.notes = notes;
        }

        if terms.is_some() {
            invoice.terms = terms;
        }

        // Recalculate total
        let adjustments_total: f64 = invoice.adjustments.iter()
            .map(|a| if a.is_credit { -a.amount } else { a.amount })
            .sum();

        invoice.total = invoice.subtotal + adjustments_total - invoice.discount_amount + invoice.tax_amount;
        invoice.balance = invoice.total - invoice.amount_paid;
        invoice.updated_at = Utc::now();

        self.save_invoice(&invoice).await?;

        Ok(invoice)
    }

    /// Send invoice to client
    pub async fn send_invoice(&self, invoice_id: &str) -> Result<Invoice> {
        let mut invoice = self.get_invoice(invoice_id).await?;

        if invoice.status == InvoiceStatus::Draft {
            invoice.status = InvoiceStatus::Sent;
            invoice.sent_at = Some(Utc::now());
            invoice.updated_at = Utc::now();

            self.save_invoice(&invoice).await?;

            // TODO: Send email to client
            // self.send_invoice_email(&invoice).await?;
        }

        Ok(invoice)
    }

    /// Record invoice viewed by client
    pub async fn mark_invoice_viewed(&self, invoice_id: &str) -> Result<Invoice> {
        let mut invoice = self.get_invoice(invoice_id).await?;

        if invoice.viewed_at.is_none() {
            invoice.viewed_at = Some(Utc::now());
            invoice.status = InvoiceStatus::Viewed;
            invoice.updated_at = Utc::now();

            self.save_invoice(&invoice).await?;
        }

        Ok(invoice)
    }

    /// Cancel invoice
    pub async fn cancel_invoice(&self, invoice_id: &str) -> Result<Invoice> {
        let mut invoice = self.get_invoice(invoice_id).await?;

        if invoice.status == InvoiceStatus::Paid {
            return Err(anyhow::anyhow!("Cannot cancel paid invoice"));
        }

        invoice.status = InvoiceStatus::Cancelled;
        invoice.updated_at = Utc::now();

        self.save_invoice(&invoice).await?;

        // Unmark time entries and expenses as billed
        let time_entry_ids: Vec<String> = invoice.time_entries.iter()
            .map(|e| e.time_entry_id.clone())
            .collect();
        self.unmark_time_entries_billed(&time_entry_ids).await?;

        let expense_ids: Vec<String> = invoice.expenses.iter()
            .map(|e| e.expense_id.clone())
            .collect();
        self.unmark_expenses_billed(&expense_ids).await?;

        Ok(invoice)
    }

    // ============= Payment Processing =============

    /// Record a payment
    pub async fn record_payment(
        &self,
        invoice_id: &str,
        amount: f64,
        payment_method: PaymentMethod,
        payment_date: DateTime<Utc>,
        reference_number: Option<String>,
        from_trust: bool,
        created_by: &str,
    ) -> Result<Payment> {
        let invoice = self.get_invoice(invoice_id).await?;

        if amount <= 0.0 {
            return Err(anyhow::anyhow!("Payment amount must be positive"));
        }

        if amount > invoice.balance {
            return Err(anyhow::anyhow!("Payment amount exceeds invoice balance"));
        }

        let payment_id = Uuid::new_v4().to_string();

        let payment = Payment {
            id: payment_id.clone(),
            invoice_id: invoice_id.to_string(),
            matter_id: invoice.matter_id.clone(),
            client_id: invoice.client_id.clone(),
            amount,
            payment_method,
            payment_date,
            reference_number,
            status: PaymentStatus::Completed,
            processor_transaction_id: None,
            processor_fee: None,
            from_trust_account: from_trust,
            trust_transaction_id: None,
            notes: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        self.save_payment(&payment).await?;

        // Update invoice
        self.apply_payment_to_invoice(invoice_id, amount).await?;

        // If from trust account, create trust transaction
        if from_trust {
            self.create_trust_withdrawal_for_payment(&payment).await?;
        }

        Ok(payment)
    }

    /// Process payment via Stripe
    pub async fn process_stripe_payment(
        &self,
        invoice_id: &str,
        payment_method_id: &str,
        amount: f64,
        created_by: &str,
    ) -> Result<Payment> {
        // This is a stub - real implementation would call Stripe API
        let payment_id = Uuid::new_v4().to_string();
        let invoice = self.get_invoice(invoice_id).await?;

        let payment = Payment {
            id: payment_id.clone(),
            invoice_id: invoice_id.to_string(),
            matter_id: invoice.matter_id.clone(),
            client_id: invoice.client_id.clone(),
            amount,
            payment_method: PaymentMethod::Stripe,
            payment_date: Utc::now(),
            reference_number: Some(payment_method_id.to_string()),
            status: PaymentStatus::Processing,
            processor_transaction_id: Some(format!("stripe_{}", Uuid::new_v4())),
            processor_fee: Some(amount * 0.029 + 0.30), // Stripe fee: 2.9% + $0.30
            from_trust_account: false,
            trust_transaction_id: None,
            notes: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        self.save_payment(&payment).await?;

        // Simulate successful processing
        self.complete_payment(&payment.id).await?;

        Ok(payment)
    }

    /// Process payment via LawPay
    pub async fn process_lawpay_payment(
        &self,
        invoice_id: &str,
        payment_method_id: &str,
        amount: f64,
        created_by: &str,
    ) -> Result<Payment> {
        // This is a stub - real implementation would call LawPay API
        let payment_id = Uuid::new_v4().to_string();
        let invoice = self.get_invoice(invoice_id).await?;

        let payment = Payment {
            id: payment_id.clone(),
            invoice_id: invoice_id.to_string(),
            matter_id: invoice.matter_id.clone(),
            client_id: invoice.client_id.clone(),
            amount,
            payment_method: PaymentMethod::LawPay,
            payment_date: Utc::now(),
            reference_number: Some(payment_method_id.to_string()),
            status: PaymentStatus::Processing,
            processor_transaction_id: Some(format!("lawpay_{}", Uuid::new_v4())),
            processor_fee: Some(amount * 0.025), // LawPay fee: 2.5%
            from_trust_account: false,
            trust_transaction_id: None,
            notes: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        self.save_payment(&payment).await?;

        // Simulate successful processing
        self.complete_payment(&payment.id).await?;

        Ok(payment)
    }

    async fn complete_payment(&self, payment_id: &str) -> Result<Payment> {
        let mut payment = self.get_payment(payment_id).await?;
        payment.status = PaymentStatus::Completed;

        self.save_payment(&payment).await?;
        self.apply_payment_to_invoice(&payment.invoice_id, payment.amount).await?;

        Ok(payment)
    }

    async fn apply_payment_to_invoice(&self, invoice_id: &str, amount: f64) -> Result<()> {
        let mut invoice = self.get_invoice(invoice_id).await?;

        invoice.amount_paid += amount;
        invoice.balance -= amount;

        if invoice.balance <= 0.0 {
            invoice.status = InvoiceStatus::Paid;
            invoice.paid_at = Some(Utc::now());
        } else {
            invoice.status = InvoiceStatus::PartiallyPaid;
        }

        invoice.updated_at = Utc::now();

        self.save_invoice(&invoice).await?;

        Ok(())
    }

    // ============= Trust Accounting =============

    /// Create trust deposit
    pub async fn create_trust_deposit(
        &self,
        trust_account_id: &str,
        matter_id: &str,
        client_id: &str,
        amount: f64,
        description: &str,
        reference_number: Option<String>,
        created_by: &str,
    ) -> Result<TrustTransaction> {
        let transaction_id = Uuid::new_v4().to_string();

        let transaction = TrustTransaction {
            id: transaction_id.clone(),
            trust_account_id: trust_account_id.to_string(),
            matter_id: matter_id.to_string(),
            client_id: client_id.to_string(),
            transaction_type: TrustTransactionType::Deposit,
            transaction_date: Utc::now(),
            amount,
            description: description.to_string(),
            reference_number,
            is_reconciled: false,
            reconciled_at: None,
            bank_statement_date: None,
            invoice_id: None,
            payment_id: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        self.save_trust_transaction(&transaction).await?;

        // Update trust account balance
        self.update_trust_account_balance(trust_account_id, amount).await?;

        Ok(transaction)
    }

    /// Create trust withdrawal
    pub async fn create_trust_withdrawal(
        &self,
        trust_account_id: &str,
        matter_id: &str,
        client_id: &str,
        amount: f64,
        description: &str,
        reference_number: Option<String>,
        created_by: &str,
    ) -> Result<TrustTransaction> {
        // Check sufficient balance
        let client_balance = self.get_client_trust_balance(client_id, matter_id).await?;
        if client_balance < amount {
            return Err(anyhow::anyhow!("Insufficient trust balance for client"));
        }

        let transaction_id = Uuid::new_v4().to_string();

        let transaction = TrustTransaction {
            id: transaction_id.clone(),
            trust_account_id: trust_account_id.to_string(),
            matter_id: matter_id.to_string(),
            client_id: client_id.to_string(),
            transaction_type: TrustTransactionType::Withdrawal,
            transaction_date: Utc::now(),
            amount: -amount, // Negative for withdrawal
            description: description.to_string(),
            reference_number,
            is_reconciled: false,
            reconciled_at: None,
            bank_statement_date: None,
            invoice_id: None,
            payment_id: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        self.save_trust_transaction(&transaction).await?;

        // Update trust account balance
        self.update_trust_account_balance(trust_account_id, -amount).await?;

        Ok(transaction)
    }

    async fn create_trust_withdrawal_for_payment(&self, payment: &Payment) -> Result<TrustTransaction> {
        // Get default trust account
        let trust_account = self.get_default_trust_account().await?;

        let transaction = TrustTransaction {
            id: Uuid::new_v4().to_string(),
            trust_account_id: trust_account.id,
            matter_id: payment.matter_id.clone(),
            client_id: payment.client_id.clone(),
            transaction_type: TrustTransactionType::Withdrawal,
            transaction_date: payment.payment_date,
            amount: -payment.amount,
            description: format!("Payment for invoice {}", payment.invoice_id),
            reference_number: payment.reference_number.clone(),
            is_reconciled: false,
            reconciled_at: None,
            bank_statement_date: None,
            invoice_id: Some(payment.invoice_id.clone()),
            payment_id: Some(payment.id.clone()),
            created_at: Utc::now(),
            created_by: payment.created_by.clone(),
        };

        self.save_trust_transaction(&transaction).await?;

        Ok(transaction)
    }

    /// Get client trust balance
    pub async fn get_client_trust_balance(&self, client_id: &str, matter_id: &str) -> Result<f64> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as balance
            FROM trust_transactions
            WHERE client_id = ? AND matter_id = ?
            "#,
            client_id,
            matter_id
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to query trust balance")?;

        Ok(result.balance.unwrap_or(0.0))
    }

    /// Get all client trust balances
    pub async fn get_all_trust_balances(&self) -> Result<Vec<ClientTrustBalance>> {
        let results = sqlx::query_as!(
            ClientTrustBalance,
            r#"
            SELECT
                client_id,
                client_name,
                matter_id,
                matter_name,
                SUM(amount) as balance,
                MAX(transaction_date) as last_transaction_date
            FROM trust_transactions t
            JOIN matters m ON t.matter_id = m.id
            JOIN clients c ON t.client_id = c.id
            GROUP BY client_id, matter_id
            HAVING balance > 0
            ORDER BY client_name, matter_name
            "#
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to query trust balances")?;

        Ok(results)
    }

    /// Three-way reconciliation: Book balance = Bank balance = Client balances sum
    pub async fn perform_three_way_reconciliation(
        &self,
        trust_account_id: &str,
        statement_date: DateTime<Utc>,
        statement_balance: f64,
    ) -> Result<TrustReconciliation> {
        let reconciliation_id = Uuid::new_v4().to_string();

        // Get book balance from transactions
        let book_balance = self.get_trust_account_book_balance(trust_account_id).await?;

        // Get sum of all client balances
        let client_balances_sum = self.get_client_balances_sum().await?;

        // Check three-way reconciliation
        let difference = (statement_balance - book_balance).abs();
        let is_reconciled = difference < 0.01 && (book_balance - client_balances_sum).abs() < 0.01;

        // Get unreconciled transactions
        let unreconciled = self.get_unreconciled_transactions(trust_account_id).await?;

        let (deposits, withdrawals): (Vec<TrustTransaction>, Vec<TrustTransaction>) =
            unreconciled.into_iter().partition(|t| t.amount > 0.0);

        let reconciliation = TrustReconciliation {
            id: reconciliation_id,
            trust_account_id: trust_account_id.to_string(),
            reconciliation_date: Utc::now(),
            statement_date,
            statement_balance,
            book_balance,
            difference,
            unreconciled_deposits: deposits,
            unreconciled_withdrawals: withdrawals,
            is_reconciled,
            notes: if !is_reconciled {
                Some(format!(
                    "Reconciliation failed. Statement: ${:.2}, Book: ${:.2}, Clients: ${:.2}",
                    statement_balance, book_balance, client_balances_sum
                ))
            } else {
                None
            },
            created_at: Utc::now(),
            created_by: "system".to_string(),
        };

        self.save_trust_reconciliation(&reconciliation).await?;

        Ok(reconciliation)
    }

    // ============= Expense Management =============

    /// Create expense
    pub async fn create_expense(
        &self,
        matter_id: &str,
        attorney_id: &str,
        category: ExpenseCategory,
        description: &str,
        amount: f64,
        date: DateTime<Utc>,
        is_billable: bool,
        is_reimbursable: bool,
        receipt_url: Option<String>,
        vendor: Option<String>,
    ) -> Result<Expense> {
        let expense_id = Uuid::new_v4().to_string();

        let expense = Expense {
            id: expense_id,
            matter_id: matter_id.to_string(),
            attorney_id: attorney_id.to_string(),
            date,
            category,
            description: description.to_string(),
            amount,
            is_reimbursable,
            is_billable,
            receipt_url,
            vendor,
            status: ExpenseStatus::Pending,
            approved_at: None,
            approved_by: None,
            billed_at: None,
            invoice_id: None,
            reimbursed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.save_expense(&expense).await?;

        Ok(expense)
    }

    /// Approve expense
    pub async fn approve_expense(&self, expense_id: &str, approved_by: &str) -> Result<Expense> {
        let mut expense = self.get_expense(expense_id).await?;

        expense.status = ExpenseStatus::Approved;
        expense.approved_at = Some(Utc::now());
        expense.approved_by = Some(approved_by.to_string());
        expense.updated_at = Utc::now();

        self.save_expense(&expense).await?;

        Ok(expense)
    }

    // ============= Helper Methods =============

    async fn generate_invoice_number(&self) -> Result<String> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM invoices")
            .fetch_one(&self.db)
            .await?;

        Ok(format!("INV-{:06}", count.count + 1))
    }

    async fn get_matter_name(&self, matter_id: &str) -> Result<String> {
        Ok(format!("Matter {}", matter_id))
    }

    async fn get_client_name(&self, client_id: &str) -> Result<String> {
        Ok(format!("Client {}", client_id))
    }

    async fn fetch_time_entries_for_invoice(&self, entry_ids: &[String]) -> Result<Vec<InvoiceTimeEntry>> {
        // Stub - would query time_entries table
        Ok(Vec::new())
    }

    async fn fetch_expenses_for_invoice(&self, expense_ids: &[String]) -> Result<Vec<InvoiceExpense>> {
        // Stub - would query expenses table
        Ok(Vec::new())
    }

    async fn mark_time_entries_billed(&self, entry_ids: &[String], invoice_id: &str) -> Result<()> {
        // Update time entries to mark as billed
        Ok(())
    }

    async fn mark_expenses_billed(&self, expense_ids: &[String], invoice_id: &str) -> Result<()> {
        // Update expenses to mark as billed
        Ok(())
    }

    async fn unmark_time_entries_billed(&self, entry_ids: &[String]) -> Result<()> {
        // Update time entries to unmark as billed
        Ok(())
    }

    async fn unmark_expenses_billed(&self, expense_ids: &[String]) -> Result<()> {
        // Update expenses to unmark as billed
        Ok(())
    }

    async fn get_default_trust_account(&self) -> Result<TrustAccount> {
        // Stub - would query trust_accounts table
        Ok(TrustAccount {
            id: "default".to_string(),
            account_name: "IOLTA Account".to_string(),
            account_number: "123456789".to_string(),
            bank_name: "Trust Bank".to_string(),
            routing_number: "987654321".to_string(),
            account_type: "IOLTA".to_string(),
            current_balance: 0.0,
            is_active: true,
            opened_date: Utc::now(),
            closed_date: None,
        })
    }

    async fn update_trust_account_balance(&self, account_id: &str, amount: f64) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE trust_accounts
            SET current_balance = current_balance + ?
            WHERE id = ?
            "#,
            amount,
            account_id
        )
        .execute(&self.db)
        .await
        .context("Failed to update trust account balance")?;

        Ok(())
    }

    async fn get_trust_account_book_balance(&self, account_id: &str) -> Result<f64> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as balance
            FROM trust_transactions
            WHERE trust_account_id = ?
            "#,
            account_id
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to query trust account balance")?;

        Ok(result.balance.unwrap_or(0.0))
    }

    async fn get_client_balances_sum(&self) -> Result<f64> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(balance), 0) as total
            FROM (
                SELECT client_id, matter_id, SUM(amount) as balance
                FROM trust_transactions
                GROUP BY client_id, matter_id
            )
            "#
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to sum client balances")?;

        Ok(result.total.unwrap_or(0.0))
    }

    async fn get_unreconciled_transactions(&self, account_id: &str) -> Result<Vec<TrustTransaction>> {
        let results = sqlx::query_as!(
            TrustTransaction,
            r#"
            SELECT id, trust_account_id, matter_id, client_id,
                   transaction_type as "transaction_type: _",
                   transaction_date, amount, description, reference_number,
                   is_reconciled, reconciled_at, bank_statement_date,
                   invoice_id, payment_id, created_at, created_by
            FROM trust_transactions
            WHERE trust_account_id = ? AND is_reconciled = 0
            ORDER BY transaction_date
            "#,
            account_id
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to query unreconciled transactions")?;

        Ok(results)
    }

    async fn save_invoice(&self, invoice: &Invoice) -> Result<()> {
        // Serialize complex fields
        let time_entries_json = serde_json::to_string(&invoice.time_entries)?;
        let expenses_json = serde_json::to_string(&invoice.expenses)?;
        let adjustments_json = serde_json::to_string(&invoice.adjustments)?;
        let status_str = format!("{:?}", invoice.status);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO invoices
            (id, invoice_number, matter_id, matter_name, client_id, client_name,
             billing_period_start, billing_period_end, issue_date, due_date,
             time_entries_json, expenses_json, adjustments_json,
             subtotal, discount_amount, tax_amount, total, amount_paid, balance,
             status, sent_at, viewed_at, paid_at, notes, terms,
             created_at, updated_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            invoice.id,
            invoice.invoice_number,
            invoice.matter_id,
            invoice.matter_name,
            invoice.client_id,
            invoice.client_name,
            invoice.billing_period_start,
            invoice.billing_period_end,
            invoice.issue_date,
            invoice.due_date,
            time_entries_json,
            expenses_json,
            adjustments_json,
            invoice.subtotal,
            invoice.discount_amount,
            invoice.tax_amount,
            invoice.total,
            invoice.amount_paid,
            invoice.balance,
            status_str,
            invoice.sent_at,
            invoice.viewed_at,
            invoice.paid_at,
            invoice.notes,
            invoice.terms,
            invoice.created_at,
            invoice.updated_at,
            invoice.created_by
        )
        .execute(&self.db)
        .await
        .context("Failed to save invoice")?;

        Ok(())
    }

    async fn get_invoice(&self, invoice_id: &str) -> Result<Invoice> {
        // Stub - would query invoices table and deserialize JSON fields
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn save_payment(&self, payment: &Payment) -> Result<()> {
        let payment_method_str = format!("{:?}", payment.payment_method);
        let status_str = format!("{:?}", payment.status);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO payments
            (id, invoice_id, matter_id, client_id, amount, payment_method, payment_date,
             reference_number, status, processor_transaction_id, processor_fee,
             from_trust_account, trust_transaction_id, notes, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            payment.id,
            payment.invoice_id,
            payment.matter_id,
            payment.client_id,
            payment.amount,
            payment_method_str,
            payment.payment_date,
            payment.reference_number,
            status_str,
            payment.processor_transaction_id,
            payment.processor_fee,
            payment.from_trust_account,
            payment.trust_transaction_id,
            payment.notes,
            payment.created_at,
            payment.created_by
        )
        .execute(&self.db)
        .await
        .context("Failed to save payment")?;

        Ok(())
    }

    async fn get_payment(&self, payment_id: &str) -> Result<Payment> {
        // Stub - would query payments table
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn save_trust_transaction(&self, transaction: &TrustTransaction) -> Result<()> {
        let transaction_type_str = format!("{:?}", transaction.transaction_type);

        sqlx::query!(
            r#"
            INSERT INTO trust_transactions
            (id, trust_account_id, matter_id, client_id, transaction_type, transaction_date,
             amount, description, reference_number, is_reconciled, reconciled_at,
             bank_statement_date, invoice_id, payment_id, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            transaction.id,
            transaction.trust_account_id,
            transaction.matter_id,
            transaction.client_id,
            transaction_type_str,
            transaction.transaction_date,
            transaction.amount,
            transaction.description,
            transaction.reference_number,
            transaction.is_reconciled,
            transaction.reconciled_at,
            transaction.bank_statement_date,
            transaction.invoice_id,
            transaction.payment_id,
            transaction.created_at,
            transaction.created_by
        )
        .execute(&self.db)
        .await
        .context("Failed to save trust transaction")?;

        Ok(())
    }

    async fn save_trust_reconciliation(&self, reconciliation: &TrustReconciliation) -> Result<()> {
        // Serialize transaction lists
        let deposits_json = serde_json::to_string(&reconciliation.unreconciled_deposits)?;
        let withdrawals_json = serde_json::to_string(&reconciliation.unreconciled_withdrawals)?;

        sqlx::query!(
            r#"
            INSERT INTO trust_reconciliations
            (id, trust_account_id, reconciliation_date, statement_date, statement_balance,
             book_balance, difference, unreconciled_deposits_json, unreconciled_withdrawals_json,
             is_reconciled, notes, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            reconciliation.id,
            reconciliation.trust_account_id,
            reconciliation.reconciliation_date,
            reconciliation.statement_date,
            reconciliation.statement_balance,
            reconciliation.book_balance,
            reconciliation.difference,
            deposits_json,
            withdrawals_json,
            reconciliation.is_reconciled,
            reconciliation.notes,
            reconciliation.created_at,
            reconciliation.created_by
        )
        .execute(&self.db)
        .await
        .context("Failed to save trust reconciliation")?;

        Ok(())
    }

    async fn save_expense(&self, expense: &Expense) -> Result<()> {
        let category_str = format!("{:?}", expense.category);
        let status_str = format!("{:?}", expense.status);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO expenses
            (id, matter_id, attorney_id, date, category, description, amount,
             is_reimbursable, is_billable, receipt_url, vendor, status,
             approved_at, approved_by, billed_at, invoice_id, reimbursed_at,
             created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            expense.id,
            expense.matter_id,
            expense.attorney_id,
            expense.date,
            category_str,
            expense.description,
            expense.amount,
            expense.is_reimbursable,
            expense.is_billable,
            expense.receipt_url,
            expense.vendor,
            status_str,
            expense.approved_at,
            expense.approved_by,
            expense.billed_at,
            expense.invoice_id,
            expense.reimbursed_at,
            expense.created_at,
            expense.updated_at
        )
        .execute(&self.db)
        .await
        .context("Failed to save expense")?;

        Ok(())
    }

    async fn get_expense(&self, expense_id: &str) -> Result<Expense> {
        // Stub - would query expenses table
        Err(anyhow::anyhow!("Not implemented"))
    }
}

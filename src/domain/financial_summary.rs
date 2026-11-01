use crate::domain::transaction::{Transaction, TransactionType};

#[derive(Debug, Clone)]
pub struct FinancialSummary {
    pub total_income: f64,
    pub total_expenses: f64,
    pub balance: f64,
}

impl FinancialSummary {
    pub fn new(transactions: &[Transaction]) -> Self {
        let mut total_income = 0.0;
        let mut total_expenses = 0.0;

        for transaction in transactions {
            match transaction.transaction_type {
                TransactionType::Income => total_income += transaction.amount,
                TransactionType::Expense => total_expenses += transaction.amount,
            }
        }

        FinancialSummary {
            total_income,
            total_expenses,
            balance: total_income - total_expenses,
        }
    }
}
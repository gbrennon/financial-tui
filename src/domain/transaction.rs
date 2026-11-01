use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub amount: f64,
    pub service: String,
    pub category: String,
    pub date: DateTime<Utc>,
    pub transaction_type: TransactionType,
}

impl Transaction {
    pub fn new(
        id: String,
        amount: f64,
        service: String,
        category: String,
        date: DateTime<Utc>,
        transaction_type: TransactionType,
    ) -> Result<Self, &'static str> {
        if amount <= 0.0 {
            return Err("Amount must be greater than zero");
        }
        
        if service.is_empty() {
            return Err("Service cannot be empty");
        }

        if category.is_empty() {
            return Err("Category cannot be empty");
        }

        Ok(Transaction {
            id,
            amount,
            service,
            category,
            date,
            transaction_type,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
pub enum TransactionType {
    Income,
    Expense,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Income => write!(f, "Income"),
            TransactionType::Expense => write!(f, "Expense"),
        }
    }
}
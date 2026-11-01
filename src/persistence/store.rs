use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::domain::transaction::Transaction;

pub trait Store {
    fn save_transactions(&self, transactions: Vec<Transaction>) -> Result<(), String>;
    fn load_transactions(&self) -> Result<Vec<Transaction>, String>;
}

#[derive(Serialize, Deserialize)]
struct TransactionFile {
    transactions: Vec<Transaction>,
}

pub struct JsonStore {
    file_path: String,
}

impl JsonStore {
    pub fn new(file_path: &str) -> Self {
        JsonStore {
            file_path: file_path.to_string(),
        }
    }
}

impl Store for JsonStore {
    fn save_transactions(&self, transactions: Vec<Transaction>) -> Result<(), String> {
        let transaction_file = TransactionFile { transactions };
        
        let json = serde_json::to_string_pretty(&transaction_file)
            .map_err(|e| format!("Failed to serialize transactions: {}", e))?;
            
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write to file {}: {}", self.file_path, e))?;
            
        Ok(())
    }

    fn load_transactions(&self) -> Result<Vec<Transaction>, String> {
        if !Path::new(&self.file_path).exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read from file {}: {}", self.file_path, e))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let transaction_file: TransactionFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from file {}: {}", self.file_path, e))?;

        Ok(transaction_file.transactions)
    }
}
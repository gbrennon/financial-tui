use financial_domain::transaction::{Transaction, TransactionType};
use financial_infrastructure::{JsonStore, Store};
use tempfile::NamedTempFile;

#[cfg(test)]
mod store_tests {
    use super::*;

    #[test]
    fn test_json_store_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let store = JsonStore::new(file_path);

        let transactions = vec![
            Transaction::new(
                "1".to_string(),
                100.0,
                "Salary".to_string(),
                "Income".to_string(),
                chrono::Utc::now(),
                TransactionType::Income,
            )
            .unwrap(),
            Transaction::new(
                "2".to_string(),
                50.0,
                "Groceries".to_string(),
                "Food".to_string(),
                chrono::Utc::now(),
                TransactionType::Expense,
            )
            .unwrap(),
        ];

        store.save_transactions(transactions.clone()).unwrap();

        let loaded_transactions = store.load_transactions().unwrap();

        assert_eq!(loaded_transactions.len(), 2);
        assert_eq!(loaded_transactions[0].id, "1");
        assert_eq!(loaded_transactions[1].id, "2");
    }

    #[test]
    fn test_json_store_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let store = JsonStore::new(file_path);

        let transactions = store.load_transactions().unwrap();
        assert_eq!(transactions.len(), 0);
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    use crate::domain::transaction::{Transaction, TransactionType};

    #[test]
    fn test_transaction_creation_success() {
        let transaction = Transaction::new(
            "1".to_string(),
            100.0,
            "Salary".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        ).unwrap();

        assert_eq!(transaction.id, "1");
        assert_eq!(transaction.amount, 100.0);
        assert_eq!(transaction.service, "Salary");
        assert_eq!(transaction.category, "Income");
        assert_eq!(transaction.transaction_type, TransactionType::Income);
    }

    #[test]
    fn test_transaction_creation_fails_with_zero_amount() {
        let result = Transaction::new(
            "1".to_string(),
            0.0,
            "Salary".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be greater than zero");
    }

    #[test]
    fn test_transaction_creation_fails_with_negative_amount() {
        let result = Transaction::new(
            "1".to_string(),
            -50.0,
            "Salary".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be greater than zero");
    }

    #[test]
    fn test_transaction_creation_fails_with_empty_service() {
        let result = Transaction::new(
            "1".to_string(),
            100.0,
            "".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Service cannot be empty");
    }

    #[test]
    fn test_transaction_creation_fails_with_empty_category() {
        let result = Transaction::new(
            "1".to_string(),
            100.0,
            "Salary".to_string(),
            "".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Category cannot be empty");
    }
}
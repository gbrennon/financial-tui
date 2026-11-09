#[cfg(test)]
mod transaction_tests {
    use financial_domain::transaction::{Transaction, TransactionType};

    #[test]
    fn test_transaction_creation_success() {
        let transaction = Transaction::new(
            "1".to_string(),
            100.0,
            "Salary".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        )
        .unwrap();

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
        assert_eq!(result.unwrap_err(), "Amount must be a positive number");
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
        assert_eq!(result.unwrap_err(), "Amount must be a positive number");
    }

    #[test]
    fn test_transaction_creation_fails_with_nan_amount() {
        let result = Transaction::new(
            "1".to_string(),
            f64::NAN,
            "Salary".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be a positive number");
    }

    #[test]
    fn test_transaction_creation_fails_with_infinite_amount() {
        let result = Transaction::new(
            "1".to_string(),
            f64::INFINITY,
            "Salary".to_string(),
            "Income".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be a positive number");
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
    fn test_transaction_creation_fails_with_whitespace_service() {
        let result = Transaction::new(
            "1".to_string(),
            100.0,
            "   ".to_string(),
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

    #[test]
    fn test_transaction_creation_fails_with_whitespace_category() {
        let result = Transaction::new(
            "1".to_string(),
            100.0,
            "Salary".to_string(),
            "  \t ".to_string(),
            chrono::Utc::now(),
            TransactionType::Income,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Category cannot be empty");
    }
}

#[cfg(test)]
mod financial_summary_tests {
    use financial_domain::{
        transaction::{Transaction, TransactionType},
        FinancialSummary,
    };

    #[test]
    fn test_financial_summary_calculation() {
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
            Transaction::new(
                "3".to_string(),
                25.0,
                "Entertainment".to_string(),
                "Leisure".to_string(),
                chrono::Utc::now(),
                TransactionType::Expense,
            )
            .unwrap(),
            Transaction::new(
                "4".to_string(),
                200.0,
                "Freelance".to_string(),
                "Income".to_string(),
                chrono::Utc::now(),
                TransactionType::Income,
            )
            .unwrap(),
        ];

        let summary = FinancialSummary::new(&transactions);

        assert_eq!(summary.total_income, 300.0);
        assert_eq!(summary.total_expenses, 75.0);
        assert_eq!(summary.balance, 225.0);
    }
}

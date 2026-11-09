use financial_domain::transaction::{Transaction, TransactionType};
use financial_infrastructure::Store;
use chrono::TimeZone;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use super::dashboard;

#[derive(Parser)]
#[clap(name = "financial-tui", about = "A financial management application")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Add an income transaction
    Income {
        /// Amount of income
        amount: f64,
        /// Service providing the income
        service: String,
        /// Category for the income
        category: String,
        /// Date (YYYY-MM-DD) - defaults to today
        #[clap(long, default_value_t = chrono::Utc::now().date_naive().to_string())]
        date: String,
    },
    /// Add an expense transaction
    Expense {
        /// Amount of expense
        amount: f64,
        /// Service providing the expense
        service: String,
        /// Category for the expense
        category: String,
        /// Date (YYYY-MM-DD) - defaults to today
        #[clap(long, default_value_t = chrono::Utc::now().date_naive().to_string())]
        date: String,
    },
    /// Generate a report with optional date range or preset filter
    Report {
        /// Preset time period for filtering transactions
        #[clap(short, long, value_enum)]
        preset: Option<PresetPeriod>,
        /// Start date (YYYY-MM-DD)
        #[clap(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[clap(long)]
        to: Option<String>,
    },
    /// Show current balance
    Balance {},
    /// Run interactive CLI mode
    Interactive {},
    /// Launch the Ratatui TUI dashboard
    Tui {},
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetPeriod {
    Last10Days,
    LastMonth,
    Last3Months,
    Last6Months,
    LastYear,
    AllTime,
}

impl PresetPeriod {
    pub fn date_range(
        &self,
        now: chrono::NaiveDate,
    ) -> (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) {
        match self {
            PresetPeriod::Last10Days => (
                now.checked_sub_signed(chrono::Duration::days(10)),
                Some(now),
            ),
            PresetPeriod::LastMonth => (
                now.checked_sub_signed(chrono::Duration::days(30)),
                Some(now),
            ),
            PresetPeriod::Last3Months => (
                now.checked_sub_signed(chrono::Duration::days(90)),
                Some(now),
            ),
            PresetPeriod::Last6Months => (
                now.checked_sub_signed(chrono::Duration::days(180)),
                Some(now),
            ),
            PresetPeriod::LastYear => (
                now.checked_sub_signed(chrono::Duration::days(365)),
                Some(now),
            ),
            PresetPeriod::AllTime => (None, None),
        }
    }
}

pub fn run(cli: Cli, store: impl Store) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Some(Commands::Income {
            amount,
            service,
            category,
            date,
        }) => {
            add_transaction(
                &store,
                TransactionType::Income,
                amount,
                service,
                category,
                date,
            )?;
        }
        Some(Commands::Expense {
            amount,
            service,
            category,
            date,
        }) => {
            add_transaction(
                &store,
                TransactionType::Expense,
                amount,
                service,
                category,
                date,
            )?;
        }
        Some(Commands::Report { preset, from, to }) => {
            show_report(&store, preset, from, to)?;
        }
        Some(Commands::Balance {}) => {
            show_balance(&store)?;
        }
        Some(Commands::Interactive {}) => {
            run_interactive(&store)?;
        }
        Some(Commands::Tui {}) | None => {
            launch_tui(&store)?;
        }
    }

    Ok(())
}

pub fn launch_tui(store: &impl Store) -> Result<(), Box<dyn std::error::Error>> {
    dashboard::run_dashboard(store)?;
    Ok(())
}

pub fn run_interactive(store: &impl Store) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Financial Manager Interactive Mode ===");
    let stdin = io::stdin();
    let mut reader = stdin.lines();

    loop {
        println!("\nSelect an action:");
        println!("  1. Add Income");
        println!("  2. Add Expense");
        println!("  3. View Current Balance");
        println!("  4. View Financial Report");
        println!("  5. Launch TUI Dashboard");
        println!("  6. Exit");
        print!("> ");
        io::stdout().flush()?;

        let choice = match reader.next() {
            Some(Ok(line)) => line.trim().to_string(),
            _ => break,
        };

        match choice.as_str() {
            "1" => {
                print!("Amount: ");
                io::stdout().flush()?;
                let amount_str = reader.next().unwrap_or(Ok(String::new()))?;
                let amount: f64 = amount_str.trim().parse().unwrap_or(0.0);

                print!("Service: ");
                io::stdout().flush()?;
                let service = reader
                    .next()
                    .unwrap_or(Ok(String::new()))?
                    .trim()
                    .to_string();

                print!("Category: ");
                io::stdout().flush()?;
                let category = reader
                    .next()
                    .unwrap_or(Ok(String::new()))?
                    .trim()
                    .to_string();

                let today = chrono::Utc::now().date_naive().to_string();
                if let Err(e) = add_transaction(
                    store,
                    TransactionType::Income,
                    amount,
                    service,
                    category,
                    today,
                ) {
                    println!("Error: {}", e);
                }
            }
            "2" => {
                print!("Amount: ");
                io::stdout().flush()?;
                let amount_str = reader.next().unwrap_or(Ok(String::new()))?;
                let amount: f64 = amount_str.trim().parse().unwrap_or(0.0);

                print!("Service: ");
                io::stdout().flush()?;
                let service = reader
                    .next()
                    .unwrap_or(Ok(String::new()))?
                    .trim()
                    .to_string();

                print!("Category: ");
                io::stdout().flush()?;
                let category = reader
                    .next()
                    .unwrap_or(Ok(String::new()))?
                    .trim()
                    .to_string();

                let today = chrono::Utc::now().date_naive().to_string();
                if let Err(e) = add_transaction(
                    store,
                    TransactionType::Expense,
                    amount,
                    service,
                    category,
                    today,
                ) {
                    println!("Error: {}", e);
                }
            }
            "3" => {
                show_balance(store)?;
            }
            "4" => {
                println!("Select Report Filter: (1: 10 Days, 2: 1 Month, 3: 3 Months, 4: 6 Months, 5: 1 Year, 6: All)");
                print!("> ");
                io::stdout().flush()?;
                let filter_choice = reader
                    .next()
                    .unwrap_or(Ok(String::new()))?
                    .trim()
                    .to_string();
                let preset = match filter_choice.as_str() {
                    "1" => Some(PresetPeriod::Last10Days),
                    "2" => Some(PresetPeriod::LastMonth),
                    "3" => Some(PresetPeriod::Last3Months),
                    "4" => Some(PresetPeriod::Last6Months),
                    "5" => Some(PresetPeriod::LastYear),
                    _ => Some(PresetPeriod::AllTime),
                };
                show_report(store, preset, None, None)?;
            }
            "5" => {
                launch_tui(store)?;
            }
            "6" | "q" | "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Invalid option. Please enter 1-6.");
            }
        }
    }

    Ok(())
}

fn add_transaction(
    store: &impl Store,
    transaction_type: TransactionType,
    amount: f64,
    service: String,
    category: String,
    date_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format. Please use YYYY-MM-DD: {}", e))?;

    let datetime = chrono::Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap());
    let id = uuid::Uuid::new_v4().to_string();

    let transaction = Transaction::new(id, amount, service, category, datetime, transaction_type)?;

    let mut transactions = store.load_transactions()?;
    transactions.push(transaction);
    store.save_transactions(transactions)?;

    println!("Transaction added successfully!");
    Ok(())
}

fn show_report(
    store: &impl Store,
    preset: Option<PresetPeriod>,
    from: Option<String>,
    to: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let transactions = store.load_transactions()?;

    let filtered_transactions = if let Some(preset_period) = preset {
        filter_by_preset(&transactions, preset_period)
    } else if from.is_some() || to.is_some() {
        match (&from, &to) {
            (Some(from_date), Some(to_date)) => {
                filter_by_date_range(&transactions, from_date, to_date)?
            }
            (Some(_), None) => {
                return Err("Both --from and --to must be provided for a custom date range".into())
            }
            (None, Some(_)) => {
                return Err("Both --from and --to must be provided for a custom date range".into())
            }
            (None, None) => unreachable!(),
        }
    } else {
        transactions
    };

    if filtered_transactions.is_empty() {
        println!("No transactions found for the specified period.");
        return Ok(());
    }

    let summary = financial_domain::FinancialSummary::new(&filtered_transactions);

    println!("\n=== Financial Report ===");
    println!("Total Income: ${:.2}", summary.total_income);
    println!("Total Expenses: ${:.2}", summary.total_expenses);
    println!("Balance: ${:.2}\n", summary.balance);

    println!("Transactions:");
    for transaction in filtered_transactions.iter() {
        println!(
            "- {}: {:.2} ({}) - {}",
            transaction.date.format("%Y-%m-%d"),
            transaction.amount,
            transaction.transaction_type,
            transaction.service
        );
    }

    Ok(())
}

fn show_balance(store: &impl Store) -> Result<(), Box<dyn std::error::Error>> {
    let transactions = store.load_transactions()?;
    let summary = financial_domain::FinancialSummary::new(&transactions);
    println!("Current Balance: ${:.2}", summary.balance);
    Ok(())
}

pub fn filter_by_preset(transactions: &[Transaction], preset: PresetPeriod) -> Vec<Transaction> {
    let now = chrono::Utc::now().date_naive();
    filter_by_preset_at(transactions, preset, now)
}

pub fn filter_by_preset_at(
    transactions: &[Transaction],
    preset: PresetPeriod,
    now: chrono::NaiveDate,
) -> Vec<Transaction> {
    let (start_opt, end_opt) = preset.date_range(now);

    transactions
        .iter()
        .filter(|t| {
            let d = t.date.date_naive();
            if let Some(start) = start_opt {
                if d < start {
                    return false;
                }
            }
            if let Some(end) = end_opt {
                if d > end {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

pub fn filter_by_date_range(
    transactions: &[Transaction],
    from: &str,
    to: &str,
) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let from_date = chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map_err(|e| format!("Invalid 'from' date format. Please use YYYY-MM-DD: {}", e))?;

    let to_date = chrono::NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map_err(|e| format!("Invalid 'to' date format. Please use YYYY-MM-DD: {}", e))?;

    if from_date > to_date {
        return Err("Start date ('from') cannot be after end date ('to')".into());
    }

    Ok(transactions
        .iter()
        .filter(|t| {
            let transaction_date = t.date.date_naive();
            transaction_date >= from_date && transaction_date <= to_date
        })
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(id: &str, days_from_now: i64) -> Transaction {
        let date = chrono::Utc::now()
            .date_naive()
            .checked_add_signed(chrono::Duration::days(days_from_now))
            .unwrap();
        let dt = chrono::Utc.from_utc_datetime(&date.and_hms_opt(12, 0, 0).unwrap());
        Transaction::new(
            id.to_string(),
            100.0,
            "Svc".to_string(),
            "Cat".to_string(),
            dt,
            TransactionType::Income,
        )
        .unwrap()
    }

    #[test]
    fn test_filter_by_preset_bounds() {
        let now = chrono::Utc::now().date_naive();
        let past_5 = make_tx("t1", -5);
        let past_15 = make_tx("t2", -15);
        let past_45 = make_tx("t3", -45);
        let future_2 = make_tx("t4", 2);

        let all = vec![past_5, past_15, past_45, future_2];

        // Last 10 days should include past_5, reject past_15, and reject future_2
        let filtered_10d = filter_by_preset_at(&all, PresetPeriod::Last10Days, now);
        assert_eq!(filtered_10d.len(), 1);
        assert_eq!(filtered_10d[0].id, "t1");

        // Last Month (30 days) should include past_5 and past_15, reject past_45 and future_2
        let filtered_1m = filter_by_preset_at(&all, PresetPeriod::LastMonth, now);
        assert_eq!(filtered_1m.len(), 2);

        // AllTime includes everything including future
        let filtered_all = filter_by_preset_at(&all, PresetPeriod::AllTime, now);
        assert_eq!(filtered_all.len(), 4);
    }

    #[test]
    fn test_filter_by_date_range_valid() {
        let now = chrono::Utc::now().date_naive();
        let past_5 = make_tx("t1", -5);
        let past_15 = make_tx("t2", -15);
        let all = vec![past_5, past_15];

        let from = now
            .checked_sub_signed(chrono::Duration::days(10))
            .unwrap()
            .to_string();
        let to = now.to_string();
        let res = filter_by_date_range(&all, &from, &to).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, "t1");
    }

    #[test]
    fn test_filter_by_date_range_reversed_fails() {
        let all = vec![make_tx("t1", 0)];
        let res = filter_by_date_range(&all, "2026-11-10", "2026-11-01");
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("cannot be after end date"));
    }
}

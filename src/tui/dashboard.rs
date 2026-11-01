use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;

use crate::cli::cli::{filter_by_date_range, filter_by_preset_at, PresetPeriod};
use crate::domain::transaction::{Transaction, TransactionType};
use crate::domain::FinancialSummary;
use crate::persistence::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputField {
    Amount,
    Service,
    Category,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    AddIncome {
        field: InputField,
        amount: String,
        service: String,
        category: String,
        error: Option<String>,
    },
    AddExpense {
        field: InputField,
        amount: String,
        service: String,
        category: String,
        error: Option<String>,
    },
    CustomRange {
        from: String,
        to: String,
        is_to: bool,
        error: Option<String>,
    },
}

pub struct DashboardApp<'a, S: Store> {
    pub store: &'a S,
    pub all_transactions: Vec<Transaction>,
    pub filtered_transactions: Vec<Transaction>,
    pub summary: FinancialSummary,
    pub active_preset: Option<PresetPeriod>,
    pub active_custom_range: Option<(String, String)>,
    pub mode: AppMode,
}

impl<'a, S: Store> DashboardApp<'a, S> {
    pub fn new(store: &'a S) -> Result<Self, Box<dyn std::error::Error>> {
        let transactions = store.load_transactions()?;
        let summary = FinancialSummary::new(&transactions);
        Ok(DashboardApp {
            store,
            all_transactions: transactions.clone(),
            filtered_transactions: transactions,
            summary,
            active_preset: None,
            active_custom_range: None,
            mode: AppMode::Normal,
        })
    }

    pub fn refresh_filter(&mut self) {
        if let Some((from, to)) = &self.active_custom_range {
            if let Ok(filtered) = filter_by_date_range(&self.all_transactions, from, to) {
                self.summary = FinancialSummary::new(&filtered);
                self.filtered_transactions = filtered;
                return;
            }
        }

        if let Some(preset) = self.active_preset {
            let now = chrono::Utc::now().date_naive();
            let filtered = filter_by_preset_at(&self.all_transactions, preset, now);
            self.summary = FinancialSummary::new(&filtered);
            self.filtered_transactions = filtered;
        } else {
            self.filtered_transactions = self.all_transactions.clone();
            self.summary = FinancialSummary::new(&self.filtered_transactions);
        }
    }

    pub fn cycle_preset(&mut self) {
        self.active_custom_range = None;
        self.active_preset = match self.active_preset {
            None => Some(PresetPeriod::Last10Days),
            Some(PresetPeriod::Last10Days) => Some(PresetPeriod::LastMonth),
            Some(PresetPeriod::LastMonth) => Some(PresetPeriod::Last3Months),
            Some(PresetPeriod::Last3Months) => Some(PresetPeriod::Last6Months),
            Some(PresetPeriod::Last6Months) => Some(PresetPeriod::LastYear),
            Some(PresetPeriod::LastYear) => Some(PresetPeriod::AllTime),
            Some(PresetPeriod::AllTime) => None,
        };
        self.refresh_filter();
    }

    pub fn add_transaction(
        &mut self,
        tx_type: TransactionType,
        amount_str: &str,
        service: &str,
        category: &str,
    ) -> Result<(), String> {
        let amount: f64 = amount_str
            .trim()
            .parse()
            .map_err(|_| "Amount must be a valid number".to_string())?;

        let id = uuid::Uuid::now_v7().to_string();
        let now = chrono::Utc::now();

        let tx = Transaction::new(
            id,
            amount,
            service.to_string(),
            category.to_string(),
            now,
            tx_type,
        )?;

        let mut candidate = self.all_transactions.clone();
        candidate.push(tx);
        self.store
            .save_transactions(candidate.clone())
            .map_err(|e| e.to_string())?;

        self.all_transactions = candidate;
        self.refresh_filter();
        Ok(())
    }
}

pub fn run_dashboard(store: &impl Store) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DashboardApp::new(store)?;

    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_loop<S: Store>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut DashboardApp<S>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let current_mode = std::mem::replace(&mut app.mode, AppMode::Normal);
                match current_mode {
                    AppMode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => app.cycle_preset(),
                        KeyCode::Char('r') => {
                            app.active_preset = None;
                            app.active_custom_range = None;
                            app.refresh_filter();
                        }
                        KeyCode::Char('i') => {
                            app.mode = AppMode::AddIncome {
                                field: InputField::Amount,
                                amount: String::new(),
                                service: String::new(),
                                category: String::new(),
                                error: None,
                            };
                        }
                        KeyCode::Char('e') => {
                            app.mode = AppMode::AddExpense {
                                field: InputField::Amount,
                                amount: String::new(),
                                service: String::new(),
                                category: String::new(),
                                error: None,
                            };
                        }
                        KeyCode::Char('c') => {
                            let today = chrono::Utc::now().date_naive().to_string();
                            app.mode = AppMode::CustomRange {
                                from: today.clone(),
                                to: today,
                                is_to: false,
                                error: None,
                            };
                        }
                        _ => {}
                    },
                    AppMode::AddIncome {
                        mut field,
                        mut amount,
                        mut service,
                        mut category,
                        mut error,
                    } => match key.code {
                        KeyCode::Esc => app.mode = AppMode::Normal,
                        KeyCode::Tab | KeyCode::Down => {
                            field = match field {
                                InputField::Amount => InputField::Service,
                                InputField::Service => InputField::Category,
                                InputField::Category => InputField::Amount,
                            };
                            app.mode = AppMode::AddIncome {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            field = match field {
                                InputField::Amount => InputField::Category,
                                InputField::Service => InputField::Amount,
                                InputField::Category => InputField::Service,
                            };
                            app.mode = AppMode::AddIncome {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::Char(c) => {
                            match field {
                                InputField::Amount => amount.push(c),
                                InputField::Service => service.push(c),
                                InputField::Category => category.push(c),
                            };
                            app.mode = AppMode::AddIncome {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::Backspace => {
                            match field {
                                InputField::Amount => {
                                    amount.pop();
                                }
                                InputField::Service => {
                                    service.pop();
                                }
                                InputField::Category => {
                                    category.pop();
                                }
                            };
                            app.mode = AppMode::AddIncome {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::Enter => {
                            match app.add_transaction(
                                TransactionType::Income,
                                &amount,
                                &service,
                                &category,
                            ) {
                                Ok(()) => app.mode = AppMode::Normal,
                                Err(msg) => {
                                    error = Some(msg);
                                    app.mode = AppMode::AddIncome {
                                        field,
                                        amount,
                                        service,
                                        category,
                                        error,
                                    };
                                }
                            }
                        }
                        _ => {
                            app.mode = AppMode::AddIncome {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                    },
                    AppMode::AddExpense {
                        mut field,
                        mut amount,
                        mut service,
                        mut category,
                        mut error,
                    } => match key.code {
                        KeyCode::Esc => app.mode = AppMode::Normal,
                        KeyCode::Tab | KeyCode::Down => {
                            field = match field {
                                InputField::Amount => InputField::Service,
                                InputField::Service => InputField::Category,
                                InputField::Category => InputField::Amount,
                            };
                            app.mode = AppMode::AddExpense {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            field = match field {
                                InputField::Amount => InputField::Category,
                                InputField::Service => InputField::Amount,
                                InputField::Category => InputField::Service,
                            };
                            app.mode = AppMode::AddExpense {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::Char(c) => {
                            match field {
                                InputField::Amount => amount.push(c),
                                InputField::Service => service.push(c),
                                InputField::Category => category.push(c),
                            };
                            app.mode = AppMode::AddExpense {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::Backspace => {
                            match field {
                                InputField::Amount => {
                                    amount.pop();
                                }
                                InputField::Service => {
                                    service.pop();
                                }
                                InputField::Category => {
                                    category.pop();
                                }
                            };
                            app.mode = AppMode::AddExpense {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                        KeyCode::Enter => {
                            match app.add_transaction(
                                TransactionType::Expense,
                                &amount,
                                &service,
                                &category,
                            ) {
                                Ok(()) => app.mode = AppMode::Normal,
                                Err(msg) => {
                                    error = Some(msg);
                                    app.mode = AppMode::AddExpense {
                                        field,
                                        amount,
                                        service,
                                        category,
                                        error,
                                    };
                                }
                            }
                        }
                        _ => {
                            app.mode = AppMode::AddExpense {
                                field,
                                amount,
                                service,
                                category,
                                error,
                            };
                        }
                    },
                    AppMode::CustomRange {
                        mut from,
                        mut to,
                        mut is_to,
                        mut error,
                    } => match key.code {
                        KeyCode::Esc => app.mode = AppMode::Normal,
                        KeyCode::Tab | KeyCode::Up | KeyCode::Down => {
                            is_to = !is_to;
                            app.mode = AppMode::CustomRange {
                                from,
                                to,
                                is_to,
                                error,
                            };
                        }
                        KeyCode::Char(c) => {
                            if is_to {
                                to.push(c);
                            } else {
                                from.push(c);
                            }
                            app.mode = AppMode::CustomRange {
                                from,
                                to,
                                is_to,
                                error,
                            };
                        }
                        KeyCode::Backspace => {
                            if is_to {
                                to.pop();
                            } else {
                                from.pop();
                            }
                            app.mode = AppMode::CustomRange {
                                from,
                                to,
                                is_to,
                                error,
                            };
                        }
                        KeyCode::Enter => {
                            match filter_by_date_range(&app.all_transactions, &from, &to) {
                                Ok(_) => {
                                    app.active_preset = None;
                                    app.active_custom_range = Some((from, to));
                                    app.refresh_filter();
                                    app.mode = AppMode::Normal;
                                }
                                Err(e) => {
                                    error = Some(e.to_string());
                                    app.mode = AppMode::CustomRange {
                                        from,
                                        to,
                                        is_to,
                                        error,
                                    };
                                }
                            }
                        }
                        _ => {
                            app.mode = AppMode::CustomRange {
                                from,
                                to,
                                is_to,
                                error,
                            };
                        }
                    },
                }
            }
        }
    }

    Ok(())
}

fn draw_ui<S: Store>(frame: &mut Frame, app: &DashboardApp<S>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.size());

    // Header
    let filter_label = if let Some((from, to)) = &app.active_custom_range {
        format!("Custom Range: {} to {}", from, to)
    } else if let Some(preset) = app.active_preset {
        match preset {
            PresetPeriod::Last10Days => "Last 10 Days".to_string(),
            PresetPeriod::LastMonth => "Last Month".to_string(),
            PresetPeriod::Last3Months => "Last 3 Months".to_string(),
            PresetPeriod::Last6Months => "Last 6 Months".to_string(),
            PresetPeriod::LastYear => "Last Year".to_string(),
            PresetPeriod::AllTime => "All Time".to_string(),
        }
    } else {
        "All Transactions".to_string()
    };

    let header = Block::default()
        .title(format!(
            "Financial Manager TUI | Filter: [{}]",
            filter_label
        ))
        .borders(Borders::ALL)
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    // Metrics
    let summary_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    let inc_card = Paragraph::new(format!("\n${:.2}", app.summary.total_income))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().title("Total Income").borders(Borders::ALL));
    frame.render_widget(inc_card, summary_chunks[0]);

    let exp_card = Paragraph::new(format!("\n${:.2}", app.summary.total_expenses))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red))
        .block(
            Block::default()
                .title("Total Expenses")
                .borders(Borders::ALL),
        );
    frame.render_widget(exp_card, summary_chunks[1]);

    let bal_color = if app.summary.balance >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };
    let bal_card = Paragraph::new(format!("\n${:.2}", app.summary.balance))
        .alignment(Alignment::Center)
        .style(Style::default().fg(bal_color))
        .block(Block::default().title("Net Balance").borders(Borders::ALL));
    frame.render_widget(bal_card, summary_chunks[2]);

    // Transactions Table
    let rows: Vec<Row> = app
        .filtered_transactions
        .iter()
        .map(|t| {
            let color = match t.transaction_type {
                TransactionType::Income => Color::Green,
                TransactionType::Expense => Color::Red,
            };
            Row::new(vec![
                t.date.format("%Y-%m-%d %H:%M").to_string(),
                format!("${:.2}", t.amount),
                t.service.clone(),
                t.category.clone(),
                t.transaction_type.to_string(),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec!["Date", "Amount", "Service", "Category", "Type"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .widths(&[
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Length(22),
            Constraint::Length(18),
            Constraint::Length(10),
        ])
        .block(
            Block::default()
                .title(format!(
                    "Transactions ({})",
                    app.filtered_transactions.len()
                ))
                .borders(Borders::ALL),
        );
    frame.render_widget(table, chunks[2]);

    // Footer
    let footer_text = "[q: Quit] [i: +Income] [e: +Expense] [p: Cycle Preset Period] [c: Custom Range] [r: Reset Filter]";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[3]);

    // Popups
    match &app.mode {
        AppMode::AddIncome {
            field,
            amount,
            service,
            category,
            error,
        } => {
            render_add_modal(
                frame,
                "Register Income",
                field,
                amount,
                service,
                category,
                error,
            );
        }
        AppMode::AddExpense {
            field,
            amount,
            service,
            category,
            error,
        } => {
            render_add_modal(
                frame,
                "Register Expense",
                field,
                amount,
                service,
                category,
                error,
            );
        }
        AppMode::CustomRange {
            from,
            to,
            is_to,
            error,
        } => {
            render_custom_range_modal(frame, from, to, *is_to, error);
        }
        AppMode::Normal => {}
    }
}

fn render_add_modal(
    frame: &mut Frame,
    title: &str,
    field: &InputField,
    amount: &str,
    service: &str,
    category: &str,
    error: &Option<String>,
) {
    let area = centered_rect(60, 45, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            " {} (Tab: next field | Enter: save | Esc: cancel) ",
            title
        ))
        .borders(Borders::ALL);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);

    let amt_style = if *field == InputField::Amount {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let svc_style = if *field == InputField::Service {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let cat_style = if *field == InputField::Category {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let p_amt = Paragraph::new(amount.to_string()).block(
        Block::default()
            .title("Amount: ")
            .borders(Borders::ALL)
            .style(amt_style),
    );
    frame.render_widget(p_amt, layout[0]);

    let p_svc = Paragraph::new(service.to_string()).block(
        Block::default()
            .title("Service: ")
            .borders(Borders::ALL)
            .style(svc_style),
    );
    frame.render_widget(p_svc, layout[1]);

    let p_cat = Paragraph::new(category.to_string()).block(
        Block::default()
            .title("Category: ")
            .borders(Borders::ALL)
            .style(cat_style),
    );
    frame.render_widget(p_cat, layout[2]);

    if let Some(err) = error {
        let p_err = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(p_err, layout[3]);
    }
}

fn render_custom_range_modal(
    frame: &mut Frame,
    from: &str,
    to: &str,
    is_to: bool,
    error: &Option<String>,
) {
    let area = centered_rect(60, 35, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Custom Date Range (Tab: switch | Enter: apply | Esc: cancel) ")
        .borders(Borders::ALL);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);

    let from_style = if !is_to {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let to_style = if is_to {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let p_from = Paragraph::new(from.to_string()).block(
        Block::default()
            .title("From (YYYY-MM-DD): ")
            .borders(Borders::ALL)
            .style(from_style),
    );
    frame.render_widget(p_from, layout[0]);

    let p_to = Paragraph::new(to.to_string()).block(
        Block::default()
            .title("To (YYYY-MM-DD): ")
            .borders(Borders::ALL)
            .style(to_style),
    );
    frame.render_widget(p_to, layout[1]);

    if let Some(err) = error {
        let p_err = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(p_err, layout[2]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::JsonStore;
    use tempfile::NamedTempFile;

    #[test]
    fn test_tui_app_add_income_and_expense() {
        let temp = NamedTempFile::new().unwrap();
        let store = JsonStore::new(temp.path().to_str().unwrap());
        let mut app = DashboardApp::new(&store).unwrap();

        assert_eq!(app.all_transactions.len(), 0);

        // Add income via app method
        app.add_transaction(TransactionType::Income, "1500.0", "Consulting", "Work")
            .unwrap();
        assert_eq!(app.all_transactions.len(), 1);
        assert_eq!(app.summary.total_income, 1500.0);
        assert_eq!(app.summary.balance, 1500.0);

        // Add expense via app method
        app.add_transaction(TransactionType::Expense, "500.0", "Office", "Rent")
            .unwrap();
        assert_eq!(app.all_transactions.len(), 2);
        assert_eq!(app.summary.total_expenses, 500.0);
        assert_eq!(app.summary.balance, 1000.0);
    }

    #[test]
    fn test_tui_app_preset_cycling() {
        let temp = NamedTempFile::new().unwrap();
        let store = JsonStore::new(temp.path().to_str().unwrap());
        let mut app = DashboardApp::new(&store).unwrap();

        assert_eq!(app.active_preset, None);
        app.cycle_preset();
        assert_eq!(app.active_preset, Some(PresetPeriod::Last10Days));
        app.cycle_preset();
        assert_eq!(app.active_preset, Some(PresetPeriod::LastMonth));
        app.cycle_preset();
        assert_eq!(app.active_preset, Some(PresetPeriod::Last3Months));
        app.cycle_preset();
        assert_eq!(app.active_preset, Some(PresetPeriod::Last6Months));
        app.cycle_preset();
        assert_eq!(app.active_preset, Some(PresetPeriod::LastYear));
        app.cycle_preset();
        assert_eq!(app.active_preset, Some(PresetPeriod::AllTime));
        app.cycle_preset();
        assert_eq!(app.active_preset, None);
    }

    struct FailingStore;
    impl Store for FailingStore {
        fn save_transactions(&self, _: Vec<Transaction>) -> Result<(), String> {
            Err("Disk write failure".to_string())
        }
        fn load_transactions(&self) -> Result<Vec<Transaction>, String> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn test_tui_app_add_transaction_failure_does_not_mutate_state() {
        let store = FailingStore;
        let mut app = DashboardApp::new(&store).unwrap();
        let res = app.add_transaction(TransactionType::Income, "100.0", "Service", "Category");
        assert!(res.is_err());
        assert_eq!(app.all_transactions.len(), 0);
        assert_eq!(app.filtered_transactions.len(), 0);
    }
}

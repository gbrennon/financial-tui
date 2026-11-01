use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Table, Row, Cell},
    layout::{Layout, Constraint, Direction, Alignment},
    style::{Style, Color},
    text::{Span, Line},
    Frame,
};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode},
    execute,
    terminal::EnterAlternateScreen,
    terminal::LeaveAlternateScreen,
};
use std::io;

use crate::domain::transaction::Transaction;
use crate::domain::FinancialSummary;

#[derive(Debug)]
pub struct Dashboard<'a> {
    pub transactions: Vec<Transaction>,
    pub summary: FinancialSummary,
    pub preset_filter: Option<&'a str>,
}

impl<'a> Dashboard<'a> {
    pub fn new(transactions: Vec<Transaction>, summary: FinancialSummary) -> Self {
        Dashboard {
            transactions,
            summary,
            preset_filter: None,
        }
    }

    pub fn with_preset_filter(mut self, filter: &'a str) -> Self {
        self.preset_filter = Some(filter);
        self
    }
}

pub fn run_dashboard(dashboard: Dashboard) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw_ui(f, &dashboard))?;
        
        // Simple way to exit - just press 'q'
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                if key_event.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_ui(frame: &mut Frame, dashboard: &Dashboard) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(6),  // Summary metrics
            Constraint::Min(5),     // Transactions table
            Constraint::Length(3),  // Footer
        ])
        .split(frame.size());

    // Header
    let header = Block::default()
        .title("Financial TUI")
        .borders(Borders::ALL);
    frame.render_widget(header, chunks[0]);

    // Summary Metrics
    let summary_block = Block::default()
        .title("Financial Summary")
        .borders(Borders::ALL);

    let income_text = Span::styled(
        format!("Income: ${:.2}", dashboard.summary.total_income),
        Style::default().fg(Color::Green),
    );
    
    let expense_text = Span::styled(
        format!("Expenses: ${:.2}", dashboard.summary.total_expenses),
        Style::default().fg(Color::Red),
    );
    
    let balance_text = Span::styled(
        format!("Balance: ${:.2}", dashboard.summary.balance),
        Style::default().fg(if dashboard.summary.balance >= 0.0 { Color::Green } else { Color::Red }),
    );

    let preset_text = match dashboard.preset_filter {
        Some(filter) => Span::raw(format!("Filter: {}", filter)),
        None => Span::raw("Filter: All Time"),
    };

    let lines = vec![
        Line::from(income_text),
        Line::from(expense_text),
        Line::from(balance_text),
        Line::from(preset_text),
    ];

    let summary_block_cloned = summary_block.clone();
    frame.render_widget(summary_block_cloned, chunks[1]);

    // Render the text inside the block with proper alignment
    let inner_area = summary_block.inner(chunks[1]);
    frame.render_widget(
        ratatui::widgets::Paragraph::new(lines)
            .alignment(Alignment::Left),
        inner_area,
    );

    // Transactions Table
    let transactions_block = Block::default()
        .title("Transactions")
        .borders(Borders::ALL);
        
    let transactions_block_cloned = transactions_block.clone();
    frame.render_widget(transactions_block_cloned, chunks[2]);

    if !dashboard.transactions.is_empty() {
        let rows: Vec<Row> = dashboard.transactions.iter()
            .map(|transaction| {
                Row::new(vec![
                    Cell::from(transaction.date.format("%Y-%m-%d").to_string()),
                    Cell::from(format!("{:.2}", transaction.amount)),
                    Cell::from(transaction.service.clone()),
                    Cell::from(transaction.category.clone()),
                    Cell::from(transaction.transaction_type.to_string()),
                ])
            })
            .collect();

        let table = Table::new(rows)
            .header(Row::new(vec![
                Cell::from("Date"),
                Cell::from("Amount"),
                Cell::from("Service"),
                Cell::from("Category"),
                Cell::from("Type"),
            ]))
            .block(transactions_block)
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(15),
                Constraint::Length(10),
            ]);
            
        frame.render_widget(table, chunks[2]);
    }

    // Footer
    let footer = Block::default()
        .title("Press 'q' to quit")
        .borders(Borders::ALL);
    frame.render_widget(footer, chunks[3]);
}
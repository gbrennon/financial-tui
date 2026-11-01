pub mod domain;
pub mod persistence;
pub mod cli;
pub mod tui;

pub struct FinancialApp<'a> {
    store: Box<dyn persistence::Store>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> FinancialApp<'a> {
    pub fn new(store: Box<dyn persistence::Store>) -> Self {
        FinancialApp {
            store,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn get_summary(&self) -> Result<domain::FinancialSummary, String> {
        let transactions = self.store.load_transactions()?;
        Ok(domain::FinancialSummary::new(&transactions))
    }

    pub fn get_transactions(&self) -> Result<Vec<domain::transaction::Transaction>, String> {
        self.store.load_transactions()
    }
}
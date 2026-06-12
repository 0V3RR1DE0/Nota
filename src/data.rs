use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    Income,
    Expense,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::Income  => write!(f, "Income"),
            EntryType::Expense => write!(f, "Expense"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub number:      u32,
    pub date:        String,
    pub desc:        String,
    pub entry_type:  EntryType,
    pub product:     String,
    pub vat_percent: f64,
    pub price_excl:  f64,
    pub vat_amount:  f64,
    pub price_incl:  f64,
    // ← NEW: optional attached filename (e.g. "receipt.pdf")
    // #[serde(default)] means: if this column is missing from an old CSV, use None
    #[serde(default)]
    pub attachment: Option<String>,
}

impl Entry {
    pub fn new(
        number:      u32,
        date:        String,
        desc: String,
        entry_type:  EntryType,
        product:     String,
        vat_percent: f64,
        price_excl:  f64,
        attachment:  Option<String>,  // ← NEW parameter
    ) -> Self {
        let vat_amount = (price_excl * vat_percent / 100.0 * 100.0).round() / 100.0;
        let price_incl = ((price_excl + vat_amount) * 100.0).round() / 100.0;
        Entry {
            number, date, desc, entry_type, product,
            vat_percent, price_excl, vat_amount, price_incl,
            attachment,
        }
    }

    #[allow(dead_code)]  // used later for inline editing
    pub fn recalculate(&mut self) {
        self.vat_amount = (self.price_excl * self.vat_percent / 100.0 * 100.0).round() / 100.0;
        self.price_incl = ((self.price_excl + self.vat_amount) * 100.0).round() / 100.0;
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub entries: Vec<Entry>,
}

impl Ledger {
    pub fn next_entry_number(&self) -> u32 {
        self.entries.iter().map(|e| e.number).max().unwrap_or(0) + 1
    }

    pub fn total_income(&self) -> f64 {
        self.entries.iter()
            .filter(|e| e.entry_type == EntryType::Income)
            .map(|e| e.price_incl).sum()
    }

    pub fn total_expenses(&self) -> f64 {
        self.entries.iter()
            .filter(|e| e.entry_type == EntryType::Expense)
            .map(|e| e.price_incl).sum()
    }

    pub fn total_vat_collected(&self) -> f64 {
        self.entries.iter()
            .filter(|e| e.entry_type == EntryType::Income)
            .map(|e| e.vat_amount).sum()
    }

    pub fn total_vat_deductible(&self) -> f64 {
        self.entries.iter()
            .filter(|e| e.entry_type == EntryType::Expense)
            .map(|e| e.vat_amount).sum()
    }
}
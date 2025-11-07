





use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Clone)]
pub struct NaiveSnapshot {
    pub timestamp_us: u64,
    pub datetime: String,
    
    pub data: HashMap<String, f64>,
}

impl NaiveSnapshot {
    
    pub fn from_csv_data(
        timestamp_us: u64,
        datetime: String,
        values: Vec<f64>,
    ) -> Self {
        let mut data = HashMap::new();

        
        for i in 0..10 {
            let price_key = format!("bid_price_{}", i + 1);
            let qty_key = format!("bid_qty_{}", i + 1);
            data.insert(price_key, values[i * 2]);
            data.insert(qty_key, values[i * 2 + 1]);
        }

        
        for i in 0..10 {
            let price_key = format!("ask_price_{}", i + 1);
            let qty_key = format!("ask_qty_{}", i + 1);
            data.insert(price_key, values[20 + i * 2]);
            data.insert(qty_key, values[20 + i * 2 + 1]);
        }

        Self {
            timestamp_us,
            datetime,
            data,
        }
    }

    
    pub fn bids(&self) -> Vec<PriceLevel> {
        let mut levels = Vec::new();
        for i in 1..=10 {
            let price_key = format!("bid_price_{}", i);
            let qty_key = format!("bid_qty_{}", i);

            if let (Some(&price), Some(&qty)) =
                (self.data.get(&price_key), self.data.get(&qty_key)) {
                levels.push(PriceLevel { price, quantity: qty });
            }
        }
        levels
    }

    
    pub fn asks(&self) -> Vec<PriceLevel> {
        let mut levels = Vec::new();
        for i in 1..=10 {
            let price_key = format!("ask_price_{}", i);
            let qty_key = format!("ask_qty_{}", i);

            if let (Some(&price), Some(&qty)) =
                (self.data.get(&price_key), self.data.get(&qty_key)) {
                levels.push(PriceLevel { price, quantity: qty });
            }
        }
        levels
    }

    
    pub fn best_bid(&self) -> Option<f64> {
        self.data.get("bid_price_1").copied()
    }

    
    pub fn best_ask(&self) -> Option<f64> {
        self.data.get("ask_price_1").copied()
    }

    
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    
    pub fn total_bid_qty(&self) -> f64 {
        self.bids().iter().map(|l| l.quantity).sum()
    }

    
    pub fn total_ask_qty(&self) -> f64 {
        self.asks().iter().map(|l| l.quantity).sum()
    }

    
    pub fn imbalance(&self) -> f64 {
        let bid_qty = self.total_bid_qty();
        let ask_qty = self.total_ask_qty();
        (bid_qty - ask_qty) / (bid_qty + ask_qty)
    }
}

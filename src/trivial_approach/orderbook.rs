





use super::snapshot::NaiveSnapshot;

#[derive(Debug)]
pub struct NaiveOrderBook {
    
    current_snapshot: Option<Box<NaiveSnapshot>>,
    update_count: u64,
}

impl NaiveOrderBook {
    pub fn new() -> Self {
        Self {
            current_snapshot: None,
            update_count: 0,
        }
    }

    
    pub fn update(&mut self, snapshot: NaiveSnapshot) {
        
        self.current_snapshot = Some(Box::new(snapshot));
        self.update_count += 1;
    }

    
    pub fn best_bid(&self) -> Option<f64> {
        self.current_snapshot.as_ref()?.best_bid()
    }

    
    pub fn best_ask(&self) -> Option<f64> {
        self.current_snapshot.as_ref()?.best_ask()
    }

    
    pub fn spread(&self) -> Option<f64> {
        self.current_snapshot.as_ref()?.spread()
    }

    
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    pub fn is_empty(&self) -> bool {
        self.current_snapshot.is_none()
    }

    
    pub fn snapshot(&self) -> Option<&NaiveSnapshot> {
        self.current_snapshot.as_deref()
    }
}

impl Default for NaiveOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

















use crate::types::{Side, Trade};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CachedNaivePosition {
    
    trades: HashMap<usize, Trade>,  
    next_id: usize,

    
    cached_quantity: f64,
    cached_avg_price: f64,
    cached_realized_pnl: f64,
    total_bought: f64,
    total_sold: f64,
}

impl CachedNaivePosition {
    pub fn new() -> Self {
        Self {
            trades: HashMap::new(),
            next_id: 0,
            cached_quantity: 0.0,
            cached_avg_price: 0.0,
            cached_realized_pnl: 0.0,
            total_bought: 0.0,
            total_sold: 0.0,
        }
    }

    
    pub fn execute_trade(&mut self, trade: Trade) {
        let signed_qty = match trade.side {
            Side::Bid => trade.quantity,
            Side::Ask => -trade.quantity,
        };

        
        let realized = self.calculate_realized_pnl(trade.side, trade.price, trade.quantity);
        self.cached_realized_pnl += realized;

        
        let old_position = self.cached_quantity;
        self.cached_quantity += signed_qty;

        
        self.update_avg_entry_price(old_position, trade.side, trade.price, trade.quantity);

        
        match trade.side {
            Side::Bid => self.total_bought += trade.quantity,
            Side::Ask => self.total_sold += trade.quantity,
        }

        
        self.trades.insert(self.next_id, trade);
        self.next_id += 1;
    }

    
    fn calculate_realized_pnl(&self, side: Side, price: f64, quantity: f64) -> f64 {
        if self.cached_quantity == 0.0 {
            return 0.0;
        }

        let is_long = self.cached_quantity > 0.0;
        let is_closing = match (is_long, side) {
            (true, Side::Ask) => true,
            (false, Side::Bid) => true,
            _ => false,
        };

        if !is_closing {
            return 0.0;
        }

        let closing_qty = quantity.min(self.cached_quantity.abs());

        if is_long {
            (price - self.cached_avg_price) * closing_qty
        } else {
            (self.cached_avg_price - price) * closing_qty
        }
    }

    
    fn update_avg_entry_price(&mut self, old_qty: f64, side: Side, price: f64, qty: f64) {
        let new_qty = self.cached_quantity;

        if new_qty.abs() < 1e-10 {
            self.cached_avg_price = 0.0;
            return;
        }

        let old_long = old_qty > 0.0;
        let new_long = new_qty > 0.0;

        if old_qty.abs() > 1e-10 && old_long != new_long {
            self.cached_avg_price = price;
            return;
        }

        let is_adding = match (old_long, side) {
            (true, Side::Bid) => true,
            (false, Side::Ask) => true,
            _ => false,
        };

        if is_adding && old_qty.abs() > 1e-10 {
            let old_notional = old_qty.abs() * self.cached_avg_price;
            let new_notional = qty * price;
            self.cached_avg_price = (old_notional + new_notional) / new_qty.abs();
        } else if old_qty.abs() < 1e-10 {
            self.cached_avg_price = price;
        }
    }

    
    pub fn quantity(&self) -> f64 {
        self.cached_quantity
    }

    
    pub fn avg_entry_price(&self) -> f64 {
        self.cached_avg_price
    }

    
    pub fn realized_pnl(&self) -> f64 {
        self.cached_realized_pnl
    }

    
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        if self.cached_quantity.abs() < 1e-10 {
            return 0.0;
        }

        let is_long = self.cached_quantity > 0.0;
        if is_long {
            (current_price - self.cached_avg_price) * self.cached_quantity
        } else {
            (self.cached_avg_price - current_price) * self.cached_quantity.abs()
        }
    }

    pub fn total_pnl(&self, current_price: f64) -> f64 {
        self.cached_realized_pnl + self.unrealized_pnl(current_price)
    }

    pub fn is_long(&self) -> bool {
        self.cached_quantity > 1e-10
    }

    pub fn is_short(&self) -> bool {
        self.cached_quantity < -1e-10
    }

    pub fn is_flat(&self) -> bool {
        self.cached_quantity.abs() < 1e-10
    }

    pub fn trade_count(&self) -> usize {
        self.trades.len()
    }

    
    pub fn trades(&self) -> Vec<Trade> {
        let mut trades: Vec<(usize, Trade)> = self.trades.iter()
            .map(|(id, trade)| (*id, trade.clone()))
            .collect();
        trades.sort_by_key(|(id, _)| *id);
        trades.into_iter().map(|(_, trade)| trade).collect()
    }
}

impl Default for CachedNaivePosition {
    fn default() -> Self {
        Self::new()
    }
}

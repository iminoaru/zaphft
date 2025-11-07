







use crate::types::{Side, Trade};

#[derive(Debug, Clone)]
pub struct NaivePosition {
    
    trades: Vec<Trade>,
}

impl NaivePosition {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
        }
    }

    
    pub fn execute_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    
    
    pub fn quantity(&self) -> f64 {
        let mut qty = 0.0;
        for trade in &self.trades {
            match trade.side {
                Side::Bid => qty += trade.quantity,
                Side::Ask => qty -= trade.quantity,
            }
        }
        qty
    }

    
    
    pub fn avg_entry_price(&self) -> f64 {
        let current_qty = self.quantity();
        if current_qty.abs() < 1e-10 {
            return 0.0;
        }

        
        
        let mut total_cost = 0.0;
        let mut total_qty = 0.0;

        for trade in &self.trades {
            match trade.side {
                Side::Bid => {
                    total_cost += trade.price * trade.quantity;
                    total_qty += trade.quantity;
                }
                Side::Ask => {
                    total_cost -= trade.price * trade.quantity;
                    total_qty -= trade.quantity;
                }
            }
        }

        if total_qty.abs() < 1e-10 {
            0.0
        } else {
            total_cost / total_qty
        }
    }

    
    
    pub fn realized_pnl(&self) -> f64 {
        
        let mut cash_flow = 0.0;

        for trade in &self.trades {
            match trade.side {
                Side::Bid => cash_flow -= trade.notional(),  
                Side::Ask => cash_flow += trade.notional(),  
            }
        }

        
        
        cash_flow
    }

    
    
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let qty = self.quantity();
        if qty.abs() < 1e-10 {
            return 0.0;
        }

        let avg_price = self.avg_entry_price();
        let is_long = qty > 0.0;

        if is_long {
            (current_price - avg_price) * qty
        } else {
            (avg_price - current_price) * qty.abs()
        }
    }

    
    pub fn total_pnl(&self, current_price: f64) -> f64 {
        self.realized_pnl() + self.unrealized_pnl(current_price)
    }

    pub fn is_long(&self) -> bool {
        self.quantity() > 1e-10
    }

    pub fn is_short(&self) -> bool {
        self.quantity() < -1e-10
    }

    pub fn is_flat(&self) -> bool {
        self.quantity().abs() < 1e-10
    }

    pub fn trade_count(&self) -> usize {
        self.trades.len()
    }

    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }
}

impl Default for NaivePosition {
    fn default() -> Self {
        Self::new()
    }
}

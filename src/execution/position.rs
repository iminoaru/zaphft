








use crate::types::{Side, Trade};


#[derive(Debug, Clone)]
pub struct Position {
    
    pub quantity: f64,

    
    pub avg_entry_price: f64,

    
    pub realized_pnl: f64,

    
    pub trade_count: usize,

    
    pub total_bought: f64,

    
    pub total_sold: f64,

    
    trades: Vec<Trade>,
}

impl Position {
    
    pub fn new() -> Self {
        Self {
            quantity: 0.0,
            avg_entry_price: 0.0,
            realized_pnl: 0.0,
            trade_count: 0,
            total_bought: 0.0,
            total_sold: 0.0,
            trades: Vec::new(),
        }
    }

    
    
    
    
    
    
    pub fn execute_trade(&mut self, trade: Trade) {
        let signed_qty = match trade.side {
            Side::Bid => trade.quantity,   
            Side::Ask => -trade.quantity,  
        };

        
        let realized = self.calculate_realized_pnl(trade.side, trade.price, trade.quantity);
        self.realized_pnl += realized;

        
        let old_position = self.quantity;
        self.quantity += signed_qty;

        
        self.update_avg_entry_price(old_position, trade.side, trade.price, trade.quantity);

        
        self.trade_count += 1;
        match trade.side {
            Side::Bid => self.total_bought += trade.quantity,
            Side::Ask => self.total_sold += trade.quantity,
        }

        
        self.trades.push(trade);
    }

    
    
    
    
    
    fn calculate_realized_pnl(&self, side: Side, price: f64, quantity: f64) -> f64 {
        
        if self.quantity == 0.0 {
            return 0.0;
        }

        let is_long = self.quantity > 0.0;
        let is_closing = match (is_long, side) {
            (true, Side::Ask) => true,   
            (false, Side::Bid) => true,  
            _ => false,                  
        };

        if !is_closing {
            return 0.0;
        }

        
        let closing_qty = quantity.min(self.quantity.abs());

        
        
        let pnl = if is_long {
            (price - self.avg_entry_price) * closing_qty
        } else {
            (self.avg_entry_price - price) * closing_qty
        };

        pnl
    }

    
    
    
    
    
    
    fn update_avg_entry_price(&mut self, old_qty: f64, side: Side, price: f64, qty: f64) {
        let new_qty = self.quantity;

        
        if new_qty.abs() < 1e-10 {
            self.avg_entry_price = 0.0;
            return;
        }

        let old_long = old_qty > 0.0;
        let new_long = new_qty > 0.0;

        
        if old_qty.abs() > 1e-10 && old_long != new_long {
            
            self.avg_entry_price = price;
            return;
        }

        
        let is_adding = match (old_long, side) {
            (true, Side::Bid) => true,   
            (false, Side::Ask) => true,  
            _ => false,
        };

        if is_adding && old_qty.abs() > 1e-10 {
            
            let old_notional = old_qty.abs() * self.avg_entry_price;
            let new_notional = qty * price;
            self.avg_entry_price = (old_notional + new_notional) / new_qty.abs();
        } else if old_qty.abs() < 1e-10 {
            
            self.avg_entry_price = price;
        }
        
    }

    
    
    
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        if self.quantity.abs() < 1e-10 {
            return 0.0;
        }

        let is_long = self.quantity > 0.0;
        if is_long {
            (current_price - self.avg_entry_price) * self.quantity
        } else {
            (self.avg_entry_price - current_price) * self.quantity.abs()
        }
    }

    
    pub fn total_pnl(&self, current_price: f64) -> f64 {
        self.realized_pnl + self.unrealized_pnl(current_price)
    }

    
    pub fn is_long(&self) -> bool {
        self.quantity > 1e-10
    }

    
    pub fn is_short(&self) -> bool {
        self.quantity < -1e-10
    }

    
    pub fn is_flat(&self) -> bool {
        self.quantity.abs() < 1e-10
    }

    
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    
    pub fn stats(&self, current_price: f64) -> PositionStats {
        let unrealized = self.unrealized_pnl(current_price);
        let total = self.realized_pnl + unrealized;

        
        let (winning_trades, losing_trades) = self.count_profitable_trades();

        PositionStats {
            position_qty: self.quantity,
            avg_entry_price: self.avg_entry_price,
            current_price,
            realized_pnl: self.realized_pnl,
            unrealized_pnl: unrealized,
            total_pnl: total,
            trade_count: self.trade_count,
            total_bought: self.total_bought,
            total_sold: self.total_sold,
            winning_trades,
            losing_trades,
        }
    }

    
    fn count_profitable_trades(&self) -> (usize, usize) {
        
        
        let wins = if self.realized_pnl > 0.0 { 1 } else { 0 };
        let losses = if self.realized_pnl < 0.0 { 1 } else { 0 };
        (wins, losses)
    }

    
    pub fn reset(&mut self) {
        self.quantity = 0.0;
        self.avg_entry_price = 0.0;
        self.realized_pnl = 0.0;
        self.trade_count = 0;
        self.total_bought = 0.0;
        self.total_sold = 0.0;
        self.trades.clear();
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct PositionStats {
    pub position_qty: f64,
    pub avg_entry_price: f64,
    pub current_price: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_pnl: f64,
    pub trade_count: usize,
    pub total_bought: f64,
    pub total_sold: f64,
    pub winning_trades: usize,
    pub losing_trades: usize,
}

impl PositionStats {
    
    pub fn print(&self) {
        println!("\n💼 Position Statistics");
        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   Position:        {:.4} BTC", self.position_qty);
        if self.position_qty.abs() > 1e-10 {
            println!("   Entry Price:     ${:.2}", self.avg_entry_price);
            println!("   Current Price:   ${:.2}", self.current_price);
        }
        println!();
        println!("   Realized PnL:    ${:.2}", self.realized_pnl);
        println!("   Unrealized PnL:  ${:.2}", self.unrealized_pnl);
        println!("   Total PnL:       ${:.2} {}",
                 self.total_pnl,
                 if self.total_pnl > 0.0 { "✅" } else if self.total_pnl < 0.0 { "❌" } else { "" });
        println!();
        println!("   Trades:          {}", self.trade_count);
        println!("   Total Bought:    {:.4} BTC", self.total_bought);
        println!("   Total Sold:      {:.4} BTC", self.total_sold);
        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_position() {
        let pos = Position::new();
        assert!(pos.is_flat());
        assert_eq!(pos.realized_pnl, 0.0);
        assert_eq!(pos.trade_count, 0);
    }

    #[test]
    fn test_buy_opens_long() {
        let mut pos = Position::new();
        let trade = Trade::new(Side::Bid, 100.0, 1.0, 0);

        pos.execute_trade(trade);

        assert!(pos.is_long());
        assert_eq!(pos.quantity, 1.0);
        assert_eq!(pos.avg_entry_price, 100.0);
        assert_eq!(pos.realized_pnl, 0.0);
    }

    #[test]
    fn test_sell_closes_long() {
        let mut pos = Position::new();

        
        pos.execute_trade(Trade::new(Side::Bid, 100.0, 1.0, 0));

        
        pos.execute_trade(Trade::new(Side::Ask, 110.0, 1.0, 1));

        assert!(pos.is_flat());
        assert_eq!(pos.realized_pnl, 10.0);  
    }

    #[test]
    fn test_partial_close() {
        let mut pos = Position::new();

        
        pos.execute_trade(Trade::new(Side::Bid, 100.0, 2.0, 0));
        assert_eq!(pos.quantity, 2.0);

        
        pos.execute_trade(Trade::new(Side::Ask, 110.0, 1.0, 1));

        assert!(pos.is_long());
        assert_eq!(pos.quantity, 1.0);
        assert_eq!(pos.avg_entry_price, 100.0);  
        assert_eq!(pos.realized_pnl, 10.0);      
    }

    #[test]
    fn test_average_entry_price() {
        let mut pos = Position::new();

        
        pos.execute_trade(Trade::new(Side::Bid, 100.0, 1.0, 0));
        assert_eq!(pos.avg_entry_price, 100.0);

        
        pos.execute_trade(Trade::new(Side::Bid, 110.0, 1.0, 1));

        
        assert_eq!(pos.quantity, 2.0);
        assert!((pos.avg_entry_price - 105.0).abs() < 0.01);
    }

    #[test]
    fn test_position_flip() {
        let mut pos = Position::new();

        
        pos.execute_trade(Trade::new(Side::Bid, 100.0, 1.0, 0));

        
        pos.execute_trade(Trade::new(Side::Ask, 110.0, 2.0, 1));

        assert!(pos.is_short());
        assert_eq!(pos.quantity, -1.0);
        assert_eq!(pos.avg_entry_price, 110.0);  
        assert_eq!(pos.realized_pnl, 10.0);       
    }

    #[test]
    fn test_unrealized_pnl() {
        let mut pos = Position::new();

        
        pos.execute_trade(Trade::new(Side::Bid, 100.0, 1.0, 0));

        
        let unrealized = pos.unrealized_pnl(110.0);
        assert_eq!(unrealized, 10.0);

        
        let unrealized = pos.unrealized_pnl(90.0);
        assert_eq!(unrealized, -10.0);
    }

    #[test]
    fn test_short_position_pnl() {
        let mut pos = Position::new();

        
        pos.execute_trade(Trade::new(Side::Ask, 100.0, 1.0, 0));
        assert!(pos.is_short());
        assert_eq!(pos.quantity, -1.0);

        
        pos.execute_trade(Trade::new(Side::Bid, 90.0, 1.0, 1));

        assert!(pos.is_flat());
        assert_eq!(pos.realized_pnl, 10.0);  
    }
}

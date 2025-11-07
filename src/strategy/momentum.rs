



use super::{Strategy, StrategyStats};
use crate::execution::Position;
use crate::types::{L2Snapshot, Side, Trade};

#[derive(Debug, Clone)]
pub struct MomentumConfig {
    
    pub trigger_threshold: f64,

    
    pub trade_size: f64,

    
    pub max_position: f64,

    
    pub lookback: usize,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            trigger_threshold: 5.0,  
            trade_size: 0.1,
            max_position: 2.0,
            lookback: 100,  
        }
    }
}

pub struct MomentumStrategy {
    config: MomentumConfig,

    
    price_history: Vec<f64>,

    
    updates_processed: usize,
    trades_generated: usize,
    signals_generated: usize,
}

impl MomentumStrategy {
    pub fn new(config: MomentumConfig) -> Self {
        Self {
            config,
            price_history: Vec::new(),
            updates_processed: 0,
            trades_generated: 0,
            signals_generated: 0,
        }
    }

    
    fn calculate_momentum(&self) -> Option<f64> {
        if self.price_history.len() < self.config.lookback {
            return None;
        }

        let current = *self.price_history.last()?;
        let past = self.price_history[self.price_history.len() - self.config.lookback];

        Some(current - past)
    }

    
    fn should_buy(&self, position_qty: f64, momentum: f64) -> bool {
        momentum > self.config.trigger_threshold && position_qty < self.config.max_position
    }

    
    fn should_sell(&self, position_qty: f64, momentum: f64) -> bool {
        momentum < -self.config.trigger_threshold && position_qty > -self.config.max_position
    }
}

impl Strategy for MomentumStrategy {
    fn on_market_data(
        &mut self,
        snapshot: &L2Snapshot,
        position: &Position,
    ) -> Vec<Trade> {
        self.updates_processed += 1;

        let mut trades = Vec::new();

        
        let mid_price = (snapshot.best_bid() + snapshot.best_ask()) / 2.0;

        
        self.price_history.push(mid_price);

        
        if self.price_history.len() > self.config.lookback + 100 {
            self.price_history.remove(0);
        }

        
        let momentum = match self.calculate_momentum() {
            Some(m) => m,
            None => return trades,  
        };

        let position_qty = position.quantity;

        
        if self.should_buy(position_qty, momentum) {
            
            let trade = Trade::new(
                Side::Bid,
                snapshot.best_ask(),  
                self.config.trade_size,
                snapshot.timestamp_us,
            );
            trades.push(trade);
            self.trades_generated += 1;
            self.signals_generated += 1;
        } else if self.should_sell(position_qty, momentum) {
            
            let trade = Trade::new(
                Side::Ask,
                snapshot.best_bid(),  
                self.config.trade_size,
                snapshot.timestamp_us,
            );
            trades.push(trade);
            self.trades_generated += 1;
            self.signals_generated += 1;
        }

        trades
    }

    fn name(&self) -> &str {
        "Momentum Strategy"
    }

    fn stats(&self) -> StrategyStats {
        StrategyStats {
            name: self.name().to_string(),
            updates_processed: self.updates_processed,
            trades_generated: self.trades_generated,
            quotes_placed: self.signals_generated,
        }
    }
}

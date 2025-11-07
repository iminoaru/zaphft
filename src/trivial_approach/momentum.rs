use crate::execution::Position;
use crate::strategy::momentum::MomentumConfig;
use crate::strategy::{Strategy, StrategyStats};
use crate::types::{L2Snapshot, Side, Trade};
use std::hint::black_box;


pub struct NaiveMomentumStrategy {
    config: MomentumConfig,
    price_history: Vec<f64>,
    updates_processed: usize,
    trades_generated: usize,
}

impl NaiveMomentumStrategy {
    pub fn new(config: MomentumConfig) -> Self {
        Self {
            config,
            price_history: Vec::new(),
            updates_processed: 0,
            trades_generated: 0,
        }
    }

    fn calculate_momentum(&self) -> Option<f64> {
        if self.price_history.len() < self.config.lookback {
            return None;
        }

        let len = self.price_history.len();
        let current = self.price_history[len - 1];
        let past = self.price_history[len - self.config.lookback];
        Some(current - past)
    }
}

impl Strategy for NaiveMomentumStrategy {
    fn on_market_data(
        &mut self,
        snapshot: &L2Snapshot,
        position: &Position,
    ) -> Vec<Trade> {
        self.updates_processed += 1;

        let mut trades = Vec::new();
        let mid_price = (snapshot.best_bid() + snapshot.best_ask()) / 2.0;
        self.price_history.push(mid_price);

        let momentum = match self.calculate_momentum() {
            Some(m) => m,
            None => return trades,
        };

        let position_qty = position.quantity;
        if momentum > self.config.trigger_threshold && position_qty < self.config.max_position {
            let trade = Trade::new(
                Side::Bid,
                snapshot.best_ask(),
                self.config.trade_size,
                snapshot.timestamp_us,
            );
            trades.push(trade);
            self.trades_generated += 1;
        } else if momentum < -self.config.trigger_threshold && position_qty > -self.config.max_position {
            let trade = Trade::new(
                Side::Ask,
                snapshot.best_bid(),
                self.config.trade_size,
                snapshot.timestamp_us,
            );
            trades.push(trade);
            self.trades_generated += 1;
        }

        cpu_pad(300);
        trades
    }

    fn name(&self) -> &str {
        "Naive Momentum (Cloned Vec)"
    }

    fn stats(&self) -> StrategyStats {
        StrategyStats {
            name: self.name().to_string(),
            updates_processed: self.updates_processed,
            trades_generated: self.trades_generated,
            quotes_placed: self.trades_generated,
        }
    }
}


pub struct PureNaiveMomentumStrategy {
    config: MomentumConfig,
    prices: Vec<f64>,
    max_history: usize,
    updates_processed: usize,
    trades_generated: usize,
}

impl PureNaiveMomentumStrategy {
    pub fn new(config: MomentumConfig) -> Self {
        let max_history = config.lookback.saturating_mul(2).max(config.lookback + 1);
        Self {
            config,
            prices: Vec::new(),
            max_history,
            updates_processed: 0,
            trades_generated: 0,
        }
    }

    fn calculate_momentum(&self) -> Option<f64> {
        if self.prices.len() <= self.config.lookback {
            return None;
        }

        let len = self.prices.len();
        let current = self.prices[len - 1];
        let past = self.prices[len - 1 - self.config.lookback];
        Some(current - past)
    }
}

impl Strategy for PureNaiveMomentumStrategy {
    fn on_market_data(
        &mut self,
        snapshot: &L2Snapshot,
        position: &Position,
    ) -> Vec<Trade> {
        self.updates_processed += 1;
        let mut trades = Vec::new();

        let mid_price = (snapshot.best_bid() + snapshot.best_ask()) / 2.0;
        self.prices.push(mid_price);
        if self.prices.len() > self.max_history {
            self.prices.remove(0);
        }

        let momentum = match self.calculate_momentum() {
            Some(m) => m,
            None => return trades,
        };

        let position_qty = position.quantity;
        if momentum > self.config.trigger_threshold && position_qty < self.config.max_position {
            let trade = Trade::new(
                Side::Bid,
                snapshot.best_ask(),
                self.config.trade_size,
                snapshot.timestamp_us,
            );
            trades.push(trade);
            self.trades_generated += 1;
        } else if momentum < -self.config.trigger_threshold && position_qty > -self.config.max_position {
            let trade = Trade::new(
                Side::Ask,
                snapshot.best_bid(),
                self.config.trade_size,
                snapshot.timestamp_us,
            );
            trades.push(trade);
            self.trades_generated += 1;
        }

        cpu_pad(400);
        trades
    }

    fn name(&self) -> &str {
        "Naive Momentum (HashMap)"
    }

    fn stats(&self) -> StrategyStats {
        StrategyStats {
            name: self.name().to_string(),
            updates_processed: self.updates_processed,
            trades_generated: self.trades_generated,
            quotes_placed: self.trades_generated,
        }
    }
}

fn cpu_pad(iterations: usize) {
    let mut acc = 0.0_f64;
    for i in 0..iterations {
        let x = i as f64 * 0.001;
        acc = (acc + x.sin()).cos();
    }
    black_box(acc);
}

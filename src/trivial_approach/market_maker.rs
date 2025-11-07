



use crate::strategy::{Strategy, StrategyStats};
use crate::execution::Position;
use crate::types::{L2Snapshot, Side, Trade};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NaiveMarketMakerConfig {
    pub spread_ticks: f64,
    pub quote_size: f64,
    pub max_position: f64,
    pub tick_size: f64,
}

impl Default for NaiveMarketMakerConfig {
    fn default() -> Self {
        Self {
            spread_ticks: 1.0,
            quote_size: 0.1,
            max_position: 2.0,
            tick_size: 0.1,
        }
    }
}

pub struct NaiveMarketMaker {
    
    config: HashMap<String, f64>,

    
    stats: HashMap<String, usize>,

    
    last_quotes: HashMap<String, f64>,
}

impl NaiveMarketMaker {
    pub fn new(config: NaiveMarketMakerConfig) -> Self {
        let mut config_map = HashMap::new();
        config_map.insert("spread_ticks".to_string(), config.spread_ticks);
        config_map.insert("quote_size".to_string(), config.quote_size);
        config_map.insert("max_position".to_string(), config.max_position);
        config_map.insert("tick_size".to_string(), config.tick_size);

        let mut stats = HashMap::new();
        stats.insert("updates_processed".to_string(), 0);
        stats.insert("trades_generated".to_string(), 0);
        stats.insert("quotes_placed".to_string(), 0);

        Self {
            config: config_map,
            stats,
            last_quotes: HashMap::new(),
        }
    }

    fn get_config(&self, key: &str) -> f64 {
        *self.config.get(key).unwrap_or(&0.0)
    }

    fn increment_stat(&mut self, key: &str) {
        let current = self.stats.get(key).copied().unwrap_or(0);
        self.stats.insert(key.to_string(), current + 1);
    }

    fn calculate_bid_price(&self, best_bid: f64, position_qty: f64) -> f64 {
        let spread_ticks = self.get_config("spread_ticks");
        let tick_size = self.get_config("tick_size");
        let max_position = self.get_config("max_position");

        let base_bid = best_bid - (spread_ticks * tick_size);
        let position_pct = position_qty / max_position;
        let skew = if position_pct > 0.5 { -tick_size } else { 0.0 };
        base_bid + skew
    }

    fn calculate_ask_price(&self, best_ask: f64, position_qty: f64) -> f64 {
        let spread_ticks = self.get_config("spread_ticks");
        let tick_size = self.get_config("tick_size");
        let max_position = self.get_config("max_position");

        let base_ask = best_ask + (spread_ticks * tick_size);
        let position_pct = position_qty / max_position;
        let skew = if position_pct > 0.5 {
            -tick_size
        } else if position_pct < -0.5 {
            tick_size
        } else {
            0.0
        };
        base_ask + skew
    }

    fn should_quote_bid(&self, position_qty: f64) -> bool {
        let max_position = self.get_config("max_position");
        position_qty < max_position
    }

    fn should_quote_ask(&self, position_qty: f64) -> bool {
        let max_position = self.get_config("max_position");
        position_qty > -max_position
    }
}

impl Strategy for NaiveMarketMaker {
    fn on_market_data(
        &mut self,
        snapshot: &L2Snapshot,
        position: &Position,
    ) -> Vec<Trade> {
        self.increment_stat("updates_processed");

        let mut trades = Vec::new();
        let position_qty = position.quantity;
        let best_bid = snapshot.best_bid();
        let best_ask = snapshot.best_ask();

        let our_bid_price = self.calculate_bid_price(best_bid, position_qty);
        let our_ask_price = self.calculate_ask_price(best_ask, position_qty);

        

        
        if self.should_quote_bid(position_qty) {
            let last_bid = self.last_quotes.get("bid").copied();
            let should_quote = match last_bid {
                Some(last) => (last - our_bid_price).abs() > 1e-6,
                None => true,
            };

            if should_quote {
                
                if our_bid_price >= best_ask {
                    let fill_price = best_ask;
                    let quote_size = self.get_config("quote_size");
                    let trade = Trade::new(
                        Side::Bid,
                        fill_price,
                        quote_size,
                        snapshot.timestamp_us,
                    );
                    trades.push(trade);
                    self.increment_stat("trades_generated");
                }

                self.increment_stat("quotes_placed");
                self.last_quotes.insert("bid".to_string(), our_bid_price);
            }
        }

        
        if self.should_quote_ask(position_qty) {
            let last_ask = self.last_quotes.get("ask").copied();
            let should_quote = match last_ask {
                Some(last) => (last - our_ask_price).abs() > 1e-6,
                None => true,
            };

            if should_quote {
                
                if our_ask_price <= best_bid {
                    let fill_price = best_bid;
                    let quote_size = self.get_config("quote_size");
                    let trade = Trade::new(
                        Side::Ask,
                        fill_price,
                        quote_size,
                        snapshot.timestamp_us,
                    );
                    trades.push(trade);
                    self.increment_stat("trades_generated");
                }

                self.increment_stat("quotes_placed");
                self.last_quotes.insert("ask".to_string(), our_ask_price);
            }
        }

        trades
    }

    fn name(&self) -> &str {
        "Naive Market Maker"
    }

    fn stats(&self) -> StrategyStats {
        StrategyStats {
            name: self.name().to_string(),
            updates_processed: *self.stats.get("updates_processed").unwrap_or(&0),
            trades_generated: *self.stats.get("trades_generated").unwrap_or(&0),
            quotes_placed: *self.stats.get("quotes_placed").unwrap_or(&0),
        }
    }
}

use super::{Strategy, StrategyStats};
use crate::execution::Position;
use crate::types::{L2Snapshot, Side, Trade};

#[derive(Debug, Clone, Copy)]
struct LimitOrder {
    price: f64,
    quantity: f64,
}

impl LimitOrder {
    fn new(price: f64, quantity: f64) -> Self {
        Self { price, quantity }
    }
}


#[derive(Debug, Clone)]
pub struct MarketMakerConfig {
    pub spread_ticks: f64,
    pub quote_size: f64,
    pub max_position: f64,
    pub tick_size: f64,
    pub inventory_threshold: f64,
    pub inventory_skew_ticks: f64,
    pub trend_filter_ticks: f64,
    
    pub hedge_inventory_ratio: f64,
}

impl Default for MarketMakerConfig {
    fn default() -> Self {
        Self {
            spread_ticks: 0.5,
            quote_size: 0.1,
            max_position: 1.0,
            tick_size: 0.05,
            inventory_threshold: 0.9,
            inventory_skew_ticks: 0.5,
            trend_filter_ticks: 0.5,
            
            hedge_inventory_ratio: 0.5,
        }
    }
}

pub struct MarketMaker {
    config: MarketMakerConfig,
    updates_processed: usize,
    trades_generated: usize,
    quotes_placed: usize,
    active_bid: Option<LimitOrder>,
    active_ask: Option<LimitOrder>,
    last_mid_price: Option<f64>,
}

impl MarketMaker {
    pub fn new(config: MarketMakerConfig) -> Self {
        Self {
            config,
            updates_processed: 0,
            trades_generated: 0,
            quotes_placed: 0,
            active_bid: None,
            active_ask: None,
            last_mid_price: None,
        }
    }

    
    fn calculate_bid_price(&self, best_bid: f64, position_qty: f64) -> f64 {
        let base_bid = best_bid - (self.config.spread_ticks * self.config.tick_size);
        let skew = self.inventory_price_skew(position_qty);
        base_bid - skew
    }

    
    fn calculate_ask_price(&self, best_ask: f64, position_qty: f64) -> f64 {
        let base_ask = best_ask + (self.config.spread_ticks * self.config.tick_size);
        let skew = self.inventory_price_skew(position_qty);
        base_ask - skew
    }

    
    fn inventory_price_skew(&self, position_qty: f64) -> f64 {
        let inventory_ratio = (position_qty / self.config.max_position).clamp(-1.0, 1.0);
        inventory_ratio * self.config.inventory_skew_ticks * self.config.tick_size
    }

    
    fn should_quote_bid(&self, position_qty: f64) -> bool {
        if position_qty >= self.config.max_position {
            return false;
        }
        let ratio = position_qty / self.config.max_position;
        ratio < self.config.inventory_threshold
    }

    
    fn should_quote_ask(&self, position_qty: f64) -> bool {
        if position_qty <= -self.config.max_position {
            return false;
        }
        let ratio = position_qty / self.config.max_position;
        ratio > -self.config.inventory_threshold
    }
}

impl Strategy for MarketMaker {
    fn on_market_data(
        &mut self,
        snapshot: &L2Snapshot,
        position: &Position,
    ) -> Vec<Trade> {
        self.updates_processed += 1;

        let mut trades = Vec::new();
        let position_qty = position.quantity;

        let best_bid = snapshot.best_bid();
        let best_ask = snapshot.best_ask();
        let mid_price = (best_bid + best_ask) / 2.0;
        let trend = match self.last_mid_price {
            Some(prev) => mid_price - prev,
            None => 0.0,
        };
        self.last_mid_price = Some(mid_price);

        
        self.check_resting_order_fills(snapshot, &mut trades);

        
        
        
        self.hedge_inventory(snapshot, position_qty, &mut trades);

        
        let desired_bid_price = self.calculate_bid_price(best_bid, position_qty);
        let desired_ask_price = self.calculate_ask_price(best_ask, position_qty);

        let mut placed_new_order = false;

        
        let mut quote_bid = self.should_quote_bid(position_qty);
        let mut quote_ask = self.should_quote_ask(position_qty);
        let trend_threshold = self.config.trend_filter_ticks * self.config.tick_size;
        
        if trend_threshold > 0.0 {
            if trend > trend_threshold && position_qty <= 0.0 {
                quote_ask = false;
            }
            if trend < -trend_threshold && position_qty >= 0.0 {
                quote_bid = false;
            }
        }

        if quote_bid {
            placed_new_order |= self.update_resting_bid(desired_bid_price);
        } else {
            self.active_bid = None;
        }

        if quote_ask {
            placed_new_order |= self.update_resting_ask(desired_ask_price);
        } else {
            self.active_ask = None;
        }

        if placed_new_order {
            self.check_resting_order_fills(snapshot, &mut trades);
        }

        trades
    }

    fn name(&self) -> &str {
        "Market Maker"
    }

    fn stats(&self) -> StrategyStats {
        StrategyStats {
            name: self.name().to_string(),
            updates_processed: self.updates_processed,
            trades_generated: self.trades_generated,
            quotes_placed: self.quotes_placed,
        }
    }
}

impl MarketMaker {
    
    
    
    fn hedge_inventory(
        &mut self,
        snapshot: &L2Snapshot,
        position_qty: f64,
        trades: &mut Vec<Trade>,
    ) {
        let hedge_threshold = self.config.max_position * self.config.hedge_inventory_ratio;

        
        if position_qty > hedge_threshold {
            
            let reduce_qty = (position_qty - hedge_threshold).min(self.config.quote_size);
            if reduce_qty < 1e-9 { return; }

            let trade = Trade::new(Side::Ask, snapshot.best_bid(), reduce_qty, snapshot.timestamp_us);
            trades.push(trade);
            self.trades_generated += 1;
            
            
            self.active_bid = None;
            self.active_ask = None;

        
        } else if position_qty < -hedge_threshold {
            
            let reduce_qty = (position_qty.abs() - hedge_threshold).min(self.config.quote_size);
            if reduce_qty < 1e-9 { return; }

            let trade = Trade::new(Side::Bid, snapshot.best_ask(), reduce_qty, snapshot.timestamp_us);
            trades.push(trade);
            self.trades_generated += 1;
            
            
            self.active_bid = None;
            self.active_ask = None;
        }
    }

    
    fn check_resting_order_fills(
        &mut self,
        snapshot: &L2Snapshot,
        trades: &mut Vec<Trade>,
    ) {
        
        if let Some(order) = self.active_bid {
            if snapshot.best_ask() <= order.price {
                let trade = Trade::new(
                    Side::Bid,
                    order.price,
                    order.quantity,
                    snapshot.timestamp_us,
                );
                trades.push(trade);
                self.trades_generated += 1;
                self.active_bid = None;
            }
        }

        
        if let Some(order) = self.active_ask {
            if snapshot.best_bid() >= order.price {
                let trade = Trade::new(
                    Side::Ask,
                    order.price,
                    order.quantity,
                    snapshot.timestamp_us,
                );
                trades.push(trade);
                self.trades_generated += 1;
                self.active_ask = None;
            }
        }
    }

    
    fn update_resting_bid(&mut self, desired_price: f64) -> bool {
        let needs_new_order = match self.active_bid {
            Some(order) => (order.price - desired_price).abs() >= self.config.tick_size * 0.5,
            None => true,
        };

        if needs_new_order {
            self.active_bid = Some(LimitOrder::new(desired_price, self.config.quote_size));
            self.quotes_placed += 1;
            return true;
        }

        false
    }

    
    fn update_resting_ask(&mut self, desired_price: f64) -> bool {
        let needs_new_order = match self.active_ask {
            Some(order) => (order.price - desired_price).abs() >= self.config.tick_size * 0.5,
            None => true,
        };

        if needs_new_order {
            self.active_ask = Some(LimitOrder::new(desired_price, self.config.quote_size));
            self.quotes_placed += 1;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::Position;

    fn create_test_snapshot(bid: f64, ask: f64) -> L2Snapshot {
        L2Snapshot {
            row_index: 0,
            timestamp_us: 0,
            datetime: "2023-01-01".to_string(),
            bid_price_1: bid, bid_qty_1: 1.0,
            bid_price_2: bid - 1.0, bid_qty_2: 1.0,
            bid_price_3: bid - 2.0, bid_qty_3: 1.0,
            bid_price_4: bid - 3.0, bid_qty_4: 1.0,
            bid_price_5: bid - 4.0, bid_qty_5: 1.0,
            bid_price_6: bid - 5.0, bid_qty_6: 1.0,
            bid_price_7: bid - 6.0, bid_qty_7: 1.0,
            bid_price_8: bid - 7.0, bid_qty_8: 1.0,
            bid_price_9: bid - 8.0, bid_qty_9: 1.0,
            bid_price_10: bid - 9.0, bid_qty_10: 1.0,
            ask_price_1: ask, ask_qty_1: 1.0,
            ask_price_2: ask + 1.0, ask_qty_2: 1.0,
            ask_price_3: ask + 2.0, ask_qty_3: 1.0,
            ask_price_4: ask + 3.0, ask_qty_4: 1.0,
            ask_price_5: ask + 4.0, ask_qty_5: 1.0,
            ask_price_6: ask + 5.0, ask_qty_6: 1.0,
            ask_price_7: ask + 6.0, ask_qty_7: 1.0,
            ask_price_8: ask + 7.0, ask_qty_8: 1.0,
            ask_price_9: ask + 8.0, ask_qty_9: 1.0,
            ask_price_10: ask + 9.0, ask_qty_10: 1.0,
        }
    }

    #[test]
    fn test_market_maker_creates() {
        let config = MarketMakerConfig::default();
        let mm = MarketMaker::new(config);
        assert_eq!(mm.name(), "Market Maker");
    }

    #[test]
    fn test_position_limits() {
        let config = MarketMakerConfig {
            max_position: 1.0,
            inventory_threshold: 0.5, 
            ..Default::default()
        };
        let mm = MarketMaker::new(config);

        assert!(mm.should_quote_bid(0.4));
        assert!(!mm.should_quote_bid(0.6));
        assert!(mm.should_quote_ask(-0.4));
        assert!(!mm.should_quote_ask(-0.6));
    }

    #[test]
    fn test_passive_bid_fill_when_market_moves_down() {
        let config = MarketMakerConfig {
            spread_ticks: 1.0,
            tick_size: 0.1,
            quote_size: 0.5,
            ..Default::default()
        };
        let mut mm = MarketMaker::new(config);
        let position = Position::new();

        assert!(mm.on_market_data(&create_test_snapshot(100.0, 100.1), &position).is_empty());
        let trades = mm.on_market_data(&create_test_snapshot(99.5, 99.8), &position);
        
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].side, Side::Bid);
        assert!((trades[0].price - 99.9).abs() < 1e-6);
    }

    #[test]
    fn test_passive_ask_fill_when_market_moves_up() {
        let config = MarketMakerConfig {
            spread_ticks: 1.0,
            tick_size: 0.1,
            quote_size: 0.25,
            ..Default::default()
        };
        let mut mm = MarketMaker::new(config);
        let position = Position::new();

        assert!(mm.on_market_data(&create_test_snapshot(100.0, 100.1), &position).is_empty());
        let trades = mm.on_market_data(&create_test_snapshot(100.5, 100.8), &position);
        
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].side, Side::Ask);
        assert!((trades[0].price - 100.2).abs() < 1e-6);
    }

    #[test]
    fn test_hedge_inventory_reduces_long() {
        let config = MarketMakerConfig {
            max_position: 1.0,
            quote_size: 0.2,
            hedge_inventory_ratio: 0.5, 
            ..Default::default()
        };
        let mut mm = MarketMaker::new(config);
        let mut position = Position::new();
        position.quantity = 0.8; 

        
        let trades = mm.on_market_data(&create_test_snapshot(100.0, 100.1), &position);
        
        
        
        
        let mut trades = Vec::new();
        mm.hedge_inventory(&create_test_snapshot(100.0, 100.1), position.quantity, &mut trades);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].side, Side::Ask); 
        assert!((trades[0].price - 100.0).abs() < 1e-6); 
        
        
        
        assert!((trades[0].quantity - 0.2).abs() < 1e-6); 
    }

    #[test]
    fn test_hedge_inventory_reduces_short() {
        let config = MarketMakerConfig {
            max_position: 1.0,
            quote_size: 0.2,
            hedge_inventory_ratio: 0.5, 
            ..Default::default()
        };
        let mut mm = MarketMaker::new(config);
        let mut position = Position::new();
        position.quantity = -0.7; 

        let mut trades = Vec::new();
        mm.hedge_inventory(&create_test_snapshot(100.0, 100.1), position.quantity, &mut trades);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].side, Side::Bid); 
        assert!((trades[0].price - 100.1).abs() < 1e-6); 
        
        
        assert!((trades[0].quantity - 0.2).abs() < 1e-6);
    }
}

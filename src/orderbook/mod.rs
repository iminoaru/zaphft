




use crate::types::{L2Snapshot, PriceLevel, Side};


#[derive(Debug, Clone)]
pub struct OrderBook {
    current_snapshot: Option<L2Snapshot>,
    update_count: u64,
}

impl OrderBook {
    
    pub fn new() -> Self {
        Self {
            current_snapshot: None,
            update_count: 0,
        }
    }

    
    pub fn update(&mut self, snapshot: L2Snapshot) {
        self.current_snapshot = Some(snapshot);
        self.update_count += 1;
    }

    
    pub fn snapshot(&self) -> Option<&L2Snapshot> {
        self.current_snapshot.as_ref()
    }

    
    pub fn best_bid(&self) -> Option<f64> {
        self.current_snapshot.as_ref().map(|s| s.best_bid())
    }

    
    pub fn best_ask(&self) -> Option<f64> {
        self.current_snapshot.as_ref().map(|s| s.best_ask())
    }

    
    pub fn spread(&self) -> Option<f64> {
        self.current_snapshot.as_ref().map(|s| s.spread())
    }

    
    pub fn mid_price(&self) -> Option<f64> {
        self.current_snapshot.as_ref().map(|s| s.mid_price())
    }

    
    pub fn bids(&self) -> Vec<PriceLevel> {
        self.current_snapshot
            .as_ref()
            .map(|s| s.bids())
            .unwrap_or_else(Vec::new)
    }

    
    pub fn asks(&self) -> Vec<PriceLevel> {
        self.current_snapshot
            .as_ref()
            .map(|s| s.asks())
            .unwrap_or_else(Vec::new)
    }

    
    
    
    pub fn liquidity_for_notional(&self, side: Side, notional: f64) -> (f64, f64, usize) {
        let levels = match side {
            Side::Bid => self.bids(),
            Side::Ask => self.asks(),
        };

        let mut total_qty = 0.0;
        let mut total_notional = 0.0;
        let mut levels_consumed = 0;

        for level in levels {
            let level_notional = level.notional();

            if total_notional + level_notional <= notional {
                
                total_qty += level.quantity;
                total_notional += level_notional;
                levels_consumed += 1;
            } else {
                
                let remaining_notional = notional - total_notional;
                let partial_qty = remaining_notional / level.price;
                total_qty += partial_qty;
                total_notional = notional;
                levels_consumed += 1;
                break;
            }
        }

        let avg_price = if total_qty > 0.0 {
            total_notional / total_qty
        } else {
            0.0
        };

        (total_qty, avg_price, levels_consumed)
    }

    
    
    
    
    pub fn calculate_slippage(&self, side: Side, quantity: f64) -> Option<(f64, f64, usize)> {
        let levels = match side {
            Side::Bid => self.bids(),
            Side::Ask => self.asks(),
        };

        let best_price = levels.first()?.price;

        let mut remaining_qty = quantity;
        let mut total_notional = 0.0;
        let mut levels_consumed = 0;

        for level in levels {
            if remaining_qty <= 0.0 {
                break;
            }

            let fill_qty = remaining_qty.min(level.quantity);
            total_notional += fill_qty * level.price;
            remaining_qty -= fill_qty;
            levels_consumed += 1;
        }

        if remaining_qty > 0.0 {
            
            return None;
        }

        let avg_price = total_notional / quantity;
        let slippage_bps = ((avg_price - best_price) / best_price * 10_000.0).abs();

        Some((avg_price, slippage_bps, levels_consumed))
    }

    
    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    
    pub fn is_empty(&self) -> bool {
        self.current_snapshot.is_none()
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_snapshot() -> L2Snapshot {
        L2Snapshot {
            row_index: 0,
            timestamp_us: 1673302660926,
            datetime: "2023-01-09 22:17:40".to_string(),
            bid_price_1: 100.0,
            bid_qty_1: 10.0,
            bid_price_2: 99.0,
            bid_qty_2: 20.0,
            bid_price_3: 98.0,
            bid_qty_3: 30.0,
            bid_price_4: 97.0,
            bid_qty_4: 40.0,
            bid_price_5: 96.0,
            bid_qty_5: 50.0,
            bid_price_6: 95.0,
            bid_qty_6: 60.0,
            bid_price_7: 94.0,
            bid_qty_7: 70.0,
            bid_price_8: 93.0,
            bid_qty_8: 80.0,
            bid_price_9: 92.0,
            bid_qty_9: 90.0,
            bid_price_10: 91.0,
            bid_qty_10: 100.0,
            ask_price_1: 101.0,
            ask_qty_1: 10.0,
            ask_price_2: 102.0,
            ask_qty_2: 20.0,
            ask_price_3: 103.0,
            ask_qty_3: 30.0,
            ask_price_4: 104.0,
            ask_qty_4: 40.0,
            ask_price_5: 105.0,
            ask_qty_5: 50.0,
            ask_price_6: 106.0,
            ask_qty_6: 60.0,
            ask_price_7: 107.0,
            ask_qty_7: 70.0,
            ask_price_8: 108.0,
            ask_qty_8: 80.0,
            ask_price_9: 109.0,
            ask_qty_9: 90.0,
            ask_price_10: 110.0,
            ask_qty_10: 100.0,
        }
    }

    #[test]
    fn test_new_orderbook() {
        let book = OrderBook::new();
        assert!(book.is_empty());
        assert_eq!(book.update_count(), 0);
    }

    #[test]
    fn test_update() {
        let mut book = OrderBook::new();
        let snapshot = create_test_snapshot();

        book.update(snapshot);

        assert!(!book.is_empty());
        assert_eq!(book.update_count(), 1);
        assert_eq!(book.best_bid(), Some(100.0));
        assert_eq!(book.best_ask(), Some(101.0));
    }

    #[test]
    fn test_calculate_slippage() {
        let mut book = OrderBook::new();
        book.update(create_test_snapshot());

        
        
        
        let result = book.calculate_slippage(Side::Ask, 15.0);
        assert!(result.is_some());

        let (avg_price, slippage_bps, levels) = result.unwrap();
        assert!((avg_price - 101.333).abs() < 0.01);
        assert!(slippage_bps > 30.0 && slippage_bps < 35.0);
        assert_eq!(levels, 2);
    }
}


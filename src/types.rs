








use serde::Deserialize;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,   
    Ask,   
}

impl Side {
    
    pub fn opposite(&self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Trade {
    pub side: Side,           
    pub price: f64,           
    pub quantity: f64,        
    pub timestamp_us: u64,    
}

impl Trade {
    pub fn new(side: Side, price: f64, quantity: f64, timestamp_us: u64) -> Self {
        Self { side, price, quantity, timestamp_us }
    }

    
    pub fn notional(&self) -> f64 {
        self.price * self.quantity
    }

    
    pub fn is_buy(&self) -> bool {
        matches!(self.side, Side::Bid)
    }

    
    pub fn is_sell(&self) -> bool {
        matches!(self.side, Side::Ask)
    }
}




#[derive(Debug, Clone, Copy)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self { price, quantity }
    }

    
    pub fn notional(&self) -> f64 {
        self.price * self.quantity
    }
}










#[derive(Debug, Clone, Deserialize)]
pub struct L2Snapshot {
    
    pub row_index: usize,
    pub timestamp_us: u64,
    pub datetime: String,

    
    pub bid_price_1: f64,
    pub bid_qty_1: f64,
    pub bid_price_2: f64,
    pub bid_qty_2: f64,
    pub bid_price_3: f64,
    pub bid_qty_3: f64,
    pub bid_price_4: f64,
    pub bid_qty_4: f64,
    pub bid_price_5: f64,
    pub bid_qty_5: f64,
    pub bid_price_6: f64,
    pub bid_qty_6: f64,
    pub bid_price_7: f64,
    pub bid_qty_7: f64,
    pub bid_price_8: f64,
    pub bid_qty_8: f64,
    pub bid_price_9: f64,
    pub bid_qty_9: f64,
    pub bid_price_10: f64,
    pub bid_qty_10: f64,

    
    pub ask_price_1: f64,
    pub ask_qty_1: f64,
    pub ask_price_2: f64,
    pub ask_qty_2: f64,
    pub ask_price_3: f64,
    pub ask_qty_3: f64,
    pub ask_price_4: f64,
    pub ask_qty_4: f64,
    pub ask_price_5: f64,
    pub ask_qty_5: f64,
    pub ask_price_6: f64,
    pub ask_qty_6: f64,
    pub ask_price_7: f64,
    pub ask_qty_7: f64,
    pub ask_price_8: f64,
    pub ask_qty_8: f64,
    pub ask_price_9: f64,
    pub ask_qty_9: f64,
    pub ask_price_10: f64,
    pub ask_qty_10: f64,
}

impl L2Snapshot {
    
    pub fn bids(&self) -> Vec<PriceLevel> {
        vec![
            PriceLevel::new(self.bid_price_1, self.bid_qty_1),
            PriceLevel::new(self.bid_price_2, self.bid_qty_2),
            PriceLevel::new(self.bid_price_3, self.bid_qty_3),
            PriceLevel::new(self.bid_price_4, self.bid_qty_4),
            PriceLevel::new(self.bid_price_5, self.bid_qty_5),
            PriceLevel::new(self.bid_price_6, self.bid_qty_6),
            PriceLevel::new(self.bid_price_7, self.bid_qty_7),
            PriceLevel::new(self.bid_price_8, self.bid_qty_8),
            PriceLevel::new(self.bid_price_9, self.bid_qty_9),
            PriceLevel::new(self.bid_price_10, self.bid_qty_10),
        ]
    }

    
    pub fn asks(&self) -> Vec<PriceLevel> {
        vec![
            PriceLevel::new(self.ask_price_1, self.ask_qty_1),
            PriceLevel::new(self.ask_price_2, self.ask_qty_2),
            PriceLevel::new(self.ask_price_3, self.ask_qty_3),
            PriceLevel::new(self.ask_price_4, self.ask_qty_4),
            PriceLevel::new(self.ask_price_5, self.ask_qty_5),
            PriceLevel::new(self.ask_price_6, self.ask_qty_6),
            PriceLevel::new(self.ask_price_7, self.ask_qty_7),
            PriceLevel::new(self.ask_price_8, self.ask_qty_8),
            PriceLevel::new(self.ask_price_9, self.ask_qty_9),
            PriceLevel::new(self.ask_price_10, self.ask_qty_10),
        ]
    }

    
    pub fn best_bid(&self) -> f64 {
        self.bid_price_1
    }

    
    pub fn best_ask(&self) -> f64 {
        self.ask_price_1
    }

    
    pub fn spread(&self) -> f64 {
        self.best_ask() - self.best_bid()
    }

    
    pub fn mid_price(&self) -> f64 {
        (self.best_bid() + self.best_ask()) / 2.0
    }

    
    pub fn total_bid_qty(&self) -> f64 {
        self.bids().iter().map(|level| level.quantity).sum()
    }

    
    pub fn total_ask_qty(&self) -> f64 {
        self.asks().iter().map(|level| level.quantity).sum()
    }

    
    pub fn total_bid_notional(&self) -> f64 {
        self.bids().iter().map(|level| level.notional()).sum()
    }

    
    pub fn total_ask_notional(&self) -> f64 {
        self.asks().iter().map(|level| level.notional()).sum()
    }

    
    
    
    
    pub fn imbalance(&self) -> f64 {
        let bid_qty = self.total_bid_qty();
        let ask_qty = self.total_ask_qty();
        (bid_qty - ask_qty) / (bid_qty + ask_qty)
    }

    
    
    
    
    
    
    
    pub fn is_valid(&self) -> bool {
        
        if self.spread() <= 0.0 {
            return false;
        }

        
        let bids = self.bids();
        for i in 0..bids.len() - 1 {
            if bids[i].price < bids[i + 1].price {
                return false;
            }
            if bids[i].quantity < 0.0 {
                return false;
            }
        }

        
        let asks = self.asks();
        for i in 0..asks.len() - 1 {
            if asks[i].price > asks[i + 1].price {
                return false;
            }
            if asks[i].quantity < 0.0 {
                return false;
            }
        }

        true
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
            bid_price_1: 17181.6,
            bid_qty_1: 23.371,
            bid_price_2: 17181.5,
            bid_qty_2: 0.746,
            bid_price_3: 17181.4,
            bid_qty_3: 5.428,
            bid_price_4: 17181.2,
            bid_qty_4: 0.89,
            bid_price_5: 17181.1,
            bid_qty_5: 3.787,
            bid_price_6: 17181.0,
            bid_qty_6: 0.908,
            bid_price_7: 17180.9,
            bid_qty_7: 1.628,
            bid_price_8: 17180.8,
            bid_qty_8: 0.007,
            bid_price_9: 17180.7,
            bid_qty_9: 0.876,
            bid_price_10: 17180.6,
            bid_qty_10: 2.854,
            ask_price_1: 17181.7,
            ask_qty_1: 7.474,
            ask_price_2: 17181.8,
            ask_qty_2: 3.442,
            ask_price_3: 17181.9,
            ask_qty_3: 1.946,
            ask_price_4: 17182.0,
            ask_qty_4: 0.601,
            ask_price_5: 17182.1,
            ask_qty_5: 1.877,
            ask_price_6: 17182.2,
            ask_qty_6: 5.168,
            ask_price_7: 17182.3,
            ask_qty_7: 0.02,
            ask_price_8: 17182.4,
            ask_qty_8: 6.692,
            ask_price_9: 17182.5,
            ask_qty_9: 1.904,
            ask_price_10: 17182.6,
            ask_qty_10: 2.546,
        }
    }

    #[test]
    fn test_best_bid_ask() {
        let snap = create_test_snapshot();
        assert_eq!(snap.best_bid(), 17181.6);
        assert_eq!(snap.best_ask(), 17181.7);
    }

    #[test]
    fn test_spread() {
        let snap = create_test_snapshot();
        assert!((snap.spread() - 0.1).abs() < 0.0001);
    }

    #[test]
    fn test_mid_price() {
        let snap = create_test_snapshot();
        let expected_mid = (17181.6 + 17181.7) / 2.0;
        assert!((snap.mid_price() - expected_mid).abs() < 0.0001);
    }

    #[test]
    fn test_bids_vector() {
        let snap = create_test_snapshot();
        let bids = snap.bids();
        assert_eq!(bids.len(), 10);
        assert_eq!(bids[0].price, 17181.6);
        assert_eq!(bids[0].quantity, 23.371);
    }

    #[test]
    fn test_is_valid() {
        let snap = create_test_snapshot();
        assert!(snap.is_valid());
    }

    #[test]
    fn test_imbalance() {
        let snap = create_test_snapshot();
        let imbalance = snap.imbalance();
        
        assert!(imbalance >= -1.0 && imbalance <= 1.0);
    }
}

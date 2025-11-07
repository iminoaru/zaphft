
pub mod types;
pub mod utils;
pub mod market_data;
pub mod orderbook;
pub mod execution;
pub mod strategy;
pub mod analytics;
pub mod trivial_approach;


pub use types::{L2Snapshot, PriceLevel, Side, Trade};
pub use market_data::{SnapshotReader, SnapshotStats};
pub use orderbook::OrderBook;
pub use execution::{Position, PositionStats};
pub use strategy::{Strategy, StrategyStats};
pub use strategy::market_maker::{MarketMaker, MarketMakerConfig};


pub use trivial_approach::{
    NaiveMarketMaker,
    NaiveMarketMakerConfig,
    NaiveMomentumStrategy,
    PureNaiveMomentumStrategy,
};

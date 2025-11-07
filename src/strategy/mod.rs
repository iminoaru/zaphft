
pub mod market_maker;
pub mod momentum;

use crate::types::{L2Snapshot, Trade};
use crate::execution::Position;




pub trait Strategy {
    
    
    
    fn on_market_data(
        &mut self,
        snapshot: &L2Snapshot,
        position: &Position,
    ) -> Vec<Trade>;

    
    fn name(&self) -> &str;

    
    fn stats(&self) -> StrategyStats;
}


#[derive(Debug, Clone)]
pub struct StrategyStats {
    pub name: String,
    pub updates_processed: usize,
    pub trades_generated: usize,
    pub quotes_placed: usize,
}

impl StrategyStats {
    pub fn print(&self) {
        println!("\n📈 Strategy Statistics: {}", self.name);
        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   Updates Processed: {}", self.updates_processed);
        println!("   Trades Generated:  {}", self.trades_generated);
        println!("   Quotes Placed:     {}", self.quotes_placed);
        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}


use crate::Position;
use std::time::Duration;


#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub name: String,
    pub metrics: PerformanceMetrics,
    pub timing: TimingMetrics,
}


#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    
    pub total_pnl: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,

    
    pub final_position: f64,
    pub max_position_long: f64,
    pub max_position_short: f64,
    pub avg_position: f64,

    
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,

    
    pub total_volume: f64,
    pub buy_volume: f64,
    pub sell_volume: f64,

    
    pub updates_processed: usize,
    pub quotes_placed: usize,
    pub quote_rate: f64,  
}


#[derive(Debug, Clone)]
pub struct TimingMetrics {
    pub total_duration: Duration,
    pub snapshots_processed: usize,
    pub time_per_snapshot: Duration,
    pub throughput: f64,  
}

impl BacktestResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            metrics: PerformanceMetrics::default(),
            timing: TimingMetrics::default(),
        }
    }

    
    pub fn calculate_from_position(
        &mut self,
        position: &Position,
        final_price: f64,
        updates_processed: usize,
        quotes_placed: usize,
    ) {
        
        self.metrics.realized_pnl = position.realized_pnl;
        self.metrics.unrealized_pnl = position.unrealized_pnl(final_price);
        self.metrics.total_pnl = self.metrics.realized_pnl + self.metrics.unrealized_pnl;

        
        self.metrics.final_position = position.quantity;

        
        let mut max_long = 0.0f64;
        let mut max_short = 0.0f64;
        let mut position_sum = 0.0f64;
        let mut current_pos = 0.0f64;

        for trade in position.trades() {
            let signed_qty = match trade.side {
                crate::Side::Bid => trade.quantity,
                crate::Side::Ask => -trade.quantity,
            };
            current_pos += signed_qty;

            max_long = max_long.max(current_pos);
            max_short = max_short.min(current_pos);
            position_sum += current_pos;
        }

        self.metrics.max_position_long = max_long;
        self.metrics.max_position_short = max_short;
        self.metrics.avg_position = if position.trade_count > 0 {
            position_sum / position.trade_count as f64
        } else {
            0.0
        };

        
        self.metrics.total_trades = position.trade_count;

        
        let mut winning = 0;
        let mut losing = 0;
        for trade in position.trades() {
            
            let pnl = match trade.side {
                crate::Side::Bid => {
                    if position.quantity < 0.0 {
                        
                        (position.avg_entry_price - trade.price) * trade.quantity
                    } else {
                        0.0
                    }
                }
                crate::Side::Ask => {
                    if position.quantity > 0.0 {
                        
                        (trade.price - position.avg_entry_price) * trade.quantity
                    } else {
                        0.0
                    }
                }
            };

            if pnl > 0.0 {
                winning += 1;
            } else if pnl < 0.0 {
                losing += 1;
            }
        }

        self.metrics.winning_trades = winning;
        self.metrics.losing_trades = losing;
        self.metrics.win_rate = if self.metrics.total_trades > 0 {
            winning as f64 / self.metrics.total_trades as f64
        } else {
            0.0
        };

        
        self.metrics.total_volume = position.total_bought + position.total_sold;
        self.metrics.buy_volume = position.total_bought;
        self.metrics.sell_volume = position.total_sold;

        
        self.metrics.updates_processed = updates_processed;
        self.metrics.quotes_placed = quotes_placed;
        self.metrics.quote_rate = if updates_processed > 0 {
            quotes_placed as f64 / updates_processed as f64
        } else {
            0.0
        };
    }

    
    pub fn set_timing(&mut self, duration: Duration, snapshots: usize) {
        self.timing.total_duration = duration;
        self.timing.snapshots_processed = snapshots;

        if snapshots > 0 {
            self.timing.time_per_snapshot = duration / snapshots as u32;
            self.timing.throughput = snapshots as f64 / duration.as_secs_f64();
        }
    }

    
    pub fn print_report(&self) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 BACKTEST RESULTS: {}", self.name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("\n💰 PnL Metrics:");
        println!("   Total PnL:           ${:.2}", self.metrics.total_pnl);
        println!("   Realized PnL:        ${:.2}", self.metrics.realized_pnl);
        println!("   Unrealized PnL:      ${:.2}", self.metrics.unrealized_pnl);

        println!("\n📈 Position Metrics:");
        println!("   Final Position:      {:.3} BTC", self.metrics.final_position);
        println!("   Max Long Position:   {:.3} BTC", self.metrics.max_position_long);
        println!("   Max Short Position:  {:.3} BTC", self.metrics.max_position_short);
        println!("   Average Position:    {:.3} BTC", self.metrics.avg_position);

        println!("\n📊 Trade Metrics:");
        println!("   Total Trades:        {}", self.metrics.total_trades);
        println!("   Winning Trades:      {}", self.metrics.winning_trades);
        println!("   Losing Trades:       {}", self.metrics.losing_trades);
        println!("   Win Rate:            {:.1}%", self.metrics.win_rate * 100.0);

        println!("\n📦 Volume Metrics:");
        println!("   Total Volume:        {:.2} BTC", self.metrics.total_volume);
        println!("   Buy Volume:          {:.2} BTC", self.metrics.buy_volume);
        println!("   Sell Volume:         {:.2} BTC", self.metrics.sell_volume);

        println!("\n🎯 Strategy Metrics:");
        println!("   Updates Processed:   {}", self.metrics.updates_processed);
        println!("   Quotes Placed:       {}", self.metrics.quotes_placed);
        println!("   Quote Rate:          {:.1}%", self.metrics.quote_rate * 100.0);

        println!("\n⚡ Performance Metrics:");
        println!("   Total Duration:      {:?}", self.timing.total_duration);
        println!("   Snapshots Processed: {}", self.timing.snapshots_processed);
        println!("   Time per Snapshot:   {:.2} ns", self.timing.time_per_snapshot.as_nanos());
        println!("   Throughput:          {:.0} snapshots/sec", self.timing.throughput);

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_pnl: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            final_position: 0.0,
            max_position_long: 0.0,
            max_position_short: 0.0,
            avg_position: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            total_volume: 0.0,
            buy_volume: 0.0,
            sell_volume: 0.0,
            updates_processed: 0,
            quotes_placed: 0,
            quote_rate: 0.0,
        }
    }
}

impl Default for TimingMetrics {
    fn default() -> Self {
        Self {
            total_duration: Duration::ZERO,
            snapshots_processed: 0,
            time_per_snapshot: Duration::ZERO,
            throughput: 0.0,
        }
    }
}


pub fn print_comparison(results: &[BacktestResult]) {
    if results.is_empty() {
        return;
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 PERFORMANCE COMPARISON");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    
    let best = &results[0];

    println!("{:<30} {:>15} {:>15} {:>15}", "Metric",
             &results[0].name,
             if results.len() > 1 { &results[1].name } else { "" },
             if results.len() > 2 { &results[2].name } else { "" });
    println!("{}", "─".repeat(75));

    
    println!("Total PnL:");
    for result in results {
        println!("  {:<28} ${:>14.2}", result.name, result.metrics.total_pnl);
    }
    println!();

    
    println!("Total Trades:");
    for result in results {
        println!("  {:<28} {:>15}", result.name, result.metrics.total_trades);
    }
    println!();

    
    println!("Time per Snapshot:");
    for result in results {
        println!("  {:<28} {:>12.2} ns", result.name, result.timing.time_per_snapshot.as_nanos());
    }
    println!();

    
    if results.len() > 1 {
        println!("Speedup vs {}:", best.name);
        for result in results.iter().skip(1) {
            let speedup = result.timing.total_duration.as_nanos() as f64
                        / best.timing.total_duration.as_nanos() as f64;
            println!("  {:<28} {:>13.2}× slower", result.name, speedup);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}





use rusthft::{
    SnapshotReader,
    Position, Strategy,
    analytics::BacktestResult,
};
use rusthft::strategy::momentum::{MomentumStrategy, MomentumConfig};
use std::path::Path;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          PROFITABLE MOMENTUM STRATEGY BACKTEST               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    
    let num_snapshots = 200_000;
    let data_path = Path::new("data/L2_processed.csv");

    println!("Test Configuration:");
    println!("   Snapshots:     {}", num_snapshots);
    println!("   Strategy:      Momentum (Conservative Trend Following)");
    println!("   Trigger:       $15 price move");
    println!("   Trade Size:    0.1 BTC");
    println!("   Max Position:  ±1.0 BTC");
    println!("   Lookback:      500 snapshots");
    println!();
    println!("   Fewer, higher-quality trades!");
    println!();

    
    println!("Loading market data...");
    let mut reader = SnapshotReader::new(data_path)?;
    let mut snapshots = Vec::new();

    for _ in 0..num_snapshots {
        match reader.next_snapshot()? {
            Some(snapshot) => snapshots.push(snapshot),
            None => break,
        }
    }

    println!("Loaded {} snapshots\n", snapshots.len());

    
    let first_mid = (snapshots[0].best_bid() + snapshots[0].best_ask()) / 2.0;
    let last_mid = {
        let last = snapshots.last().unwrap();
        (last.best_bid() + last.best_ask()) / 2.0
    };
    let market_change = last_mid - first_mid;
    let market_pct = (market_change / first_mid) * 100.0;

    println!("Market Analysis:");
    println!("   Start Price:   ${:.2}", first_mid);
    println!("   End Price:     ${:.2}", last_mid);
    println!("   Change:        ${:.2} ({:+.2}%)", market_change, market_pct);
    println!();

    if market_change > 0.0 {
        println!("   Bullish market - Momentum strategy should profit!");
    } else {
        println!("   WARNING: Bearish market - Momentum may struggle");
    }
    println!();

    
    
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("RUNNING MOMENTUM STRATEGY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config = MomentumConfig {
        trigger_threshold: 15.0,  
        trade_size: 0.1,
        max_position: 1.0,        
        lookback: 500,            
    };

    let mut strategy = MomentumStrategy::new(config);
    let mut position = Position::new();

    println!("Running backtest...");
    let start = Instant::now();

    for snapshot in &snapshots {
        let trades = strategy.on_market_data(snapshot, &position);
        for trade in trades {
            position.execute_trade(trade);
        }
    }

    let duration = start.elapsed();
    println!("Completed in {:?}\n", duration);

    
    let final_price = last_mid;
    let stats = strategy.stats();
    let mut result = BacktestResult::new("Momentum Strategy".to_string());
    result.calculate_from_position(&position, final_price, stats.updates_processed, stats.quotes_placed);
    result.set_timing(duration, snapshots.len());
    result.print_report();

    
    
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("PERFORMANCE ANALYSIS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let pnl = result.metrics.total_pnl;
    let trades = result.metrics.total_trades;

    if trades == 0 {
        println!("WARNING: No trades generated!");
        println!("   Market may not have moved enough ($5 threshold).");
        println!("   Try reducing trigger_threshold to see more signals.");
    } else {
        println!("Generated {} trades", trades);
        println!();

        if pnl > 0.0 {
            println!("PROFITABLE STRATEGY!");
            println!("   Total PnL:        ${:.2}", pnl);
            println!("   PnL per trade:    ${:.2}", pnl / trades as f64);
            println!("   Market capture:   {:.1}%", (pnl / market_change) * 100.0);
            println!();
            println!("   Strategy successfully captured the uptrend!");
        } else {
            println!("📉 Strategy lost money");
            println!("   Total PnL:        ${:.2}", pnl);
            println!("   PnL per trade:    ${:.2}", pnl / trades as f64);
            println!();

            if market_change > 0.0 {
                println!("   Market went up but strategy lost - needs tuning");
            } else {
                println!("   Momentum struggles in choppy/down markets");
            }
        }
    }

    println!();
    println!("Final Position:     {:.3} BTC", result.metrics.final_position);
    println!("Max Long:           {:.3} BTC", result.metrics.max_position_long);
    println!("Max Short:          {:.3} BTC", result.metrics.max_position_short);
    println!("Total Volume:       {:.2} BTC", result.metrics.total_volume);
    println!();

    
    let buy_hold_pnl = market_change * 0.1;  
    println!("Comparison:");
    println!("   Strategy PnL:     ${:.2}", pnl);
    println!("   Buy & Hold:       ${:.2} (0.1 BTC)", buy_hold_pnl);

    if pnl > buy_hold_pnl {
        println!("   Strategy outperformed buy & hold!");
    } else if pnl > 0.0 {
        println!("   Strategy profitable but underperformed buy & hold");
    } else {
        println!("   Strategy lost money");
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

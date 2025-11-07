







use rusthft::{
    SnapshotReader,
    Position,
    Strategy, MarketMaker, MarketMakerConfig,
    NaiveMarketMaker, NaiveMarketMakerConfig,
};
use std::time::Instant;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("📊 Market Making Strategy Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    
    let num_snapshots = 10_000;
    println!("Configuration:");
    println!("  • Processing {} snapshots", num_snapshots);
    println!("  • Spread: 1 tick ($0.10)");
    println!("  • Quote size: 0.1 BTC");
    println!("  • Max position: ±2.0 BTC");
    println!();

    
    let config = MarketMakerConfig {
        spread_ticks: 1.0,
        quote_size: 0.1,
        max_position: 2.0,
        tick_size: 0.1,
        ..MarketMakerConfig::default()
    };
    let mut mm = MarketMaker::new(config.clone());
    let mut position = Position::new();

    
    let naive_config = NaiveMarketMakerConfig {
        spread_ticks: 1.0,
        quote_size: 0.1,
        max_position: 2.0,
        tick_size: 0.1,
    };
    let mut naive_mm = NaiveMarketMaker::new(naive_config);
    let mut naive_position = Position::new();

    
    println!("Loading market data...");
    let mut reader = SnapshotReader::new(Path::new("data/L2_processed.csv"))?;

    
    reader.next_snapshot()?;

    
    let mut snapshots = Vec::new();
    for _ in 0..num_snapshots {
        if let Some(snapshot) = reader.next_snapshot()? {
            snapshots.push(snapshot);
        } else {
            break;
        }
    }

    println!("✓ Loaded {} snapshots\n", snapshots.len());

    
    
    
    println!("Running Optimized Market Maker...");
    let start = Instant::now();

    for snapshot in &snapshots {
        let trades = mm.on_market_data(snapshot, &position);
        for trade in trades {
            position.execute_trade(trade);
        }
    }

    let opt_duration = start.elapsed();
    let opt_stats = mm.stats();

    
    let last_snapshot = snapshots.last().unwrap();
    let final_price = (last_snapshot.best_bid() + last_snapshot.best_ask()) / 2.0;
    let opt_unrealized = position.unrealized_pnl(final_price);
    let opt_total_pnl = position.realized_pnl + opt_unrealized;

    println!("✓ Completed in {:?}", opt_duration);
    println!();

    
    
    
    println!("Running Naive Market Maker (cached with HashMap)...");
    let start = Instant::now();

    for snapshot in &snapshots {
        let trades = naive_mm.on_market_data(snapshot, &naive_position);
        for trade in trades {
            naive_position.execute_trade(trade);
        }
    }

    let naive_duration = start.elapsed();
    let naive_stats = naive_mm.stats();

    let naive_unrealized = naive_position.unrealized_pnl(final_price);
    let naive_total_pnl = naive_position.realized_pnl + naive_unrealized;

    println!("✓ Completed in {:?}", naive_duration);
    println!();

    
    
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 OPTIMIZED MARKET MAKER RESULTS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Strategy Statistics:");
    println!("  • Updates processed:  {}", opt_stats.updates_processed);
    println!("  • Trades generated:   {}", opt_stats.trades_generated);
    println!("  • Quotes placed:      {}", opt_stats.quotes_placed);
    println!();
    println!("Position & PnL:");
    println!("  • Final position:     {:.3} BTC", position.quantity);
    println!("  • Avg entry price:    ${:.2}", position.avg_entry_price);
    println!("  • Realized PnL:       ${:.2}", position.realized_pnl);
    println!("  • Unrealized PnL:     ${:.2}", opt_unrealized);
    println!("  • Total PnL:          ${:.2}", opt_total_pnl);
    println!();
    println!("Performance:");
    println!("  • Total time:         {:?}", opt_duration);
    println!("  • Time per update:    {:.2} ns", opt_duration.as_nanos() as f64 / snapshots.len() as f64);
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📉 NAIVE MARKET MAKER RESULTS (HashMap)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Strategy Statistics:");
    println!("  • Updates processed:  {}", naive_stats.updates_processed);
    println!("  • Trades generated:   {}", naive_stats.trades_generated);
    println!("  • Quotes placed:      {}", naive_stats.quotes_placed);
    println!();
    println!("Position & PnL:");
    println!("  • Final position:     {:.3} BTC", naive_position.quantity);
    println!("  • Avg entry price:    ${:.2}", naive_position.avg_entry_price);
    println!("  • Realized PnL:       ${:.2}", naive_position.realized_pnl);
    println!("  • Unrealized PnL:     ${:.2}", naive_unrealized);
    println!("  • Total PnL:          ${:.2}", naive_total_pnl);
    println!();
    println!("Performance:");
    println!("  • Total time:         {:?}", naive_duration);
    println!("  • Time per update:    {:.2} ns", naive_duration.as_nanos() as f64 / snapshots.len() as f64);
    println!();

    
    
    
    let speedup = naive_duration.as_nanos() as f64 / opt_duration.as_nanos() as f64;
    let opt_ns_per_update = opt_duration.as_nanos() as f64 / snapshots.len() as f64;
    let naive_ns_per_update = naive_duration.as_nanos() as f64 / snapshots.len() as f64;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚡ PERFORMANCE COMPARISON");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Optimized:  {:.2} ns per update", opt_ns_per_update);
    println!("Naive:      {:.2} ns per update", naive_ns_per_update);
    println!();
    println!("🚀 Speedup: {:.2}× faster", speedup);
    println!();

    
    println!("Why the difference?");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Optimized:   Direct field access (config.spread_ticks)");
    println!("Naive:       HashMap lookups (config.get(\"spread_ticks\"))");
    println!();
    println!("HashMap overhead:");
    println!("  • Hash function computation");
    println!("  • Bucket lookup");
    println!("  • String key comparison");
    println!("  • Pointer indirection");
    println!();
    println!("Even with caching, data structure choice matters!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    
    println!();
    println!("Verification:");
    let trades_match = opt_stats.trades_generated == naive_stats.trades_generated;
    let quotes_match = opt_stats.quotes_placed == naive_stats.quotes_placed;
    let position_match = (position.quantity - naive_position.quantity).abs() < 1e-6;
    let pnl_match = (opt_total_pnl - naive_total_pnl).abs() < 1e-2;

    println!("  • Trades match:    {} {}", if trades_match { "✓" } else { "✗" },
             if !trades_match { format!("({} vs {})", opt_stats.trades_generated, naive_stats.trades_generated) } else { String::new() });
    println!("  • Quotes match:    {} {}", if quotes_match { "✓" } else { "✗" },
             if !quotes_match { format!("({} vs {})", opt_stats.quotes_placed, naive_stats.quotes_placed) } else { String::new() });
    println!("  • Position match:  {} {}", if position_match { "✓" } else { "✗" },
             if !position_match { format!("({:.3} vs {:.3})", position.quantity, naive_position.quantity) } else { String::new() });
    println!("  • PnL match:       {} {}", if pnl_match { "✓" } else { "✗" },
             if !pnl_match { format!("(${:.2} vs ${:.2})", opt_total_pnl, naive_total_pnl) } else { String::new() });

    if trades_match && quotes_match && position_match && pnl_match {
        println!();
        println!("✅ Both strategies produce identical results!");
    } else {
        println!();
        println!("⚠️  Results differ - check implementation");
    }

    Ok(())
}

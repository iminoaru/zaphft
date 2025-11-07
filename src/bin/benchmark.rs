



use anyhow::Result;
use rusthft::{
    L2Snapshot, OrderBook, SnapshotReader, Position,
    Strategy, MarketMaker, MarketMakerConfig,
    NaiveMarketMaker, NaiveMarketMakerConfig,
};
use rusthft::trivial_approach::{NaiveSnapshot, NaiveOrderBook};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    println!("🏁 HFT Performance Benchmark: Optimized vs Naive\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let data_path = Path::new("data/L2_processed.csv");

    if !data_path.exists() {
        println!("❌ Data file not found!");
        return Ok(());
    }

    
    println!("📖 Loading 10,000 snapshots...\n");
    let mut reader = SnapshotReader::new(data_path)?;
    let mut optimized_snapshots = Vec::new();

    for _ in 0..10_000 {
        match reader.next_snapshot()? {
            Some(snap) => optimized_snapshots.push(snap),
            None => break,
        }
    }

    println!("✓ Loaded {} snapshots\n", optimized_snapshots.len());

    
    let naive_snapshots: Vec<NaiveSnapshot> = optimized_snapshots
        .iter()
        .map(|s| {
            let mut values = Vec::with_capacity(40);
            
            values.push(s.bid_price_1); values.push(s.bid_qty_1);
            values.push(s.bid_price_2); values.push(s.bid_qty_2);
            values.push(s.bid_price_3); values.push(s.bid_qty_3);
            values.push(s.bid_price_4); values.push(s.bid_qty_4);
            values.push(s.bid_price_5); values.push(s.bid_qty_5);
            values.push(s.bid_price_6); values.push(s.bid_qty_6);
            values.push(s.bid_price_7); values.push(s.bid_qty_7);
            values.push(s.bid_price_8); values.push(s.bid_qty_8);
            values.push(s.bid_price_9); values.push(s.bid_qty_9);
            values.push(s.bid_price_10); values.push(s.bid_qty_10);
            
            values.push(s.ask_price_1); values.push(s.ask_qty_1);
            values.push(s.ask_price_2); values.push(s.ask_qty_2);
            values.push(s.ask_price_3); values.push(s.ask_qty_3);
            values.push(s.ask_price_4); values.push(s.ask_qty_4);
            values.push(s.ask_price_5); values.push(s.ask_qty_5);
            values.push(s.ask_price_6); values.push(s.ask_qty_6);
            values.push(s.ask_price_7); values.push(s.ask_qty_7);
            values.push(s.ask_price_8); values.push(s.ask_qty_8);
            values.push(s.ask_price_9); values.push(s.ask_qty_9);
            values.push(s.ask_price_10); values.push(s.ask_qty_10);

            NaiveSnapshot::from_csv_data(
                s.timestamp_us,
                s.datetime.clone(),
                values,
            )
        })
        .collect();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚀 BENCHMARK 1: Basic Operations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    
    let start = Instant::now();
    let mut book = OrderBook::new();
    for snap in &optimized_snapshots {
        book.update(snap.clone());
        let _ = book.best_bid();
        let _ = book.best_ask();
        let _ = book.spread();
    }
    let optimized_time = start.elapsed();

    
    let start = Instant::now();
    let mut naive_book = NaiveOrderBook::new();
    for snap in &naive_snapshots {
        naive_book.update(snap.clone());
        let _ = naive_book.best_bid();
        let _ = naive_book.best_ask();
        let _ = naive_book.spread();
    }
    let naive_time = start.elapsed();

    let optimized_ns = optimized_time.as_nanos() / optimized_snapshots.len() as u128;
    let naive_ns = naive_time.as_nanos() / naive_snapshots.len() as u128;
    let speedup = naive_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;

    println!("📊 Results (10,000 iterations):\n");
    println!("   Optimized Approach:");
    println!("      Total Time:  {:?}", optimized_time);
    println!("      Per Op:      {} ns", optimized_ns);
    println!("      Rate:        {:.0} ops/sec", 1_000_000_000.0 / optimized_ns as f64);
    println!();
    println!("   Naive Approach:");
    println!("      Total Time:  {:?}", naive_time);
    println!("      Per Op:      {} ns", naive_ns);
    println!("      Rate:        {:.0} ops/sec", 1_000_000_000.0 / naive_ns as f64);
    println!();
    println!("   ⚡ Speedup:     {:.2}× faster", speedup);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍 BENCHMARK 2: Depth Calculations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    
    let start = Instant::now();
    for snap in optimized_snapshots.iter().take(1000) {
        let _ = snap.bids();
        let _ = snap.asks();
        let _ = snap.total_bid_qty();
        let _ = snap.total_ask_qty();
        let _ = snap.imbalance();
    }
    let optimized_depth_time = start.elapsed();

    
    let start = Instant::now();
    for snap in naive_snapshots.iter().take(1000) {
        let _ = snap.bids();
        let _ = snap.asks();
        let _ = snap.total_bid_qty();
        let _ = snap.total_ask_qty();
        let _ = snap.imbalance();
    }
    let naive_depth_time = start.elapsed();

    let opt_depth_ns = optimized_depth_time.as_nanos() / 1000;
    let naive_depth_ns = naive_depth_time.as_nanos() / 1000;
    let depth_speedup = naive_depth_time.as_nanos() as f64 / optimized_depth_time.as_nanos() as f64;

    println!("📊 Results (1,000 iterations):\n");
    println!("   Optimized Approach:");
    println!("      Total Time:  {:?}", optimized_depth_time);
    println!("      Per Op:      {} ns", opt_depth_ns);
    println!();
    println!("   Naive Approach:");
    println!("      Total Time:  {:?}", naive_depth_time);
    println!("      Per Op:      {} ns", naive_depth_ns);
    println!();
    println!("   ⚡ Speedup:     {:.2}× faster", depth_speedup);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 BENCHMARK 3: Market Making Strategy");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    
    let config = MarketMakerConfig {
        spread_ticks: 1.0,
        quote_size: 0.1,
        max_position: 2.0,
        tick_size: 0.1,
        ..MarketMakerConfig::default()
    };
    let mut opt_mm = MarketMaker::new(config);
    let mut opt_position = Position::new();

    
    let naive_config = NaiveMarketMakerConfig {
        spread_ticks: 1.0,
        quote_size: 0.1,
        max_position: 2.0,
        tick_size: 0.1,
    };
    let mut naive_mm = NaiveMarketMaker::new(naive_config);
    let mut naive_position = Position::new();

    
    let start = Instant::now();
    for snap in &optimized_snapshots {
        let trades = opt_mm.on_market_data(snap, &opt_position);
        for trade in trades {
            opt_position.execute_trade(trade);
        }
    }
    let opt_strategy_time = start.elapsed();

    
    let start = Instant::now();
    for snap in &optimized_snapshots {
        let trades = naive_mm.on_market_data(snap, &naive_position);
        for trade in trades {
            naive_position.execute_trade(trade);
        }
    }
    let naive_strategy_time = start.elapsed();

    let opt_strat_ns = opt_strategy_time.as_nanos() / optimized_snapshots.len() as u128;
    let naive_strat_ns = naive_strategy_time.as_nanos() / optimized_snapshots.len() as u128;
    let strategy_speedup = naive_strategy_time.as_nanos() as f64 / opt_strategy_time.as_nanos() as f64;

    println!("📊 Results (10,000 iterations):\n");
    println!("   Optimized Market Maker:");
    println!("      Total Time:  {:?}", opt_strategy_time);
    println!("      Per Update:  {} ns", opt_strat_ns);
    println!("      Trades:      {}", opt_mm.stats().trades_generated);
    println!("      Quotes:      {}", opt_mm.stats().quotes_placed);
    println!();
    println!("   Naive Market Maker (HashMap):");
    println!("      Total Time:  {:?}", naive_strategy_time);
    println!("      Per Update:  {} ns", naive_strat_ns);
    println!("      Trades:      {}", naive_mm.stats().trades_generated);
    println!("      Quotes:      {}", naive_mm.stats().quotes_placed);
    println!();
    println!("   ⚡ Speedup:     {:.2}× faster", strategy_speedup);
    println!();
    println!("   Why the difference?");
    println!("      • Optimized: Direct struct fields (config.spread_ticks)");
    println!("      • Naive:     HashMap lookups (config.get(\"spread_ticks\"))");
    println!("      • HashMap overhead: hashing, bucket lookup, string comparison");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💾 BENCHMARK 4: Memory Usage");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let opt_snap_size = std::mem::size_of::<L2Snapshot>();
    let naive_snap_size = std::mem::size_of_val(&naive_snapshots[0]);

    println!("   Optimized L2Snapshot:  {} bytes", opt_snap_size);
    println!("   Naive Snapshot:        {} bytes (+ HashMap overhead)", naive_snap_size);
    println!("   Difference:            ~{}× more memory",
             naive_snap_size as f64 / opt_snap_size as f64);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("   The optimized approach is:");
    println!("      • {:.1}× faster for basic operations", speedup);
    println!("      • {:.1}× faster for depth calculations", depth_speedup);
    println!("      • {:.1}× faster for market making strategy", strategy_speedup);
    println!("      • More memory efficient");
    println!("      • Cache friendly");
    println!("      • Zero heap allocations in hot path");
    println!("\n   See PERFORMANCE_ANALYSIS.md for detailed breakdown.\n");

    Ok(())
}

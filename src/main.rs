use anyhow::Result;
use rusthft::{OrderBook, Side, SnapshotReader, SnapshotStats};
use std::path::Path;

fn main() -> Result<()> {
    println!("🚀 HFT Backtesting Engine - Data Analysis Demo\n");

    let data_path = Path::new("data/L2_processed.csv");

    
    if !data_path.exists() {
        println!("❌ Processed data not found!");
        println!("   Run preprocessing first: cargo run --bin preprocess");
        return Ok(());
    }

    println!("📖 Reading snapshots from: {}\n", data_path.display());

    
    let mut reader = SnapshotReader::new(data_path)?;
    let mut snapshots = Vec::new();

    println!("Loading first 10,000 snapshots...");
    for _ in 0..10_000 {
        match reader.next_snapshot()? {
            Some(snap) => snapshots.push(snap),
            None => break,
        }
    }

    println!("✓ Loaded {} snapshots\n", snapshots.len());

    
    let stats = SnapshotStats::from_snapshots(&snapshots);
    stats.print();

    
    println!("\n📚 Order Book Analysis\n");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut book = OrderBook::new();
    book.update(snapshots[0].clone());

    println!("   Snapshot #1:");
    println!("   Best Bid:    ${:.2}", book.best_bid().unwrap());
    println!("   Best Ask:    ${:.2}", book.best_ask().unwrap());
    println!("   Spread:      ${:.4}", book.spread().unwrap());
    println!("   Mid Price:   ${:.2}", book.mid_price().unwrap());

    
    println!("\n   💧 Slippage Analysis (buying):");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for qty in [1.0, 5.0, 10.0, 20.0] {
        if let Some((avg_price, slippage_bps, levels)) = book.calculate_slippage(Side::Ask, qty) {
            println!(
                "   Buy {:.1} BTC → Avg: ${:.2}, Slippage: {:.2} bps, Levels: {}",
                qty, avg_price, slippage_bps, levels
            );
        }
    }

    
    println!("\n   📊 Order Book Depth (Top 5 Levels):");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let bids = book.bids();
    let asks = book.asks();

    println!("   {:>10} | {:>8} | {:>8} | {:>10}", "BID QTY", "BID", "ASK", "ASK QTY");
    println!("   ───────────────────────────────────────────");

    for i in 0..5 {
        println!(
            "   {:>10.3} | {:>8.2} | {:>8.2} | {:>10.3}",
            bids[i].quantity,
            bids[i].price,
            asks[i].price,
            asks[i].quantity
        );
    }

    
    println!("\n   ⚖️  Order Book Imbalance (first 100 snapshots):");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut positive_count = 0;
    let mut negative_count = 0;

    for snap in snapshots.iter().take(100) {
        let imbalance = snap.imbalance();
        if imbalance > 0.0 {
            positive_count += 1;
        } else {
            negative_count += 1;
        }
    }

    println!("   More Bids:  {} snapshots ({:.1}%)", positive_count, positive_count as f64);
    println!("   More Asks:  {} snapshots ({:.1}%)", negative_count, negative_count as f64);

    
    println!("\n   ⚡ Performance Test:");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    use std::time::Instant;
    let start = Instant::now();
    let mut book = OrderBook::new();

    for snap in &snapshots {
        book.update(snap.clone());
        let _ = book.best_bid();
        let _ = book.best_ask();
        let _ = book.spread();
    }

    let elapsed = start.elapsed();
    let snapshots_per_sec = snapshots.len() as f64 / elapsed.as_secs_f64();
    let ns_per_snapshot = elapsed.as_nanos() / snapshots.len() as u128;

    println!("   Processed: {} snapshots", snapshots.len());
    println!("   Time:      {:.2?}", elapsed);
    println!("   Rate:      {:.0} snapshots/sec", snapshots_per_sec);
    println!("   Latency:   {} ns/snapshot", ns_per_snapshot);

    println!("\n✅ Demo complete!\n");

    Ok(())
}

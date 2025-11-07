



use rusthft::SnapshotReader;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("📊 Market Movement Analysis\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let num_snapshots = 200_000;
    let data_path = Path::new("data/L2_processed.csv");

    println!("Loading {} snapshots...", num_snapshots);
    let mut reader = SnapshotReader::new(data_path)?;
    let mut snapshots = Vec::new();

    for _ in 0..num_snapshots {
        match reader.next_snapshot()? {
            Some(snapshot) => snapshots.push(snapshot),
            None => break,
        }
    }

    println!("✓ Loaded {} snapshots\n", snapshots.len());

    
    let mut min_bid = f64::MAX;
    let mut max_bid = f64::MIN;
    let mut min_ask = f64::MAX;
    let mut max_ask = f64::MIN;

    let mut spreads = Vec::new();
    let mut price_changes = Vec::new();

    let first_mid = (snapshots[0].best_bid() + snapshots[0].best_ask()) / 2.0;
    let mut prev_mid = first_mid;

    for snapshot in &snapshots {
        let bid = snapshot.best_bid();
        let ask = snapshot.best_ask();
        let mid = (bid + ask) / 2.0;
        let spread = ask - bid;

        min_bid = min_bid.min(bid);
        max_bid = max_bid.max(bid);
        min_ask = min_ask.min(ask);
        max_ask = max_ask.max(ask);

        spreads.push(spread);

        let change = (mid - prev_mid).abs();
        if change > 0.01 {  
            price_changes.push(change);
        }
        prev_mid = mid;
    }

    let last_mid = (snapshots.last().unwrap().best_bid() + snapshots.last().unwrap().best_ask()) / 2.0;
    let total_change = last_mid - first_mid;

    
    let avg_spread = spreads.iter().sum::<f64>() / spreads.len() as f64;
    let avg_change = if !price_changes.is_empty() {
        price_changes.iter().sum::<f64>() / price_changes.len() as f64
    } else {
        0.0
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 PRICE RANGE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Best Bid:   ${:.2} to ${:.2}", min_bid, max_bid);
    println!("Best Ask:   ${:.2} to ${:.2}", min_ask, max_ask);
    println!("Range:      ${:.2}", max_ask - min_bid);
    println!();
    println!("Starting Mid: ${:.2}", first_mid);
    println!("Ending Mid:   ${:.2}", last_mid);
    println!("Total Change: ${:.2} ({:.2}%)", total_change, (total_change / first_mid) * 100.0);
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 SPREAD ANALYSIS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Average Spread:     ${:.2}", avg_spread);
    println!("Min Spread:         ${:.2}", spreads.iter().cloned().fold(f64::INFINITY, f64::min));
    println!("Max Spread:         ${:.2}", spreads.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    println!();

    
    println!("Our Strategy:");
    println!("  Spread Ticks:     1.0");
    println!("  Tick Size:        $0.10");
    println!("  Our Quotes Away:  $0.10 from best bid/ask");
    println!();
    println!("For fills, market needs to move > $0.10");
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📉 VOLATILITY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Price Changes:      {} significant moves", price_changes.len());
    println!("Avg Change:         ${:.2}", avg_change);
    if !price_changes.is_empty() {
        println!("Max Change:         ${:.2}", price_changes.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    }
    println!();

    
    let big_moves = price_changes.iter().filter(|&&x| x >= 0.10).count();
    println!("Moves > $0.10:      {} times", big_moves);
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 ANALYSIS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if big_moves == 0 {
        println!("❌ No moves > $0.10 detected!");
        println!("   This is why we got 0 fills.");
        println!();
        println!("Solutions:");
        println!("  1. Reduce spread_ticks to 0.5 ($0.05 away)");
        println!("  2. Set spread_ticks to 0.0 (quote AT best)");
        println!("  3. Set spread_ticks to -0.5 (CROSS the spread)");
    } else {
        println!("✓ Found {} moves > $0.10", big_moves);
        println!("  We SHOULD have gotten fills...");
        println!("  Strategy logic might need adjustment.");
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

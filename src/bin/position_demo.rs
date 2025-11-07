








use anyhow::Result;
use rusthft::{Position, Side, Trade};
use rusthft::trivial_approach::{NaivePosition, CachedNaivePosition};
use std::time::Instant;

fn main() -> Result<()> {
    println!("💼 Position Tracking & PnL Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    
    println!("📊 Example 1: Simple Long Position");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut pos = Position::new();

    println!("   Buy 1 BTC at $17,181.60");
    pos.execute_trade(Trade::new(Side::Bid, 17181.60, 1.0, 0));

    println!("   Position: {:.4} BTC", pos.quantity);
    println!("   Entry Price: ${:.2}", pos.avg_entry_price);
    println!("   Realized PnL: ${:.2}", pos.realized_pnl);

    println!("\n   Market moves to $17,200.00");
    let unrealized = pos.unrealized_pnl(17200.00);
    println!("   Unrealized PnL: ${:.2} ✅", unrealized);

    println!("\n   Sell 1 BTC at $17,200.00 (close position)");
    pos.execute_trade(Trade::new(Side::Ask, 17200.00, 1.0, 1));

    println!("   Position: {:.4} BTC (flat)", pos.quantity);
    println!("   Realized PnL: ${:.2} ✅", pos.realized_pnl);

    
    println!("\n\n📊 Example 2: Averaging Down");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut pos = Position::new();

    println!("   Buy 1 BTC at $17,200.00");
    pos.execute_trade(Trade::new(Side::Bid, 17200.00, 1.0, 0));
    println!("   Entry Price: ${:.2}", pos.avg_entry_price);

    println!("\n   Market drops to $17,100.00");
    println!("   Unrealized PnL: ${:.2} ❌", pos.unrealized_pnl(17100.00));

    println!("\n   Buy 1 more BTC at $17,100.00 (averaging down)");
    pos.execute_trade(Trade::new(Side::Bid, 17100.00, 1.0, 1));
    println!("   Position: {:.4} BTC", pos.quantity);
    println!("   Average Entry Price: ${:.2}", pos.avg_entry_price);

    println!("\n   Market recovers to $17,150.00");
    println!("   Unrealized PnL: ${:.2} ✅", pos.unrealized_pnl(17150.00));

    
    println!("\n\n📊 Example 3: Position Flip (Long → Short)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut pos = Position::new();

    println!("   Buy 1 BTC at $17,200.00 (open long)");
    pos.execute_trade(Trade::new(Side::Bid, 17200.00, 1.0, 0));
    println!("   Position: {:.4} BTC (long)", pos.quantity);

    println!("\n   Sell 2 BTC at $17,250.00 (close long + open short)");
    pos.execute_trade(Trade::new(Side::Ask, 17250.00, 2.0, 1));
    println!("   Position: {:.4} BTC (short)", pos.quantity);
    println!("   Realized PnL: ${:.2} (from closed long)", pos.realized_pnl);
    println!("   New Entry Price: ${:.2}", pos.avg_entry_price);

    println!("\n   Market moves to $17,220.00");
    println!("   Unrealized PnL: ${:.2} ✅ (profit on short)", pos.unrealized_pnl(17220.00));

    
    println!("\n\n📊 Example 4: Partial Position Close");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut pos = Position::new();

    println!("   Buy 3 BTC at $17,180.00");
    pos.execute_trade(Trade::new(Side::Bid, 17180.00, 3.0, 0));
    println!("   Position: {:.4} BTC", pos.quantity);

    println!("\n   Sell 1 BTC at $17,200.00");
    pos.execute_trade(Trade::new(Side::Ask, 17200.00, 1.0, 1));
    println!("   Position: {:.4} BTC", pos.quantity);
    println!("   Realized PnL: ${:.2}", pos.realized_pnl);

    println!("\n   Sell 1 more BTC at $17,220.00");
    pos.execute_trade(Trade::new(Side::Ask, 17220.00, 1.0, 2));
    println!("   Position: {:.4} BTC", pos.quantity);
    println!("   Realized PnL: ${:.2}", pos.realized_pnl);

    println!("\n   Sell final 1 BTC at $17,250.00");
    pos.execute_trade(Trade::new(Side::Ask, 17250.00, 1.0, 3));
    println!("   Position: {:.4} BTC (flat)", pos.quantity);
    println!("   Total Realized PnL: ${:.2} ✅", pos.realized_pnl);

    
    println!("\n\n⚡ Performance Comparison: THREE Approaches");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let num_trades = 10_000;
    let mut trades = Vec::new();

    
    for i in 0..num_trades {
        let side = if i % 2 == 0 { Side::Bid } else { Side::Ask };
        let price = 17000.0 + (i % 100) as f64;
        let qty = 0.1;
        trades.push(Trade::new(side, price, qty, i as u64));
    }

    
    println!("   [1/3] Testing Optimized (cache + direct fields)...");
    let start = Instant::now();
    let mut opt_pos = Position::new();
    for trade in &trades {
        opt_pos.execute_trade(trade.clone());
        let _ = opt_pos.quantity;
        let _ = opt_pos.avg_entry_price;
        let _ = opt_pos.unrealized_pnl(17100.0);
    }
    let opt_time = start.elapsed();

    
    println!("   [2/3] Testing Cached Naive (cache + HashMap)...");
    let start = Instant::now();
    let mut cached_naive_pos = CachedNaivePosition::new();
    for trade in &trades {
        cached_naive_pos.execute_trade(trade.clone());
        let _ = cached_naive_pos.quantity();      
        let _ = cached_naive_pos.avg_entry_price(); 
        let _ = cached_naive_pos.unrealized_pnl(17100.0);
    }
    let cached_naive_time = start.elapsed();

    
    println!("   [3/3] Testing Pure Naive (no cache, recalculates)...");
    let start = Instant::now();
    let mut naive_pos = NaivePosition::new();
    for trade in &trades {
        naive_pos.execute_trade(trade.clone());
        let _ = naive_pos.quantity();           
        let _ = naive_pos.avg_entry_price();    
        let _ = naive_pos.unrealized_pnl(17100.0);  
    }
    let naive_time = start.elapsed();

    let speedup_vs_cached = cached_naive_time.as_nanos() as f64 / opt_time.as_nanos() as f64;
    let speedup_vs_naive = naive_time.as_nanos() as f64 / opt_time.as_nanos() as f64;
    let cache_benefit = naive_time.as_nanos() as f64 / cached_naive_time.as_nanos() as f64;

    println!("\n   Results ({} trades):", num_trades);
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   Optimized (cache + direct):  {:?} ({} ns/trade)",
             opt_time, opt_time.as_nanos() / num_trades);
    println!("   Cached Naive (cache + Hash): {:?} ({} ns/trade)",
             cached_naive_time, cached_naive_time.as_nanos() / num_trades);
    println!("   Pure Naive (no cache):       {:?} ({} ns/trade)",
             naive_time, naive_time.as_nanos() / num_trades);
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   Speedup vs Cached Naive:  {:.2}× faster", speedup_vs_cached);
    println!("   Speedup vs Pure Naive:    {:.2}× faster", speedup_vs_naive);
    println!("   Caching benefit alone:    {:.2}× faster", cache_benefit);

    
    println!("\n   Correctness Check:");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   Optimized:      {:.4} BTC", opt_pos.quantity);
    println!("   Cached Naive:   {:.4} BTC", cached_naive_pos.quantity());
    println!("   Pure Naive:     {:.4} BTC", naive_pos.quantity());
    let diff1 = (opt_pos.quantity - cached_naive_pos.quantity()).abs();
    let diff2 = (opt_pos.quantity - naive_pos.quantity()).abs();
    if diff1 < 1e-6 && diff2 < 1e-6 {
        println!("   ✅ All results match!");
    } else {
        println!("   ❌ Results differ!");
    }

    
    let stats = opt_pos.stats(17100.0);
    stats.print();

    println!("\n💡 Key Insights:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   • Caching provides {:.0}× speedup", cache_benefit);
    println!("   • BUT even with caching, HashMap is {:.1}× slower than direct fields", speedup_vs_cached);
    println!("   • Our speedup comes from MULTIPLE optimizations:");
    println!("     1. Caching (avoid recalculation)");
    println!("     2. Direct fields (avoid HashMap lookups)");
    println!("     3. Cache-friendly layout (sequential memory)");
    println!("   • Total speedup: {:.0}× faster than naive", speedup_vs_naive);
    println!("   • This proves it's NOT \"just caching\" - data structure matters!\n");

    Ok(())
}

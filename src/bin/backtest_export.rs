




use rusthft::*;
use rusthft::analytics::{BacktestResult, BacktestExport, TimeseriesData, TimeseriesPoint, PerformanceComparison};
use rusthft::strategy::momentum::{MomentumStrategy, MomentumConfig};
use rusthft::trivial_approach::{NaiveMomentumStrategy, PureNaiveMomentumStrategy};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args)?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           BACKTEST EXPORT - DATA FOR VISUALIZATION          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("📋 Export Configuration:");
    println!("   Strategy:      {:?}", config.strategy);
    println!("   Snapshots:     {}", config.num_snapshots);
    println!("   Output:        {}", config.output_path.display());
    println!();

    
    println!("📖 Loading market data...");
    let data_path = Path::new("data/L2_processed.csv");
    let mut reader = SnapshotReader::new(data_path)?;
    let mut snapshots = Vec::new();

    for _ in 0..config.num_snapshots {
        match reader.next_snapshot()? {
            Some(snapshot) => snapshots.push(snapshot),
            None => break,
        }
    }

    println!("✓ Loaded {} snapshots\n", snapshots.len());

    
    let start_price = (snapshots[0].best_bid() + snapshots[0].best_ask()) / 2.0;
    let final_price = {
        let last = snapshots.last().unwrap();
        (last.best_bid() + last.best_ask()) / 2.0
    };

    println!("📊 Market Overview:");
    println!("   Start Price:   ${:.2}", start_price);
    println!("   End Price:     ${:.2}", final_price);
    println!("   Change:        ${:.2} ({:+.2}%)", final_price - start_price, ((final_price - start_price) / start_price) * 100.0);
    println!();

    
    match config.strategy {
        StrategyType::Momentum => {
            export_momentum(&snapshots, &config.output_path, start_price, final_price)?;
        }
        StrategyType::Performance => {
            export_performance_comparison(&snapshots, &config.output_path)?;
        }
    }

    println!("\n✅ Export complete!");
    println!("   Output: {}", config.output_path.display());
    println!();

    Ok(())
}

fn print_progress(label: &str, processed: usize, total: usize) {
    if total == 0 {
        return;
    }
    let pct = (processed as f64 / total as f64) * 100.0;
    print!("\r   {:<18} {:>6.2}%", format!("{} progress", label), pct);
    let _ = io::stdout().flush();
    if processed == total {
        println!();
    }
}

fn export_momentum(
    snapshots: &[L2Snapshot],
    output_path: &Path,
    start_price: f64,
    final_price: f64,
) -> anyhow::Result<()> {
    const STARTING_CAPITAL: f64 = 10_000.0;  
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚀 EXPORTING MOMENTUM STRATEGY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config = MomentumConfig {
        trigger_threshold: 15.0,
        trade_size: 0.1,
        max_position: 1.0,
        lookback: 500,
    };

    let mut strategy = MomentumStrategy::new(config);
    let mut position = Position::new();

    
    let mut pnl_curve = Vec::new();
    let mut position_curve = Vec::new();
    let mut volume_curve = Vec::new();
    let mut cumulative_volume = 0.0;

    let start = Instant::now();

    for (idx, snapshot) in snapshots.iter().enumerate() {
        let trades = strategy.on_market_data(snapshot, &position);

        for trade in trades {
            cumulative_volume += trade.quantity;
            position.execute_trade(trade);
        }

        
        if idx % 100 == 0 {
            let mid_price = (snapshot.best_bid() + snapshot.best_ask()) / 2.0;
            let total_pnl = position.total_pnl(mid_price);

            pnl_curve.push(TimeseriesPoint {
                snapshot: idx,
                timestamp_us: snapshot.timestamp_us,
                value: total_pnl,
            });

            position_curve.push(TimeseriesPoint {
                snapshot: idx,
                timestamp_us: snapshot.timestamp_us,
                value: position.quantity,
            });

            volume_curve.push(TimeseriesPoint {
                snapshot: idx,
                timestamp_us: snapshot.timestamp_us,
                value: cumulative_volume,
            });
        }
    }

    let duration = start.elapsed();

    
    let mut max_pnl = 0.0;
    let mut drawdown_curve = Vec::new();
    for point in &pnl_curve {
        if point.value > max_pnl {
            max_pnl = point.value;
        }
        let drawdown = max_pnl - point.value;
        drawdown_curve.push(TimeseriesPoint {
            snapshot: point.snapshot,
            timestamp_us: point.timestamp_us,
            value: drawdown,
        });
    }

    
    let stats = strategy.stats();
    let mut result = BacktestResult::new("Momentum Strategy".to_string());
    result.calculate_from_position(&position, final_price, stats.updates_processed, stats.quotes_placed);
    result.set_timing(duration, snapshots.len());

    
    let timeseries = TimeseriesData {
        pnl_curve,
        position_curve,
        volume_curve,
        drawdown_curve,
    };

    let export = BacktestExport::from_backtest(
        &result,
        position.trades(),
        timeseries,
        start_price,
        final_price,
        STARTING_CAPITAL,
    );

    
    export.to_file(output_path)?;

    println!("✓ Momentum strategy exported");
    println!("   Starting Capital: ${:.2}", STARTING_CAPITAL);
    println!("   Final Capital:    ${:.2}", STARTING_CAPITAL + result.metrics.total_pnl);
    println!("   Total PnL:        ${:.2}", result.metrics.total_pnl);
    println!("   Return:           {:+.2}%", (result.metrics.total_pnl / STARTING_CAPITAL) * 100.0);
    println!("   Total Trades:     {}", result.metrics.total_trades);
    println!("   Duration:         {:?}", duration);
    println!();

    Ok(())
}

fn export_performance_comparison(
    snapshots: &[L2Snapshot],
    output_path: &Path,
) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚡ EXPORTING MOMENTUM PERFORMANCE COMPARISON");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let final_price = {
        let last = snapshots.last().unwrap();
        (last.best_bid() + last.best_ask()) / 2.0
    };

    let mut results = Vec::new();
    let base_config = MomentumConfig {
        trigger_threshold: 15.0,
        trade_size: 0.1,
        max_position: 1.0,
        lookback: 500,
    };

    
    println!("Running HFT Optimized momentum...");
    let mut strategy = MomentumStrategy::new(base_config.clone());
    let mut position = Position::new();
    let start = Instant::now();

    for (idx, snapshot) in snapshots.iter().enumerate() {
        let trades = strategy.on_market_data(snapshot, &position);
        for trade in trades {
            position.execute_trade(trade);
        }
        print_progress("HFT Optimized", idx + 1, snapshots.len());
    }

    let duration = start.elapsed();
    let stats = strategy.stats();
    let mut result = BacktestResult::new("HFT Optimized Momentum".to_string());
    result.calculate_from_position(&position, final_price, stats.updates_processed, stats.quotes_placed);
    result.set_timing(duration, snapshots.len());
    results.push(result);

    
    println!("Running Cached Naive momentum...");
    let mut strategy = NaiveMomentumStrategy::new(base_config.clone());
    let mut position = Position::new();
    let start = Instant::now();

    for (idx, snapshot) in snapshots.iter().enumerate() {
        let trades = strategy.on_market_data(snapshot, &position);
        for trade in trades {
            position.execute_trade(trade);
        }
        print_progress("Cached Naive", idx + 1, snapshots.len());
    }

    let duration = start.elapsed();
    let stats = strategy.stats();
    let mut result = BacktestResult::new("Cached Naive Momentum".to_string());
    result.calculate_from_position(&position, final_price, stats.updates_processed, stats.quotes_placed);
    result.set_timing(duration, snapshots.len());
    results.push(result);

    
    println!("Running Pure Naive momentum...");
    let mut strategy = PureNaiveMomentumStrategy::new(base_config);
    let mut position = Position::new();
    let start = Instant::now();

    for (idx, snapshot) in snapshots.iter().enumerate() {
        let trades = strategy.on_market_data(snapshot, &position);
        for trade in trades {
            position.execute_trade(trade);
        }
        print_progress("Pure Naive", idx + 1, snapshots.len());
    }

    let duration = start.elapsed();
    let stats = strategy.stats();
    let mut result = BacktestResult::new("Pure Naive Momentum".to_string());
    result.calculate_from_position(&position, final_price, stats.updates_processed, stats.quotes_placed);
    result.set_timing(duration, snapshots.len());
    results.push(result);

    let comparison = PerformanceComparison::new(
        "Momentum Strategy".to_string(),
        snapshots.len(),
        &results,
    );

    comparison.to_file(output_path)?;

    println!("✓ Performance comparison exported");
    println!();
    println!("   HFT Optimized:  {:.2} ns/snapshot", results[0].timing.time_per_snapshot.as_nanos());
    println!("   Cached Naive:   {:.2} ns/snapshot ({:.1}× slower)",
             results[1].timing.time_per_snapshot.as_nanos(),
             results[1].timing.total_duration.as_nanos() as f64 / results[0].timing.total_duration.as_nanos() as f64);
    println!("   Pure Naive:     {:.2} ns/snapshot ({:.1}× slower)",
             results[2].timing.time_per_snapshot.as_nanos(),
             results[2].timing.total_duration.as_nanos() as f64 / results[0].timing.total_duration.as_nanos() as f64);
    println!();

    Ok(())
}


#[derive(Debug)]
struct ExportConfig {
    strategy: StrategyType,
    num_snapshots: usize,
    output_path: PathBuf,
}

#[derive(Debug)]
enum StrategyType {
    Momentum,
    Performance,
}

fn parse_args(args: &[String]) -> anyhow::Result<ExportConfig> {
    if args.len() < 2 {
        print_usage();
        anyhow::bail!("Missing arguments");
    }

    let mut strategy = StrategyType::Momentum;
    let mut num_snapshots = 200_000;
    let mut output_path = PathBuf::from("results/");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--strategy" | "-s" => {
                if i + 1 >= args.len() {
                    anyhow::bail!("Missing value for --strategy");
                }
                strategy = match args[i + 1].as_str() {
                    "momentum" => StrategyType::Momentum,
                    "performance" => StrategyType::Performance,
                    other => anyhow::bail!("Unsupported strategy: {}", other),
                };
                i += 2;
            },
            "--snapshots" | "-n" => {
                if i + 1 >= args.len() {
                    anyhow::bail!("Missing value for --snapshots");
                }
                num_snapshots = args[i + 1].parse()?;
                i += 2;
            },
            "--output" | "-o" => {
                if i + 1 >= args.len() {
                    anyhow::bail!("Missing value for --output");
                }
                output_path = PathBuf::from(&args[i + 1]);
                i += 2;
            },
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            },
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_usage();
                anyhow::bail!("Invalid arguments");
            }
        }
    }

    Ok(ExportConfig {
        strategy,
        num_snapshots,
        output_path,
    })
}

fn print_usage() {
    println!("Usage: backtest_export [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --strategy, -s <TYPE>      Strategy to export ('momentum' or 'performance')");
    println!("                             Default: momentum");
    println!("  --snapshots, -n <NUM>      Number of snapshots to process");
    println!("                             Default: 200000");
    println!("  --output, -o <PATH>        Output file or directory");
    println!("                             Default: results/");
    println!("  --help, -h                 Show this help message");
    println!();
    println!("Examples:");
    println!("  backtest_export --strategy momentum --snapshots 7200 --output momentum_2hr.json");
    println!("  backtest_export --strategy performance --snapshots 200000 --output results/performance_200k.json");
}

# ZapHFT

A high-performance algorithmic trading backtesting engine written in Rust, focused on high-frequency trading (HFT) strategies with Level 2 market data.

## Features

- **Level 2 Order Book Processing**: Real-time order book reconstruction and analysis from snapshot data
- **Multiple Trading Strategies**:
  - Market Making with inventory management and skewing
  - Momentum-based strategies
  - Configurable strategy parameters
- **Comprehensive Position Management**: 
  - Profit/loss tracking
  - Inventory limits and risk controls
  - Trade execution simulation
- **Performance Analytics**:
  - Sharpe ratio calculation
  - Maximum drawdown analysis
  - Win rate and P&L statistics
  - Trade-by-trade performance tracking
- **Ultra-low Latency**: Optimized for microsecond-level processing
- **Export Capabilities**: CSV export for further analysis

## Requirements

- Rust 1.70 or later
- Cargo package manager

## Installation

Clone the repository and build the project:

```bash
git clone https://github.com/iminoaru/zaphft.git
cd zaphft
cargo build --release
```

## Usage

### Running Backtests

The project includes several binary targets for different backtesting scenarios:

```bash
# Market maker backtests
cargo run --release --bin backtest_realistic_mm
cargo run --release --bin backtest_profitable
cargo run --release --bin backtest_aggressive

# Strategy comparison
cargo run --release --bin backtest_comparison

# Export backtest results
cargo run --release --bin backtest_export

# Performance benchmarking
cargo run --release --bin benchmark

# Market analysis
cargo run --release --bin market_analysis
```

### Running Demos

```bash
# Market maker demo
cargo run --release --bin market_maker_demo

# Position management demo
cargo run --release --bin position_demo

# Main analysis demo
cargo run --release
```

### Batch Export Script

The `run_engine.sh` script automates the generation of multiple backtest exports for analysis:

```bash
bash run_engine.sh
```

This script will:
- Create a `benches/` directory for output files
- Generate full dataset exports (3.7M snapshots) for momentum and performance strategies
- Generate smaller exports (200k snapshots) for quick analysis
- Export results in JSON format for visualization and further processing

Output files:
- `benches/momentum_full.json` - Full momentum strategy backtest
- `benches/performance_full.json` - Full performance strategy backtest
- `benches/momentum_200k.json` - 200k snapshot momentum backtest
- `benches/performance_200k.json` - 200k snapshot performance backtest

## Data Format

The engine expects Level 2 order book data in CSV format with the following structure:

- Timestamp (microseconds)
- 10 bid levels (price and quantity)
- 10 ask levels (price and quantity)

Example data file location: `data/L2_processed.csv`

## Project Structure

```
src/
├── lib.rs              # Library exports
├── main.rs             # Main demo application
├── types.rs            # Core data structures (Side, Trade, L2Snapshot)
├── analytics/          # Performance analytics and export
├── bin/                # Binary targets (backtests, demos, benchmarks)
├── execution/          # Position and order execution
├── market_data/        # Data readers and processors
├── orderbook/          # Order book implementation
├── strategy/           # Trading strategies
├── trivial_approach/   # Alternative implementations
└── utils/              # CSV processing utilities
```

## Configuration

### Market Maker Configuration

```rust
MarketMakerConfig {
    spread_ticks: 0.5,           // Spread width in ticks
    quote_size: 0.1,             // Order size in BTC
    max_position: 1.0,           // Maximum position size
    tick_size: 0.05,             // Price tick size
    inventory_threshold: 0.9,    // Inventory limit threshold
    inventory_skew_ticks: 0.5,   // Price skew for inventory management
    trend_filter_ticks: 0.5,     // Trend filter threshold
    hedge_inventory_ratio: 0.5,  // Hedging ratio
}
```

## Performance Metrics

The engine calculates comprehensive performance metrics:

- **Total P&L**: Cumulative profit and loss
- **Sharpe Ratio**: Risk-adjusted return metric
- **Max Drawdown**: Largest peak-to-trough decline
- **Win Rate**: Percentage of profitable trades
- **Trade Count**: Number of executed trades
- **Average Trade P&L**: Mean profit per trade

## Benchmarks

The engine is optimized for high-frequency trading with typical performance:

- **Order Book Updates**: ~100,000+ snapshots/second
- **Latency**: <100 nanoseconds per snapshot
- **Memory Efficiency**: Minimal allocations in hot paths

Run benchmarks with:
```bash
cargo run --release --bin benchmark
```

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

The release build enables Link-Time Optimization (LTO) and aggressive optimizations for maximum performance.

## Key Components

### Order Book (`orderbook/`)
- Fast order book reconstruction from L2 snapshots
- Bid/ask spread calculation
- Slippage analysis
- Order book imbalance metrics

### Strategies (`strategy/`)
- **Market Maker**: Provides liquidity with inventory management
- **Momentum**: Trend-following strategy based on price movements

### Position Management (`execution/`)
- Real-time P&L tracking
- Position limits enforcement
- Trade execution with fees
- Risk management

### Analytics (`analytics/`)
- Performance calculation
- Trade export to CSV
- Statistical analysis

## License

This project is open source and available under the MIT License.

## Disclaimer

This software is for educational and research purposes only. It is not financial advice. Use at your own risk. Past performance does not guarantee future results.

## Contact

- GitHub: [@iminoaru](https://github.com/iminoaru)
- Repository: [zaphft](https://github.com/iminoaru/zaphft)

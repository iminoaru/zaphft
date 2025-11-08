mkdir -p benches

echo "=== Generating FULL dataset exports ==="
cargo run --release --bin backtest_export -- --strategy momentum --snapshots 3700000 --output benches/momentum_full.json
cargo run --release --bin backtest_export -- --strategy performance --snapshots 3700000 --output benches/performance_full.json

echo "=== Generating 200k exports ==="
cargo run --release --bin backtest_export -- --strategy momentum --snapshots 200000 --output benches/momentum_200k.json
cargo run --release --bin backtest_export -- --strategy performance --snapshots 200000 --output benches/performance_200k.json



echo "✅ ALL exports complete!"
du -sh benches/*
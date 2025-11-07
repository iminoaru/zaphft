




use serde::{Deserialize, Serialize};
use crate::types::{Side, Trade};
use super::{BacktestResult, PerformanceMetrics, TimingMetrics};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestExport {
    pub metadata: ExportMetadata,
    pub summary: SummaryMetrics,
    pub timeseries: TimeseriesData,
    pub trades: TradeHistory,
    pub risk: RiskMetrics,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub strategy_name: String,
    pub dataset_size: usize,
    pub timestamp: String,
    pub duration_ms: f64,
    pub throughput: f64,  
    pub starting_capital: f64,
    pub final_capital: f64,
    pub return_pct: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetrics {
    
    pub starting_capital: f64,
    pub final_capital: f64,
    pub return_pct: f64,

    
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

    
    pub time_per_snapshot_ns: f64,
    pub throughput_per_sec: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesData {
    
    pub pnl_curve: Vec<TimeseriesPoint>,

    
    pub position_curve: Vec<TimeseriesPoint>,

    
    pub volume_curve: Vec<TimeseriesPoint>,

    
    pub drawdown_curve: Vec<TimeseriesPoint>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesPoint {
    pub snapshot: usize,
    pub timestamp_us: u64,
    pub value: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    pub all_trades: Vec<TradeExport>,
    pub best_trade: Option<TradeExport>,
    pub worst_trade: Option<TradeExport>,
    pub recent_trades: Vec<TradeExport>,  
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExport {
    pub id: usize,
    pub timestamp_us: u64,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub pnl_impact: f64,  
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub metadata: ExportMetadata,
    pub approaches: Vec<ApproachMetrics>,
    pub speedup_data: Vec<SpeedupPoint>,  
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachMetrics {
    pub name: String,
    pub time_per_snapshot_ns: f64,
    pub throughput_per_sec: f64,
    pub total_duration_ms: f64,
    pub speedup_vs_optimized: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedupPoint {
    pub approach: String,
    pub speedup: f64,
}

impl BacktestExport {
    
    pub fn from_backtest(
        result: &BacktestResult,
        trades: &[Trade],
        timeseries: TimeseriesData,
        start_price: f64,
        final_price: f64,
        starting_capital: f64,
    ) -> Self {
        let total_pnl = result.metrics.total_pnl;
        let final_capital = starting_capital + total_pnl;
        let return_pct = (total_pnl / starting_capital) * 100.0;

        let metadata = ExportMetadata {
            strategy_name: result.name.clone(),
            dataset_size: result.timing.snapshots_processed,
            timestamp: chrono::Local::now().to_rfc3339(),
            duration_ms: result.timing.total_duration.as_secs_f64() * 1000.0,
            throughput: result.timing.throughput,
            starting_capital,
            final_capital,
            return_pct,
        };

        let summary = SummaryMetrics::from_metrics(&result.metrics, &result.timing, starting_capital);
        let trade_history = TradeHistory::from_trades(trades, start_price);
        let risk = RiskMetrics::calculate(trades, &timeseries.pnl_curve, start_price, final_price);

        Self {
            metadata,
            summary,
            timeseries,
            trades: trade_history,
            risk,
        }
    }

    
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    
    pub fn to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl SummaryMetrics {
    fn from_metrics(metrics: &PerformanceMetrics, timing: &TimingMetrics, starting_capital: f64) -> Self {
        let final_capital = starting_capital + metrics.total_pnl;
        let return_pct = (metrics.total_pnl / starting_capital) * 100.0;

        Self {
            starting_capital,
            final_capital,
            return_pct,
            total_pnl: metrics.total_pnl,
            realized_pnl: metrics.realized_pnl,
            unrealized_pnl: metrics.unrealized_pnl,
            final_position: metrics.final_position,
            max_position_long: metrics.max_position_long,
            max_position_short: metrics.max_position_short,
            avg_position: metrics.avg_position,
            total_trades: metrics.total_trades,
            winning_trades: metrics.winning_trades,
            losing_trades: metrics.losing_trades,
            win_rate: metrics.win_rate,
            total_volume: metrics.total_volume,
            buy_volume: metrics.buy_volume,
            sell_volume: metrics.sell_volume,
            updates_processed: metrics.updates_processed,
            quotes_placed: metrics.quotes_placed,
            quote_rate: metrics.quote_rate,
            time_per_snapshot_ns: timing.time_per_snapshot.as_nanos() as f64,
            throughput_per_sec: timing.throughput,
        }
    }
}

impl TradeHistory {
    fn from_trades(trades: &[Trade], start_price: f64) -> Self {
        let mut all_trades = Vec::new();
        let mut current_pos = 0.0;
        let mut avg_entry = start_price;

        for (id, trade) in trades.iter().enumerate() {
            let signed_qty = match trade.side {
                Side::Bid => trade.quantity,
                Side::Ask => -trade.quantity,
            };

            
            let pnl_impact = if current_pos > 0.0 && matches!(trade.side, Side::Ask) {
                
                (trade.price - avg_entry) * trade.quantity.min(current_pos)
            } else if current_pos < 0.0 && matches!(trade.side, Side::Bid) {
                
                (avg_entry - trade.price) * trade.quantity.min(current_pos.abs())
            } else {
                0.0
            };

            all_trades.push(TradeExport {
                id,
                timestamp_us: trade.timestamp_us,
                side: match trade.side {
                    Side::Bid => "buy".to_string(),
                    Side::Ask => "sell".to_string(),
                },
                price: trade.price,
                size: trade.quantity,
                pnl_impact,
            });

            
            current_pos += signed_qty;

            
            if current_pos == 0.0 {
                avg_entry = trade.price;
            } else if (current_pos > 0.0 && matches!(trade.side, Side::Bid)) ||
                      (current_pos < 0.0 && matches!(trade.side, Side::Ask)) {
                
                avg_entry = trade.price;
            }
        }

        
        let best_trade = all_trades.iter()
            .max_by(|a, b| a.pnl_impact.partial_cmp(&b.pnl_impact).unwrap())
            .cloned();

        let worst_trade = all_trades.iter()
            .min_by(|a, b| a.pnl_impact.partial_cmp(&b.pnl_impact).unwrap())
            .cloned();

        
        let recent_trades = all_trades.iter()
            .rev()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        Self {
            all_trades,
            best_trade,
            worst_trade,
            recent_trades,
        }
    }
}

impl RiskMetrics {
    fn calculate(
        trades: &[Trade],
        pnl_curve: &[TimeseriesPoint],
        start_price: f64,
        _final_price: f64,
    ) -> Self {
        
        let (max_dd, max_dd_pct) = Self::calculate_max_drawdown(pnl_curve);

        
        let sharpe = Self::calculate_sharpe_ratio(pnl_curve);

        
        let (profit_factor, avg_win, avg_loss, largest_win, largest_loss) =
            Self::calculate_profit_metrics(trades, start_price);

        Self {
            max_drawdown: max_dd,
            max_drawdown_pct: max_dd_pct,
            sharpe_ratio: sharpe,
            profit_factor,
            avg_win,
            avg_loss,
            largest_win,
            largest_loss,
        }
    }

    fn calculate_max_drawdown(pnl_curve: &[TimeseriesPoint]) -> (f64, f64) {
        if pnl_curve.is_empty() {
            return (0.0, 0.0);
        }

        let mut max_value = pnl_curve[0].value;
        let mut max_drawdown = 0.0;
        let mut max_drawdown_pct = 0.0;

        for point in pnl_curve {
            if point.value > max_value {
                max_value = point.value;
            }

            let drawdown = max_value - point.value;
            let drawdown_pct = if max_value > 0.0 {
                (drawdown / max_value) * 100.0
            } else {
                0.0
            };

            if drawdown > max_drawdown {
                max_drawdown = drawdown;
                max_drawdown_pct = drawdown_pct;
            }
        }

        (max_drawdown, max_drawdown_pct)
    }

    fn calculate_sharpe_ratio(pnl_curve: &[TimeseriesPoint]) -> f64 {
        if pnl_curve.len() < 2 {
            return 0.0;
        }

        
        let mut returns = Vec::new();
        for i in 1..pnl_curve.len() {
            let ret = pnl_curve[i].value - pnl_curve[i-1].value;
            returns.push(ret);
        }

        if returns.is_empty() {
            return 0.0;
        }

        
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev < 1e-10 {
            return 0.0;
        }

        
        mean / std_dev * (252.0_f64).sqrt()  
    }

    fn calculate_profit_metrics(trades: &[Trade], start_price: f64) -> (f64, f64, f64, f64, f64) {
        if trades.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let mut wins = Vec::new();
        let mut losses = Vec::new();
        let mut current_pos = 0.0;
        let mut avg_entry = start_price;

        for trade in trades {
            let signed_qty = match trade.side {
                Side::Bid => trade.quantity,
                Side::Ask => -trade.quantity,
            };

            
            let trade_pnl = if current_pos > 0.0 && matches!(trade.side, Side::Ask) {
                (trade.price - avg_entry) * trade.quantity.min(current_pos)
            } else if current_pos < 0.0 && matches!(trade.side, Side::Bid) {
                (avg_entry - trade.price) * trade.quantity.min(current_pos.abs())
            } else {
                0.0
            };

            if trade_pnl > 0.0 {
                wins.push(trade_pnl);
            } else if trade_pnl < 0.0 {
                losses.push(trade_pnl.abs());
            }

            current_pos += signed_qty;
            if current_pos.abs() < 1e-10 {
                avg_entry = trade.price;
            }
        }

        let total_wins: f64 = wins.iter().sum();
        let total_losses: f64 = losses.iter().sum();
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else if total_wins > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let avg_win = if !wins.is_empty() {
            total_wins / wins.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losses.is_empty() {
            total_losses / losses.len() as f64
        } else {
            0.0
        };

        let largest_win = wins.iter().copied().fold(0.0, f64::max);
        let largest_loss = losses.iter().copied().fold(0.0, f64::max);

        (profit_factor, avg_win, avg_loss, largest_win, largest_loss)
    }
}

impl PerformanceComparison {
    pub fn new(
        strategy_name: String,
        dataset_size: usize,
        results: &[BacktestResult],
    ) -> Self {
        const STARTING_CAPITAL: f64 = 10_000.0;
        let total_pnl = results[0].metrics.total_pnl;
        let final_capital = STARTING_CAPITAL + total_pnl;
        let return_pct = (total_pnl / STARTING_CAPITAL) * 100.0;

        let metadata = ExportMetadata {
            strategy_name,
            dataset_size,
            timestamp: chrono::Local::now().to_rfc3339(),
            duration_ms: results[0].timing.total_duration.as_secs_f64() * 1000.0,
            throughput: results[0].timing.throughput,
            starting_capital: STARTING_CAPITAL,
            final_capital,
            return_pct,
        };

        let approaches: Vec<ApproachMetrics> = results.iter().map(|r| {
            let speedup = if results[0].timing.total_duration.as_nanos() > 0 {
                r.timing.total_duration.as_nanos() as f64
                    / results[0].timing.total_duration.as_nanos() as f64
            } else {
                1.0
            };

            ApproachMetrics {
                name: r.name.clone(),
                time_per_snapshot_ns: r.timing.time_per_snapshot.as_nanos() as f64,
                throughput_per_sec: r.timing.throughput,
                total_duration_ms: r.timing.total_duration.as_secs_f64() * 1000.0,
                speedup_vs_optimized: speedup,
            }
        }).collect();

        let speedup_data: Vec<SpeedupPoint> = results.iter().map(|r| {
            let speedup = if results[0].timing.total_duration.as_nanos() > 0 {
                r.timing.total_duration.as_nanos() as f64
                    / results[0].timing.total_duration.as_nanos() as f64
            } else {
                1.0
            };

            SpeedupPoint {
                approach: r.name.clone(),
                speedup,
            }
        }).collect();

        Self {
            metadata,
            approaches,
            speedup_data,
        }
    }

    
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    
    pub fn to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

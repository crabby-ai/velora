//! Performance analytics and metrics calculation.

use crate::portfolio::{CompletedTrade, EquityPoint};
use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Complete performance metrics for a backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Return Metrics
    /// Total return as percentage
    pub total_return: f64,

    /// Annualized return as percentage
    pub annualized_return: f64,

    /// Total profit/loss in dollars
    pub total_pnl: f64,

    // Risk Metrics
    /// Sharpe ratio (risk-adjusted return)
    pub sharpe_ratio: f64,

    /// Sortino ratio (downside risk-adjusted return)
    pub sortino_ratio: f64,

    /// Maximum drawdown as percentage
    pub max_drawdown: f64,

    /// Duration of maximum drawdown
    pub max_drawdown_duration_days: i64,

    // Trade Statistics
    /// Total number of completed trades
    pub total_trades: usize,

    /// Number of winning trades
    pub winning_trades: usize,

    /// Number of losing trades
    pub losing_trades: usize,

    /// Win rate as percentage
    pub win_rate: f64,

    /// Average profit of winning trades
    pub avg_win: f64,

    /// Average loss of losing trades
    pub avg_loss: f64,

    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,

    /// Largest winning trade
    pub largest_win: f64,

    /// Largest losing trade
    pub largest_loss: f64,

    // Time Metrics
    /// Average holding period in hours
    pub avg_holding_period_hours: f64,

    /// Total backtest duration in days
    pub duration_days: i64,
}

/// Calculate performance metrics from backtest results
pub fn calculate_metrics(
    equity_curve: &[EquityPoint],
    trades: &[CompletedTrade],
    initial_capital: f64,
) -> PerformanceMetrics {
    let total_pnl = calculate_total_pnl(trades);
    let total_return = (total_pnl / initial_capital) * 100.0;

    let duration_days = calculate_duration_days(equity_curve);
    let annualized_return = if duration_days > 0 {
        let years = duration_days as f64 / 365.25;
        ((1.0 + total_return / 100.0).powf(1.0 / years) - 1.0) * 100.0
    } else {
        0.0
    };

    let (winning_trades, losing_trades) = count_winning_losing(trades);
    let total_trades = trades.len();
    let win_rate = if total_trades > 0 {
        (winning_trades as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let (avg_win, largest_win) = calculate_win_stats(trades);
    let (avg_loss, largest_loss) = calculate_loss_stats(trades);

    let profit_factor = calculate_profit_factor(trades);

    let daily_returns = calculate_daily_returns(equity_curve);
    let sharpe_ratio = calculate_sharpe_ratio(&daily_returns);
    let sortino_ratio = calculate_sortino_ratio(&daily_returns);

    let (max_drawdown, max_drawdown_duration_days) = calculate_max_drawdown(equity_curve);

    let avg_holding_period_hours = calculate_avg_holding_period(trades);

    PerformanceMetrics {
        total_return,
        annualized_return,
        total_pnl,
        sharpe_ratio,
        sortino_ratio,
        max_drawdown,
        max_drawdown_duration_days,
        total_trades,
        winning_trades,
        losing_trades,
        win_rate,
        avg_win,
        avg_loss,
        profit_factor,
        largest_win,
        largest_loss,
        avg_holding_period_hours,
        duration_days,
    }
}

fn calculate_total_pnl(trades: &[CompletedTrade]) -> f64 {
    trades.iter().map(|t| t.pnl).sum()
}

fn calculate_duration_days(equity_curve: &[EquityPoint]) -> i64 {
    if equity_curve.len() < 2 {
        return 0;
    }

    let start = equity_curve.first().unwrap().timestamp;
    let end = equity_curve.last().unwrap().timestamp;
    (end - start).num_days()
}

fn count_winning_losing(trades: &[CompletedTrade]) -> (usize, usize) {
    let winning = trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing = trades.iter().filter(|t| t.pnl < 0.0).count();
    (winning, losing)
}

fn calculate_win_stats(trades: &[CompletedTrade]) -> (f64, f64) {
    let wins: Vec<f64> = trades
        .iter()
        .filter(|t| t.pnl > 0.0)
        .map(|t| t.pnl)
        .collect();

    if wins.is_empty() {
        return (0.0, 0.0);
    }

    let avg = wins.iter().sum::<f64>() / wins.len() as f64;
    let max = wins.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    (avg, max)
}

fn calculate_loss_stats(trades: &[CompletedTrade]) -> (f64, f64) {
    let losses: Vec<f64> = trades
        .iter()
        .filter(|t| t.pnl < 0.0)
        .map(|t| t.pnl)
        .collect();

    if losses.is_empty() {
        return (0.0, 0.0);
    }

    let avg = losses.iter().sum::<f64>() / losses.len() as f64;
    let max = losses.iter().copied().fold(f64::INFINITY, f64::min);

    (avg, max)
}

fn calculate_profit_factor(trades: &[CompletedTrade]) -> f64 {
    let gross_profit: f64 = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();

    let gross_loss: f64 = trades
        .iter()
        .filter(|t| t.pnl < 0.0)
        .map(|t| t.pnl.abs())
        .sum();

    if gross_loss == 0.0 {
        if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        gross_profit / gross_loss
    }
}

fn calculate_daily_returns(equity_curve: &[EquityPoint]) -> Vec<f64> {
    if equity_curve.len() < 2 {
        return vec![];
    }

    equity_curve
        .windows(2)
        .map(|window| {
            let prev = window[0].equity;
            let curr = window[1].equity;
            ((curr - prev) / prev) * 100.0
        })
        .collect()
}

fn calculate_sharpe_ratio(daily_returns: &[f64]) -> f64 {
    if daily_returns.is_empty() {
        return 0.0;
    }

    let mean = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;

    let variance = daily_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / daily_returns.len() as f64;

    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        0.0
    } else {
        // Annualize: sqrt(252 trading days)
        (mean / std_dev) * (252.0_f64).sqrt()
    }
}

fn calculate_sortino_ratio(daily_returns: &[f64]) -> f64 {
    if daily_returns.is_empty() {
        return 0.0;
    }

    let mean = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;

    // Only consider downside deviation
    let downside_returns: Vec<f64> = daily_returns
        .iter()
        .filter(|&&r| r < 0.0)
        .copied()
        .collect();

    if downside_returns.is_empty() {
        return f64::INFINITY;
    }

    let downside_variance =
        downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / downside_returns.len() as f64;

    let downside_dev = downside_variance.sqrt();

    if downside_dev == 0.0 {
        0.0
    } else {
        // Annualize
        (mean / downside_dev) * (252.0_f64).sqrt()
    }
}

fn calculate_max_drawdown(equity_curve: &[EquityPoint]) -> (f64, i64) {
    if equity_curve.is_empty() {
        return (0.0, 0);
    }

    let mut peak = equity_curve[0].equity;
    let mut peak_time = equity_curve[0].timestamp;
    let mut max_dd = 0.0;
    let mut max_dd_duration = Duration::zero();
    let mut current_dd_start = peak_time;

    for point in equity_curve {
        if point.equity > peak {
            peak = point.equity;
            peak_time = point.timestamp;
            current_dd_start = peak_time;
        } else {
            let dd = ((point.equity - peak) / peak) * 100.0;
            if dd < max_dd {
                max_dd = dd;
                let duration = point.timestamp - current_dd_start;
                if duration > max_dd_duration {
                    max_dd_duration = duration;
                }
            }
        }
    }

    (max_dd, max_dd_duration.num_days())
}

fn calculate_avg_holding_period(trades: &[CompletedTrade]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let total_hours: i64 = trades
        .iter()
        .map(|t| (t.exit_time - t.entry_time).num_hours())
        .sum();

    total_hours as f64 / trades.len() as f64
}

impl PerformanceMetrics {
    /// Print a formatted summary of the metrics
    pub fn print_summary(&self) {
        println!("\n=== Backtest Performance Summary ===\n");

        println!("Period: {} days\n", self.duration_days);

        println!("Returns:");
        println!("  Total Return:        {:>10.2}%", self.total_return);
        println!("  Annualized Return:   {:>10.2}%", self.annualized_return);
        println!("  Total P&L:           {:>10.2}", self.total_pnl);

        println!("\nRisk Metrics:");
        println!("  Sharpe Ratio:        {:>10.2}", self.sharpe_ratio);
        println!("  Sortino Ratio:       {:>10.2}", self.sortino_ratio);
        println!("  Max Drawdown:        {:>10.2}%", self.max_drawdown);
        println!(
            "  Max DD Duration:     {:>10} days",
            self.max_drawdown_duration_days
        );

        println!("\nTrade Statistics:");
        println!("  Total Trades:        {:>10}", self.total_trades);
        println!(
            "  Winning Trades:      {:>10} ({:.2}%)",
            self.winning_trades, self.win_rate
        );
        println!(
            "  Losing Trades:       {:>10} ({:.2}%)",
            self.losing_trades,
            100.0 - self.win_rate
        );
        println!("  Avg Win:             {:>10.2}", self.avg_win);
        println!("  Avg Loss:            {:>10.2}", self.avg_loss);
        println!("  Profit Factor:       {:>10.2}", self.profit_factor);
        println!("  Largest Win:         {:>10.2}", self.largest_win);
        println!("  Largest Loss:        {:>10.2}", self.largest_loss);

        println!("\nPosition Metrics:");
        println!(
            "  Avg Holding Period:  {:>10.1} hours",
            self.avg_holding_period_hours
        );

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use velora_strategy::PositionSide;

    #[test]
    fn test_calculate_total_pnl() {
        let trades = vec![
            CompletedTrade {
                symbol: "BTC".to_string(),
                side: PositionSide::Long,
                entry_time: Utc::now(),
                exit_time: Utc::now(),
                entry_price: 50000.0,
                exit_price: 51000.0,
                quantity: 1.0,
                pnl: 1000.0,
                pnl_pct: 2.0,
                commission: 50.0,
            },
            CompletedTrade {
                symbol: "BTC".to_string(),
                side: PositionSide::Long,
                entry_time: Utc::now(),
                exit_time: Utc::now(),
                entry_price: 51000.0,
                exit_price: 50500.0,
                quantity: 1.0,
                pnl: -500.0,
                pnl_pct: -1.0,
                commission: 50.0,
            },
        ];

        assert_eq!(calculate_total_pnl(&trades), 500.0);
    }

    #[test]
    fn test_win_rate_calculation() {
        let trades = vec![
            CompletedTrade {
                symbol: "BTC".to_string(),
                side: PositionSide::Long,
                entry_time: Utc::now(),
                exit_time: Utc::now(),
                entry_price: 50000.0,
                exit_price: 51000.0,
                quantity: 1.0,
                pnl: 1000.0,
                pnl_pct: 2.0,
                commission: 0.0,
            },
            CompletedTrade {
                symbol: "BTC".to_string(),
                side: PositionSide::Long,
                entry_time: Utc::now(),
                exit_time: Utc::now(),
                entry_price: 51000.0,
                exit_price: 50500.0,
                quantity: 1.0,
                pnl: -500.0,
                pnl_pct: -1.0,
                commission: 0.0,
            },
        ];

        let (winning, losing) = count_winning_losing(&trades);
        assert_eq!(winning, 1);
        assert_eq!(losing, 1);
    }

    #[test]
    fn test_profit_factor() {
        let trades = vec![
            CompletedTrade {
                symbol: "BTC".to_string(),
                side: PositionSide::Long,
                entry_time: Utc::now(),
                exit_time: Utc::now(),
                entry_price: 50000.0,
                exit_price: 52000.0,
                quantity: 1.0,
                pnl: 2000.0,
                pnl_pct: 4.0,
                commission: 0.0,
            },
            CompletedTrade {
                symbol: "BTC".to_string(),
                side: PositionSide::Long,
                entry_time: Utc::now(),
                exit_time: Utc::now(),
                entry_price: 51000.0,
                exit_price: 50000.0,
                quantity: 1.0,
                pnl: -1000.0,
                pnl_pct: -2.0,
                commission: 0.0,
            },
        ];

        let pf = calculate_profit_factor(&trades);
        assert_eq!(pf, 2.0); // 2000 / 1000
    }
}

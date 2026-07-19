use std::sync::Arc;

use chrono::Utc;
use pumpfun::common::types::{Cluster, PriorityFee};
use pumpfun::PumpFun;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use sqlx::SqlitePool;

use serde_wincode::{wincode, SerdeCompat};

use coins_database::models::trade::{Trade, TradeStatus};
use coins_database::queries::risk_settings::{get_risk, upsert_risk};
use coins_database::queries::trade::{create, get_by_id, update};

const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;
const TX_FEE_BUFFER_SOL: f64 = 0.01;

/// Error returned when the on-chain wallet balance is insufficient for a trade.
#[derive(Debug, Clone)]
pub struct InsufficientBalance {
    pub balance_sol: f64,
    pub required_sol: f64,
}

impl std::fmt::Display for InsufficientBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "wallet balance {:.6} SOL is less than required {:.6} SOL",
            self.balance_sol, self.required_sol,
        )
    }
}

impl std::error::Error for InsufficientBalance {}

/// Check the on-chain SOL balance of the wallet.
///
/// Returns the balance in SOL.
pub async fn check_wallet_balance(
    keypair: &Keypair,
    rpc_url: &str,
) -> anyhow::Result<f64> {
    let rpc = RpcClient::new(rpc_url.to_string());
    let lamports = rpc
        .get_balance(&keypair.pubkey())
        .await
        .map_err(|e| anyhow::anyhow!("failed to fetch wallet balance: {e}"))?;
    Ok(lamports as f64 / LAMPORTS_PER_SOL)
}

/// Ensure the wallet has enough SOL to cover the trade amount plus a fee buffer.
///
/// Returns `Ok(())` if sufficient, or `Err(InsufficientBalance)` if not.
pub async fn ensure_sufficient_balance(
    keypair: &Keypair,
    rpc_url: &str,
    required_sol: f64,
) -> Result<(), InsufficientBalance> {
    let balance_sol = check_wallet_balance(keypair, rpc_url)
        .await
        .unwrap_or(0.0);
    let needed = required_sol + TX_FEE_BUFFER_SOL;
    if balance_sol >= needed {
        Ok(())
    } else {
        Err(InsufficientBalance {
            balance_sol,
            required_sol: needed,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RealBuyRequest {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub amount_sol: f64,
    pub slippage_basis_points: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
    pub narrative: String,
}

#[derive(Debug, Clone)]
pub struct RealSellRequest {
    pub trade_id: i64,
    pub slippage_basis_points: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
    pub close_reason: Option<String>,
}

pub async fn real_buy(
    pool: &SqlitePool,
    keypair: Arc<Keypair>,
    rpc_url: &str,
    req: RealBuyRequest,
) -> anyhow::Result<Trade> {
    let ws_url = rpc_url.replace("https://", "wss://");
    let priority_fee = req.priority_fee_lamports.map(|f| PriorityFee {
        unit_limit: Some(200_000),
        unit_price: Some(f),
    });
    let cluster = Cluster::new(
        rpc_url.into(),
        ws_url,
        CommitmentConfig::confirmed(),
        priority_fee.unwrap_or(PriorityFee {
            unit_limit: None,
            unit_price: None,
        }),
    );

    ensure_sufficient_balance(&keypair, rpc_url, req.amount_sol)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let pump = PumpFun::new(keypair.clone(), cluster);
    let mint: Pubkey = req.mint.parse()?;
    let amount_lamports = (req.amount_sol * LAMPORTS_PER_SOL) as u64;

    let sig = pump
        .buy(
            mint,
            amount_lamports,
            Some(false),
            req.slippage_basis_points,
            None,
        )
        .await
        .map_err(|e| anyhow::anyhow!("PumpFun buy failed: {e}"))?;

    let mut settings = get_risk(pool).await?;

    if req.amount_sol > settings.real_portfolio_value {
        anyhow::bail!(
            "insufficient real portfolio value: have {:.2}, need {:.2}",
            settings.real_portfolio_value,
            req.amount_sol,
        );
    }

    let now = Utc::now().naive_utc();

    let trade = Trade {
        id: 0,
        address: req.mint.clone(),
        symbol: req.symbol,
        name: req.name,
        status: TradeStatus::Bought,
        trade_type: "real".into(),
        entry_price: None,
        entry_date: Some(now),
        position_size: Some(req.amount_sol),
        exit_price: None,
        exit_date: None,
        notes: String::new(),
        stop_loss_pct: Some(settings.default_stop_pct),
        stop_price: None,
        trailing_stop: false,
        peak_price: None,
        stop_loss_enabled: true,
        take_profit_enabled: false,
        take_profit_multiplier: None,
        peak_decay_enabled: false,
        peak_decay_pct: None,
        volume_exhaustion_enabled: false,
        volume_exhaustion_pct: None,
        peak_volume_24h: None,
        close_reason: None,
        tx_hash: sig.to_string(),
        narrative: req.narrative,
        pump_graduated: false,
        created_at: now,
        updated_at: now,
    };

    let trade = create(pool, &trade).await?;

    settings.real_portfolio_value -= req.amount_sol;
    if settings.real_portfolio_value > settings.real_peak_value {
        settings.real_peak_value = settings.real_portfolio_value;
    }
    settings.updated_at = now;
    upsert_risk(pool, &settings).await?;

    Ok(trade)
}

pub async fn real_sell(
    pool: &SqlitePool,
    keypair: Arc<Keypair>,
    rpc_url: &str,
    req: RealSellRequest,
) -> anyhow::Result<Trade> {
    let mut trade = get_by_id(pool, req.trade_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("trade {} not found", req.trade_id))?;

    anyhow::ensure!(
        trade.status == TradeStatus::Bought,
        "trade {} is not Bought (status: {:?})",
        req.trade_id,
        trade.status,
    );

    let ws_url = rpc_url.replace("https://", "wss://");
    let priority_fee = req.priority_fee_lamports.map(|f| PriorityFee {
        unit_limit: Some(200_000),
        unit_price: Some(f),
    });
    let cluster = Cluster::new(
        rpc_url.into(),
        ws_url,
        CommitmentConfig::confirmed(),
        priority_fee.unwrap_or(PriorityFee {
            unit_limit: None,
            unit_price: None,
        }),
    );

    let pump = PumpFun::new(keypair.clone(), cluster);
    let mint: Pubkey = trade.address.parse()?;

    let sig = pump
        .sell(mint, None, req.slippage_basis_points, None)
        .await
        .map_err(|e| anyhow::anyhow!("PumpFun sell failed: {e}"))?;

    let now = Utc::now().naive_utc();

    trade.status = TradeStatus::Sold;
    trade.exit_date = Some(now);
    trade.close_reason = req.close_reason;
    trade.tx_hash = sig.to_string();
    trade.updated_at = now;

    update(pool, &trade).await?;

    let mut settings = get_risk(pool).await?;
    settings.real_portfolio_value += trade.position_size.unwrap_or(0.0);
    if settings.real_portfolio_value > settings.real_peak_value {
        settings.real_peak_value = settings.real_portfolio_value;
    }
    settings.updated_at = now;
    upsert_risk(pool, &settings).await?;

    Ok(trade)
}

pub async fn jupiter_buy(
    pool: &SqlitePool,
    keypair: Arc<Keypair>,
    rpc_url: &str,
    jupiter_url: &str,
    req: RealBuyRequest,
) -> anyhow::Result<Trade> {
    use jup_ag_sdk::types::{QuoteRequest, SwapRequest};
    use jup_ag_sdk::JupiterClient;

    let mut settings = get_risk(pool).await?;

    if req.amount_sol > settings.real_portfolio_value {
        anyhow::bail!(
            "insufficient real portfolio value: have {:.2}, need {:.2}",
            settings.real_portfolio_value,
            req.amount_sol,
        );
    }

    ensure_sufficient_balance(&keypair, rpc_url, req.amount_sol)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let amount_lamports = (req.amount_sol * LAMPORTS_PER_SOL) as u64;
    let sol_mint = coins_config::scanner::SOL_ADDRESS;
    let output_mint = &req.mint;

    let jupiter = JupiterClient::new(jupiter_url);
    let quote = jupiter
        .get_quote(
            &QuoteRequest::new(sol_mint, output_mint, amount_lamports)
                .slippage_bps(req.slippage_basis_points.unwrap_or(500) as u16),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Jupiter quote failed: {e}"))?;

    let swap_req = SwapRequest::new(
        keypair.pubkey().to_string(),
        keypair.pubkey().to_string(),
        quote,
    );
    let swap_resp = jupiter
        .get_swap_transaction(&swap_req)
        .await
        .map_err(|e| anyhow::anyhow!("Jupiter swap tx failed: {e}"))?;

    let rpc = RpcClient::new(rpc_url.to_string());
    let tx_bytes = bs64_decode(&swap_resp.swap_transaction)?;
    let mut tx: solana_sdk::transaction::Transaction =
        <SerdeCompat<solana_sdk::transaction::Transaction> as wincode::Deserialize>::deserialize(
            &tx_bytes,
        )?;

    let blockhash = rpc
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow::anyhow!("failed to get blockhash: {e}"))?;
    tx.try_sign(&[keypair.as_ref()], blockhash)?;

    let sig = rpc
        .send_transaction(&tx)
        .await
        .map_err(|e| anyhow::anyhow!("Jupiter swap submit failed: {e}"))?;

    let now = Utc::now().naive_utc();

    let trade = Trade {
        id: 0,
        address: req.mint.clone(),
        symbol: req.symbol,
        name: req.name,
        status: TradeStatus::Bought,
        trade_type: "real".into(),
        entry_price: None,
        entry_date: Some(now),
        position_size: Some(req.amount_sol),
        exit_price: None,
        exit_date: None,
        notes: String::new(),
        stop_loss_pct: Some(settings.default_stop_pct),
        stop_price: None,
        trailing_stop: false,
        peak_price: None,
        stop_loss_enabled: true,
        take_profit_enabled: false,
        take_profit_multiplier: None,
        peak_decay_enabled: false,
        peak_decay_pct: None,
        volume_exhaustion_enabled: false,
        volume_exhaustion_pct: None,
        peak_volume_24h: None,
        close_reason: None,
        tx_hash: sig.to_string(),
        narrative: req.narrative,
        pump_graduated: true,
        created_at: now,
        updated_at: now,
    };

    let trade = create(pool, &trade).await?;

    settings.real_portfolio_value -= req.amount_sol;
    if settings.real_portfolio_value > settings.real_peak_value {
        settings.real_peak_value = settings.real_portfolio_value;
    }
    settings.updated_at = now;
    upsert_risk(pool, &settings).await?;

    Ok(trade)
}

pub async fn jupiter_sell(
    pool: &SqlitePool,
    keypair: Arc<Keypair>,
    rpc_url: &str,
    jupiter_url: &str,
    req: RealSellRequest,
) -> anyhow::Result<Trade> {
    use jup_ag_sdk::types::{QuoteRequest, SwapRequest};
    use jup_ag_sdk::JupiterClient;

    let mut trade = get_by_id(pool, req.trade_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("trade {} not found", req.trade_id))?;

    anyhow::ensure!(
        trade.status == TradeStatus::Bought,
        "trade {} is not Bought (status: {:?})",
        req.trade_id,
        trade.status,
    );

    let position_size = trade.position_size.unwrap_or(0.0);
    let entry_price = trade.entry_price.unwrap_or(0.0);
    let token_amount = if entry_price > 0.0 {
        ((position_size / entry_price) * 1_000_000.0) as u64
    } else {
        0
    };

    let sol_mint = coins_config::scanner::SOL_ADDRESS;
    let input_mint = &trade.address;

    let jupiter = JupiterClient::new(jupiter_url);
    let quote = jupiter
        .get_quote(
            &QuoteRequest::new(input_mint, sol_mint, token_amount)
                .slippage_bps(req.slippage_basis_points.unwrap_or(500) as u16),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Jupiter quote failed: {e}"))?;

    let swap_req = SwapRequest::new(
        keypair.pubkey().to_string(),
        keypair.pubkey().to_string(),
        quote,
    );
    let swap_resp = jupiter
        .get_swap_transaction(&swap_req)
        .await
        .map_err(|e| anyhow::anyhow!("Jupiter swap tx failed: {e}"))?;

    let rpc = RpcClient::new(rpc_url.to_string());
    let tx_bytes = bs64_decode(&swap_resp.swap_transaction)?;
    let mut tx: solana_sdk::transaction::Transaction =
        <SerdeCompat<solana_sdk::transaction::Transaction> as wincode::Deserialize>::deserialize(
            &tx_bytes,
        )?;

    let blockhash = rpc
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow::anyhow!("failed to get blockhash: {e}"))?;
    tx.try_sign(&[keypair.as_ref()], blockhash)?;

    let sig = rpc
        .send_transaction(&tx)
        .await
        .map_err(|e| anyhow::anyhow!("Jupiter swap submit failed: {e}"))?;

    let now = Utc::now().naive_utc();

    trade.status = TradeStatus::Sold;
    trade.exit_date = Some(now);
    trade.close_reason = req.close_reason;
    trade.tx_hash = sig.to_string();
    trade.updated_at = now;

    update(pool, &trade).await?;

    let mut settings = get_risk(pool).await?;
    settings.real_portfolio_value += position_size;
    if settings.real_portfolio_value > settings.real_peak_value {
        settings.real_peak_value = settings.real_portfolio_value;
    }
    settings.updated_at = now;
    upsert_risk(pool, &settings).await?;

    Ok(trade)
}

fn bs64_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| anyhow::anyhow!("base64 decode failed: {e}"))
}

// ---------------------------------------------------------------------------
// Readiness
// ---------------------------------------------------------------------------

/// Result of a single readiness check.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    pub name: &'static str,
    pub passed: bool,
    pub message: Option<String>,
}

/// Summary of the overall real-trade readiness.
#[derive(Debug, Clone)]
pub struct ReadinessReport {
    pub checks: Vec<CheckResult>,
}

impl ReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
}

/// Minimum SOL a wallet should hold to be considered ready for trading.
/// Covers a few tx fees and the rent-exempt minimum for an associated token account.
pub const MIN_WALLET_BALANCE_SOL: f64 = 0.005;

/// Run a suite of readiness checks for real trading.
///
/// Checks performed:
/// 1. **RPC connectivity** – fetches the latest blockhash.
/// 2. **Wallet balance** – fetches the on-chain SOL balance.
/// 3. **Minimum balance** – the wallet holds at least [`MIN_WALLET_BALANCE_SOL`] SOL.
pub async fn is_ready(keypair: &Keypair, rpc_url: &str) -> ReadinessReport {
    let mut checks = Vec::with_capacity(3);

    // 1. RPC reachable
    {
        let rpc = RpcClient::new(rpc_url.to_string());
        match rpc.get_latest_blockhash().await {
            Ok(_) => checks.push(CheckResult {
                name: "rpc_reachable",
                passed: true,
                message: None,
            }),
            Err(e) => checks.push(CheckResult {
                name: "rpc_reachable",
                passed: false,
                message: Some(format!("RPC unreachable: {e}")),
            }),
        }
    }

    // 2. Wallet balance
    {
        let rpc = RpcClient::new(rpc_url.to_string());
        match rpc.get_balance(&keypair.pubkey()).await {
            Ok(lamports) => {
                let balance_sol = lamports as f64 / LAMPORTS_PER_SOL;
                checks.push(CheckResult {
                    name: "wallet_exists",
                    passed: true,
                    message: Some(format!("wallet balance: {balance_sol:.6} SOL")),
                });

                // 3. Minimum balance
                if balance_sol >= MIN_WALLET_BALANCE_SOL {
                    checks.push(CheckResult {
                        name: "minimum_balance",
                        passed: true,
                        message: None,
                    });
                } else {
                    checks.push(CheckResult {
                        name: "minimum_balance",
                        passed: false,
                        message: Some(format!(
                            "wallet has {balance_sol:.6} SOL, need at least {MIN_WALLET_BALANCE_SOL} SOL"
                        )),
                    });
                }
            }
            Err(e) => {
                checks.push(CheckResult {
                    name: "wallet_exists",
                    passed: false,
                    message: Some(format!("failed to fetch balance: {e}")),
                });
                checks.push(CheckResult {
                    name: "minimum_balance",
                    passed: false,
                    message: Some("wallet unreachable".into()),
                });
            }
        }
    }

    ReadinessReport { checks }
}

/// Load a Solana keypair from a JSON keypair file.
///
/// The file must be a JSON array of 64 integers (the seed bytes).
/// If `path` is `None`, defaults to `~/.config/solana/id.json`.
pub fn load_keypair_from_file(path: Option<&std::path::Path>) -> anyhow::Result<Keypair> {
    let path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_keypair_path);

    let data = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read keypair file {path:?}: {e}"))?;

    let bytes: Vec<u8> = serde_json::from_str(&data)
        .map_err(|e| anyhow::anyhow!("invalid keypair JSON in {path:?}: {e}"))?;

    anyhow::ensure!(
        bytes.len() == 64,
        "keypair file {path:?} has {} bytes, expected 64",
        bytes.len(),
    );

    Keypair::try_from(bytes.as_slice())
        .map_err(|e| anyhow::anyhow!("invalid keypair in {path:?}: {e}"))
}

fn default_keypair_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    std::path::Path::new(&home)
        .join(".config")
        .join("solana")
        .join("id.json")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insufficient_balance_display() {
        let err = InsufficientBalance {
            balance_sol: 0.5,
            required_sol: 1.0,
        };
        let msg = err.to_string();
        assert!(msg.contains("0.500000"), "{msg}");
        assert!(msg.contains("1.000000"), "{msg}");
    }

    #[test]
    fn insufficient_balance_is_error() {
        let err = InsufficientBalance {
            balance_sol: 0.0,
            required_sol: 1.0,
        };
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn real_buy_request_debug_clone() {
        let req = RealBuyRequest {
            mint: "So11111111111111111111111111111111111111112".into(),
            symbol: "SOL".into(),
            name: "Solana".into(),
            amount_sol: 0.5,
            slippage_basis_points: Some(300),
            priority_fee_lamports: Some(1000),
            narrative: "test".into(),
        };
        let cloned = req.clone();
        assert_eq!(req.mint, cloned.mint);
        assert_eq!(req.amount_sol, cloned.amount_sol);
        let _ = format!("{req:?}");
    }

    #[test]
    fn real_sell_request_defaults() {
        let req = RealSellRequest {
            trade_id: 42,
            slippage_basis_points: None,
            priority_fee_lamports: None,
            close_reason: Some("stop loss".into()),
        };
        assert_eq!(req.trade_id, 42);
        assert_eq!(req.close_reason.as_deref(), Some("stop loss"));
        let _ = format!("{req:?}");
    }

    #[test]
    fn bs64_decode_rejects_invalid() {
        assert!(bs64_decode("not-valid-base64!").is_err());
    }

    #[test]
    fn bs64_decode_handles_valid() {
        let decoded = bs64_decode("SGVsbG8=").unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn lampotrs_per_sol_constant() {
        assert_eq!(LAMPORTS_PER_SOL, 1_000_000_000.0);
        assert_eq!(TX_FEE_BUFFER_SOL, 0.01);
    }
}

use std::time::Duration;

use serde::Deserialize;

pub const APP_USER_AGENT: &str = "coins-rust/0.1.0";

pub fn http_client(timeout_secs: u64) -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
}

#[derive(Clone, Default, Deserialize)]
pub struct Config {
    pub core_database: String,
    pub otlp_endpoint: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub solana_rpc_url: Option<String>,
    pub dexscreener_token_profiles_url: Option<String>,
    pub spike_threshold: Option<f64>,
    pub baseline_windows: Option<u64>,
    pub new_narrative_min_count: Option<i32>,
    pub scanner_poll_interval_secs: Option<u64>,
    pub cex_poll_interval_secs: Option<u64>,
    pub cex_binance_tickers_url: Option<String>,
    pub cex_coinbase_assets_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_poll_interval_secs: Option<u64>,
    pub distiller_poll_interval_secs: Option<u64>,
    pub dexscreener_search_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }

    pub fn solana_rpc_url(&self) -> String {
        self.solana_rpc_url
            .clone()
            .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string())
    }

    pub fn token_profiles_url(&self) -> String {
        self.dexscreener_token_profiles_url
            .clone()
            .unwrap_or_else(|| "https://api.dexscreener.com/token-profiles/latest/v1".to_string())
    }

    pub fn spike_threshold(&self) -> f64 {
        self.spike_threshold.unwrap_or(2.0)
    }

    pub fn baseline_windows(&self) -> usize {
        self.baseline_windows.unwrap_or(6) as usize
    }

    pub fn new_narrative_min_count(&self) -> i32 {
        self.new_narrative_min_count.unwrap_or(3)
    }

    pub fn scanner_poll_interval(&self) -> u64 {
        self.scanner_poll_interval_secs.unwrap_or(300)
    }

    pub fn cex_poll_interval(&self) -> u64 {
        self.cex_poll_interval_secs.unwrap_or(300)
    }

    pub fn cex_binance_tickers_url(&self) -> String {
        self.cex_binance_tickers_url
            .clone()
            .unwrap_or_else(|| cex::BINANCE_TICKERS_URL.to_string())
    }

    pub fn cex_coinbase_assets_url(&self) -> String {
        self.cex_coinbase_assets_url
            .clone()
            .unwrap_or_else(|| cex::COINBASE_ASSETS_URL.to_string())
    }

    pub fn telegram_bot_token(&self) -> String {
        self.telegram_bot_token.clone().unwrap_or_default()
    }

    pub fn telegram_poll_interval(&self) -> u64 {
        self.telegram_poll_interval_secs.unwrap_or(60)
    }

    pub fn distiller_poll_interval(&self) -> u64 {
        self.distiller_poll_interval_secs.unwrap_or(300)
    }

    pub fn dexscreener_search_url(&self) -> String {
        self.dexscreener_search_url
            .clone()
            .unwrap_or_else(|| "https://api.dexscreener.com/latest/dex/search".to_string())
    }
}

pub mod scanner {
    pub const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1q1frNYHuzeLXfQM9H24wFSUt1Mp8";
    pub const PUMPFUN_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    pub const SOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
    pub const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const USDT_ADDRESS: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    pub const POOL_DATA_SIZE: u64 = 752;
    pub const COIN_MINT_OFFSET: usize = 144;
    pub const PC_MINT_OFFSET: usize = 176;
    pub const PUMPFUN_BC_SIZE: u64 = 105;
}

pub mod cex {
    pub const BINANCE_TICKERS_URL: &str =
        "https://api.coingecko.com/api/v3/exchanges/binance/tickers";
    pub const COINBASE_ASSETS_URL: &str = "https://www.coinbase.com/api/v2/assets/info";
}

use serde::Deserialize;

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
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }
}

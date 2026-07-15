use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub core_database: String,
    pub otlp_endpoint: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }
}

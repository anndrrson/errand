use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub solana_rpc_url: String,
    pub anthropic_api_key: String,
    pub tavily_api_key: Option<String>,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Build DATABASE_URL from individual params if available (avoids # encoding issues)
        let database_url = if let (Ok(host), Ok(user), Ok(pass), Ok(db)) = (
            std::env::var("DB_HOST"),
            std::env::var("DB_USER"),
            std::env::var("DB_PASSWORD"),
            std::env::var("DB_NAME"),
        ) {
            let encoded_pass = urlencoding::encode(&pass);
            format!("postgresql://{user}:{encoded_pass}@{host}:5432/{db}?sslmode=require")
        } else {
            std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?
        };

        Ok(Self {
            database_url,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")
                .context("ANTHROPIC_API_KEY must be set")?,
            tavily_api_key: std::env::var("TAVILY_API_KEY").ok(),
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        })
    }
}

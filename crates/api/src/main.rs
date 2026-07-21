mod error;
mod extractor;
mod openapi;
mod routes;

use axum_server::tls_rustls::RustlsConfig;
use coins_app::App;
use coins_config::Config;
use std::net::SocketAddr;
use std::process::exit;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::error::not_found;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--openapi") {
        println!("{}", openapi::ApiDoc::openapi().to_pretty_json()?);
        return Ok(());
    }

    tracing_subscriber::fmt().init();

    let config = Config::from_env()?;
    let app = App::init(&config).await?;

    let host = config.host.clone().unwrap_or("0.0.0.0".into());
    let port = config.port.unwrap_or(3000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let router = routes::router()
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
        .layer(CorsLayer::permissive())
        .with_state(app.state)
        .fallback(not_found);

    let (tls_cert, tls_key) = (config.tls_cert_path.clone(), config.tls_key_path.clone());
    match (&tls_cert, &tls_key) {
        (Some(cert), Some(key)) => {
            let tls_config = RustlsConfig::from_pem_file(cert, key).await?;
            info!("listening on https://{}", addr);
            axum_server::bind_rustls(addr, tls_config)
                .serve(router.into_make_service())
                .await?;
        }
        _ => {
            error!("Generate a key before continuing, see scripts");
            exit(1);
        }
    }

    Ok(())
}

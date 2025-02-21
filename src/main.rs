use axum::{http::Response, response::IntoResponse, routing::get, Router};
use prometheus::{labels, opts, register_gauge, Encoder, Gauge, TextEncoder};
use twillo::TwilloAPI;
mod twillo;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    dotenv::dotenv().ok();

    let app = Router::new()
        .route("/", get(metrics))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    return "alive";
}

lazy_static::lazy_static! {
    static ref TOTAL_VERIFICATIONS: Gauge = register_gauge!(opts!(
        "total_verifications",
        "Total number of verifications",
        labels! {"handler" => "all",}
    )).unwrap();

    static ref CONVERTED_VERIFICATIONS: Gauge = register_gauge!(opts!(
        "converted_verifications",
        "Converted Verifications",
        labels! {"handler" => "all",}
    )).unwrap();

    static ref FAILED_VERIFICATIONS: Gauge = register_gauge!(opts!(
        "failed_verifications",
        "Failed verifications",
        labels! {"handler" => "all",}
    )).unwrap();
}

async fn metrics() -> impl IntoResponse {
    let app_name = std::env::var("APP_NAME").unwrap();
    let sid = std::env::var("SID").unwrap();
    let token = std::env::var("TOKEN").unwrap();

    let twillo = TwilloAPI::new(&app_name, &sid, &token);
    let encoder = TextEncoder::new();
    let summary = twillo.get_verification_summary("", None).await.unwrap();

    TOTAL_VERIFICATIONS.set(summary.total_attempts as f64);
    FAILED_VERIFICATIONS.set(summary.total_unconverted as f64);
    CONVERTED_VERIFICATIONS.set(summary.total_converted as f64);
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    return buffer;
}

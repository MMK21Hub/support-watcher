use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use argh::FromArgs;
use metrics::{describe_gauge, gauge};
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};
use serde::Deserialize;

#[derive(FromArgs)]
/// Prometheus metric exporter for the Helper Heidi bot
struct SupportWatcher {
    /// port to bind to
    #[argh(option, default = "9000")]
    port: u16,
}

const HEALTH_API: &str = "https://nephthys.hackclub.com/health";
const STATS_API: &str = "https://nephthys.hackclub.com/api/stats";

#[derive(Deserialize)]
struct HealthData {
    healthy: bool,
    slack: bool,
    database: bool,
}

fn main() -> Result<(), BuildError> {
    // CLI args
    let support_watcher: SupportWatcher = argh::from_env();

    // Set up Prometheus exporter
    let builder = PrometheusBuilder::new();
    let listen_on = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), support_watcher.port);
    match builder.with_http_listener(listen_on).install() {
        Ok(_) => {}
        Err(BuildError::FailedToCreateHTTPListener(message)) => {
            eprintln!("Failed to attach to {}", listen_on);
            return Err(BuildError::FailedToCreateHTTPListener(message));
        }
        Err(error) => return Err(error),
    };
    println!("Prometheus exporter listening on {}", listen_on);
    println!(
        "Try accessing http://localhost:{}/metrics",
        support_watcher.port
    );

    // Initialize metrics
    let nephthys_overall_health = gauge!("nephthys_overall_health");
    describe_gauge!("nephthys_overall_health", "Whether Helper Heidi is healthy");
    let nephthys_slack_health = gauge!("nephthys_slack_health");
    describe_gauge!(
        "nephthys_slack_health",
        "Whether Helper Heidi is connected to the Slack"
    );
    let nephthys_database_health = gauge!("nephthys_database_health");
    describe_gauge!(
        "nephthys_database_health",
        "Whether Helper Heidi is connected to her database"
    );

    // Initialize HTTP client
    let client = reqwest::blocking::Client::new();

    loop {
        // Update Helper Heidi's health
        let health_data: HealthData = match client.get(HEALTH_API).send() {
            Err(error) => {
                eprintln!("Failed to fetch health data: {}", error);
                continue;
            }
            Ok(response) => match response.json() {
                Err(error) => {
                    eprintln!("Failed to parse health data: {}", error);
                    continue;
                }
                Ok(data) => data,
            },
        };

        nephthys_overall_health.set(if health_data.healthy { 1 } else { 0 });
        nephthys_slack_health.set(if health_data.slack { 1 } else { 0 });
        nephthys_database_health.set(if health_data.database { 1 } else { 0 });
    }
}

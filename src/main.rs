use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::sleep,
    time::Duration,
};

use argh::FromArgs;
use metrics::{counter, describe_gauge, gauge};
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

#[derive(Deserialize, Debug)]
struct HealthData {
    healthy: bool,
    slack: bool,
    database: bool,
}

#[derive(Deserialize, Debug)]
struct StatsData {
    total_tickets: u64,
    total_open: u64,
    total_in_progress: u64,
    total_closed: u64,
    total_top_3_users_with_closed_tickets: Vec<UserStatsData>,
    prev_day_total: u64,
    prev_day_open: u64,
    prev_day_in_progress: u64,
    prev_day_closed: u64,
    prev_day_top_3_users_with_closed_tickets: Vec<UserStatsData>,
}

#[derive(Debug, Deserialize)]
struct UserStatsData {
    user_id: u64,
    slack_id: String,
    closed_ticket_count: u64,
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
    let nephthys_overall_health = gauge!("nephthys_overall_up");
    describe_gauge!("nephthys_overall_up", "Whether Helper Heidi is healthy");
    let nephthys_slack_health = gauge!("nephthys_slack_up");
    describe_gauge!(
        "nephthys_slack_up",
        "Whether Helper Heidi is connected to the Slack"
    );
    let nephthys_database_health = gauge!("nephthys_database_up");
    describe_gauge!(
        "nephthys_database_up",
        "Whether Helper Heidi is connected to her database"
    );
    let tickets_counter = counter!("nephthys_tickets_total");
    let open_tickets_gauge = gauge!("nephthys_open_tickets");
    let in_progress_tickets_gauge = gauge!("nephthys_in_progress_tickets");
    let closed_tickets_gauge = gauge!("nephthys_closed_tickets");

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

        // Update the statistic metrics!
        let stats_data: StatsData = match client.get(STATS_API).send() {
            Err(error) => {
                eprintln!("Failed to fetch stats data: {}", error);
                continue;
            }
            Ok(response) => match response.json() {
                Err(error) => {
                    eprintln!("Failed to parse stats data: {}", error);
                    continue;
                }
                Ok(data) => data,
            },
        };
        tickets_counter.absolute(stats_data.total_tickets);
        open_tickets_gauge.set(stats_data.total_open as f64);
        in_progress_tickets_gauge.set(stats_data.total_in_progress as f64);
        closed_tickets_gauge.set(stats_data.total_closed as f64);
        println!(
            "Latest tickets data: {} open, {} in progress, {} closed",
            stats_data.total_open, stats_data.total_in_progress, stats_data.total_closed
        );

        // Wait a bit so that we don't spam the API
        sleep(Duration::from_secs(60));
    }
}

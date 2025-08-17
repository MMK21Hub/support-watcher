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
    average_hang_time_minutes: f64,
    prev_day_total: u64,
    prev_day_open: u64,
    prev_day_in_progress: u64,
    prev_day_closed: u64,
    prev_day_top_3_users_with_closed_tickets: Vec<UserStatsData>,
    prev_day_average_hang_time_minutes: f64,
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

    // Describe some of the metrics
    describe_gauge!("nephthys_overall_up", "Whether Helper Heidi is healthy");
    describe_gauge!(
        "nephthys_slack_up",
        "Whether Helper Heidi is connected to the Slack"
    );
    describe_gauge!(
        "nephthys_database_up",
        "Whether Helper Heidi is connected to her database"
    );

    // Grab a HTTP client that we can reuse
    let client = reqwest::blocking::Client::new();

    loop {
        // Update Helper Heidi's health
        let health_data: HealthData = match client.get(HEALTH_API).send() {
            Err(error) => {
                eprintln!("Failed to fetch health data: {:?}", error);
                sleep(Duration::from_secs(30));
                continue;
            }
            Ok(response) => match response.json() {
                Err(error) => {
                    eprintln!("Failed to parse health data: {:?}", error);
                    sleep(Duration::from_secs(30));
                    continue;
                }
                Ok(data) => data,
            },
        };
        gauge!("nephthys_overall_up").set(if health_data.healthy { 1 } else { 0 });
        gauge!("nephthys_slack_up").set(if health_data.slack { 1 } else { 0 });
        gauge!("nephthys_database_up").set(if health_data.database { 1 } else { 0 });

        // Update the statistic metrics!
        let stats_data: StatsData = match client.get(STATS_API).send() {
            Err(error) => {
                eprintln!("Failed to fetch stats data: {:?}", error);
                sleep(Duration::from_secs(30));
                continue;
            }
            Ok(response) => match response.json() {
                Err(error) => {
                    eprintln!("Failed to parse stats data: {:?}", error);
                    sleep(Duration::from_secs(30));
                    continue;
                }
                Ok(data) => data,
            },
        };
        counter!("nephthys_tickets_total").absolute(stats_data.total_tickets);
        gauge!("nephthys_open_tickets").set(stats_data.total_open as f64);
        gauge!("nephthys_in_progress_tickets").set(stats_data.total_in_progress as f64);
        gauge!("nephthys_closed_tickets").set(stats_data.total_closed as f64);
        gauge!("nephthys_average_hang_time_minutes").set(stats_data.average_hang_time_minutes);
        // Update user-specific stats
        for stats in stats_data.total_top_3_users_with_closed_tickets {
            counter!(
                "nephthys_user_closed_tickets_total",
                "internal_id" => stats.user_id.to_string(),
                "slack_id" => stats.slack_id
            )
            .absolute(stats.closed_ticket_count);
        }
        // Update the "previous 24h" metrics
        gauge!("nephthys_tickets_prev_24h").set(stats_data.prev_day_total as f64);
        gauge!("nephthys_open_tickets_prev_24h").set(stats_data.prev_day_open as f64);
        gauge!("nephthys_in_progress_tickets_prev_24h").set(stats_data.prev_day_in_progress as f64);
        gauge!("nephthys_closed_tickets_prev_24h").set(stats_data.prev_day_closed as f64);
        gauge!("nephthys_average_hang_time_prev_24h_minutes")
            .set(stats_data.prev_day_average_hang_time_minutes);
        // User-specific stats for the previous 24h
        for stats in stats_data.prev_day_top_3_users_with_closed_tickets {
            gauge!(
                "nephthys_user_closed_tickets_prev_24h",
                "internal_id" => stats.user_id.to_string(),
                "slack_id" => stats.slack_id
            )
            .set(stats.closed_ticket_count as f64);
        }

        // Wait a bit so that we don't spam the API
        sleep(Duration::from_secs(30));
    }
}

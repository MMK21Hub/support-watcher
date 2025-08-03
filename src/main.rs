use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use argh::FromArgs;
use metrics::{describe_gauge, gauge};
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};

#[derive(FromArgs)]
/// Prometheus metric exporter for the Helper Heidi bot
struct SupportWatcher {
    /// port to bind to
    #[argh(option, default = "9000")]
    port: u16,
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
    describe_gauge!("nephthys_overall_health", "Whether Helper Heidi is healthy");
    let nephthys_overall_health = gauge!("nephthys_overall_health");
    nephthys_overall_health.set(1);

    loop {}
}

use std::process::Command;
use std::str;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use log::error;
use log::info;

const IP_BIN: &str = "/usr/sbin/ip";
const LONG_ABOUT: &str = "This is a horrible hack to restart frr if it's healthy but no BGP routes exist in the Linux routing table";
const SEARCH_SUBSTR: &str = " bgp ";
const SLEEP_TIME: u64 = 30;
const SYSTEMCTL_BIN: &str = "/usr/bin/systemctl";
const SYSTEMD_SERVICE_NAME: &str = "frr";

/// Clap CLI Args struct with metadata in help output
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = LONG_ABOUT)]
struct Cli {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity<InfoLevel>,
}

// Only checking IPv4 as they all go missing or none do ...
fn have_bgp_routes() -> bool {
    let route_table = match Command::new(IP_BIN).arg("route").arg("show").output() {
        Ok(output) => str::from_utf8(&output.stdout)
            .expect("Unable to convert ip output to str")
            .to_owned(),
        Err(err) => {
            error!(
                "Unable to workout if there are BGP routes: {:#?}. Skipping",
                err
            );
            return true;
        }
    };

    if route_table.contains(SEARCH_SUBSTR) {
        info!("-> Found '{}' routes in the linux table", SEARCH_SUBSTR);
        return true;
    }
    error!("!! Found NO '{}' routes in the linux table", SEARCH_SUBSTR);
    false
}

fn restart_frr() {
    match Command::new(SYSTEMCTL_BIN)
        .arg("restart")
        .arg(SYSTEMD_SERVICE_NAME)
        .status()
    {
        Ok(exit_code) => info!(
            "Successfully restarted {} (Returned {})",
            SYSTEMD_SERVICE_NAME, exit_code
        ),
        Err(err) => error!("Unable to restart {}: {:#?}", SYSTEMD_SERVICE_NAME, err),
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("Starting to monitor bgp routes in the linux routing table");

    loop {
        info!(
            "Checking the routing table has '{}' routes ...",
            SEARCH_SUBSTR
        );
        if !have_bgp_routes() {
            restart_frr();
        }

        info!("sleeping for {} seconds ...", SLEEP_TIME);
        std::thread::sleep(Duration::new(SLEEP_TIME, 0));
    }
}

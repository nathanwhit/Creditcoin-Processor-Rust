#![allow(
    clippy::suspicious_operation_groupings,
    clippy::try_err,
    clippy::wrong_self_convention
)]
#![deny(unused_must_use)]
#![cfg_attr(test, allow(dead_code, unused_imports))]

pub mod ext;
pub mod handler;

#[allow(non_snake_case)]
pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/cc.protos.rs"));
}

use anyhow::Result;
use clap::{clap_app, crate_authors, crate_description, crate_version};
use log::LevelFilter;
use sawtooth_sdk::processor::TransactionProcessor;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_tree::HierarchicalLayer;

const DEFAULT_ENDPOINT: &str = "tcp://localhost:4004";
const DEFAULT_GATEWAY: &str = "tcp://localhost:55555";

fn setup_logs(verbose_count: u64) -> Result<()> {
    let level = match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", level.as_str().to_lowercase())
    }

    // tracing_subscriber::FmtSubscriber::default().init();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_ansi(true)
                .with_indent_lines(true)
                .with_targets(true),
        )
        .init();

    // Dispatch::new()
    //     .level(level)
    //     .level_for("sawtooth_sdk::consensus::zmq_driver", LevelFilter::Error)
    //     .level_for("sawtooth_sdk::messaging::zmq_stream", LevelFilter::Error)
    //     .format(fmt_log)
    //     .chain(stdout())
    //     .apply()?;

    Ok(())
}

#[cfg(not(all(test, feature = "mock")))]
#[tracing::instrument]
fn main() -> Result<()> {
    let matches = clap_app!(consensus_engine =>
      (version: crate_version!())
      (author: crate_authors!())
      (about: crate_description!())
      (@arg endpoint: -E --endpoint +takes_value "connection endpoint for validator")
      (@arg gateway: -G --gateway +takes_value "connection endpoint for gateway")
      (@arg old: --old "use compatibility")
      (@arg verbose: -v --verbose +multiple "increase output verbosity")
    )
    .get_matches();

    let endpoint: &str = matches.value_of("endpoint").unwrap_or(DEFAULT_ENDPOINT);
    let gateway: &str = matches.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);

    setup_logs(matches.occurrences_of("verbose"))?;

    info!("ccprocessor-rust ({})", env!("CARGO_PKG_VERSION"));

    info!("ccprocessor-rust connecting to {} ...", endpoint);
    let mut processor = TransactionProcessor::new(endpoint);

    info!("ccprocessor-rust connecting to gateway {} ...", gateway);
    let handler = handler::CCTransactionHandler::new(gateway);

    processor.add_handler(&handler);
    processor.start();

    info!("ccprocessor-rust exiting ...");

    Ok(())
}

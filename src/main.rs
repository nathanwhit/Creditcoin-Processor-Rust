use anyhow::Result;
use std::fmt::Arguments;
use std::io::stdout;

use clap::{clap_app, crate_authors, crate_description, crate_version};
use log::info;

use ccprocessor_rust::{DEFAULT_ENDPOINT, DEFAULT_GATEWAY};
use sawtooth_sdk::processor::TransactionProcessor;

#[cfg(not(all(test, feature = "mock")))]
fn main() -> Result<()> {
    let matches = clap_app!(consensus_engine =>
      (version: crate_version!())
      (author: crate_authors!())
      (about: crate_description!())
      (@arg endpoint: -E --endpoint +takes_value "connection endpoint for validator")
      (@arg gateway: -G --gateway +takes_value "connection endpoint for gateway")
      (@arg verbose: -v --verbose +multiple "increase output verbosity")
    )
    .get_matches();

    let endpoint: &str = matches.value_of("endpoint").unwrap_or(DEFAULT_ENDPOINT);
    let gateway: &str = matches.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);

    ccprocessor_rust::setup_logs(matches.occurrences_of("verbose"))?;

    info!("ccprocessor-rust ({})", env!("CARGO_PKG_VERSION"));

    info!("ccprocessor-rust connecting to {} ...", endpoint);
    let mut processor = TransactionProcessor::new(endpoint);

    info!("ccprocessor-rust connecting to gateway {} ...", gateway);
    let handler = ccprocessor_rust::handler::CCTransactionHandler::new(gateway);

    processor.add_handler(&handler);
    processor.start();

    info!("ccprocessor-rust exiting ...");

    Ok(())
}

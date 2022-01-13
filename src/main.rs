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
use fern::FormatCallback;
use fern::{colors::Color, Dispatch};
use log::LevelFilter;
use log::Record;
use std::fmt::Arguments;
use std::io::stdout;

use clap::Parser;
use fern::colors::ColoredLevelConfig;
use log::info;

use sawtooth_sdk::processor::TransactionProcessor;

const DEFAULT_ENDPOINT: &str = "tcp://localhost:4004";
const DEFAULT_GATEWAY: &str = "tcp://localhost:55555";

const TIME_FMT: &str = "%Y-%m-%d %H:%M:%S.%3f";

fn fmt_log(out: FormatCallback, message: &Arguments, record: &Record) {
    let module: &str = record
        .module_path_static()
        .or_else(|| record.module_path())
        .unwrap_or("???");
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::BrightMagenta);
    out.finish(format_args!(
        "[{} {:<5} {}] {}",
        chrono::Utc::now().format(TIME_FMT),
        colors.color(record.level()),
        module,
        message
    ))
}

fn setup_logs(verbose_count: u64) -> Result<()> {
    let level = match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Dispatch::new()
        .level(level)
        .level_for(
            "sawtooth_sdk_creditcoin::messaging::zmq_stream",
            LevelFilter::Error,
        )
        .format(fmt_log)
        .chain(stdout())
        .apply()?;

    Ok(())
}

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short('E'), long, default_value_t = String::from(DEFAULT_ENDPOINT), help = "connection endpoint for validator")]
    endpoint: String,

    #[clap(short('G'), long, default_value_t = String::from(DEFAULT_GATEWAY), help = "connection endpoint for gateway")]
    gateway: String,

    #[clap(long, help = "use compatibility for Creditcoin 1.7")]
    old: bool,

    #[clap(
        short('v'),
        long,
        parse(from_occurrences),
        help = "increase output verbosity"
    )]
    verbose: u64,
}

#[cfg(not(all(test, feature = "mock")))]
fn main() -> Result<()> {
    let args = Args::parse();

    setup_logs(args.verbose)?;

    info!("ccprocessor-rust ({})", env!("CARGO_PKG_VERSION"));

    info!("ccprocessor-rust connecting to {} ...", args.endpoint);
    let mut processor = TransactionProcessor::new(&args.endpoint);

    info!("ccprocessor-rust connecting to gateway {} ...", args.gateway);
    let handler = handler::CCTransactionHandler::new(args.gateway);

    processor.add_handler(&handler);
    processor.start();

    info!("ccprocessor-rust exiting ...");

    Ok(())
}

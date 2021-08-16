#![allow(
    clippy::suspicious_operation_groupings,
    clippy::try_err,
    clippy::wrong_self_convention
)]
#![deny(unused_must_use)]
#![cfg_attr(test, allow(dead_code, unused_imports))]

pub mod ext;
pub mod handler;
pub mod test_utils;

#[allow(non_snake_case)]
pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/cc.protos.rs"));
}

use anyhow::Result;
use fern::colors::ColoredLevelConfig;
use fern::FormatCallback;
use fern::{colors::Color, Dispatch};
use log::LevelFilter;
use log::Record;
use std::fmt::Arguments;

use std::io::stdout;

pub const DEFAULT_ENDPOINT: &str = "tcp://localhost:4004";
pub const DEFAULT_GATEWAY: &str = "tcp://localhost:55555";

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

pub fn setup_logs(verbose_count: u64) -> Result<()> {
    let level = match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Dispatch::new()
        .level(level)
        .level_for("sawtooth_sdk::consensus::zmq_driver", LevelFilter::Error)
        .level_for("sawtooth_sdk::messaging::zmq_stream", LevelFilter::Error)
        .level_for("bollard", LevelFilter::Error)
        .level_for("mio", LevelFilter::Error)
        .level_for("want", LevelFilter::Error)
        .level_for("ureq", LevelFilter::Warn)
        .format(fmt_log)
        .chain(stdout())
        .apply()?;

    Ok(())
}

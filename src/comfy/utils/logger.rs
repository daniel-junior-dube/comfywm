/*
.##.......####....####....####...######..#####..
.##......##..##..##......##......##......##..##.
.##......##..##..##.###..##.###..####....#####..
.##......##..##..##..##..##..##..##......##..##.
.######...####....####....####...######..##..##.
................................................
*/

use chrono::prelude::Utc;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use wlroots::utils::wlr_log_importance;
use wlroots::utils::LogCallback;
use wlroots::utils::LogVerbosity;

/// Generates a string for a log output file. The output directory is `/tmp` and the filename is built from the current datetime, prefix by 'comfywm_'.
fn generate_log_output_file_path() -> String {
	let datetime = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();
	format!("/tmp/comfywm_{}.log", datetime)
}

/// Generates the default encoder used by all appender to format the output.
fn generate_default_encoder() -> PatternEncoder {
	PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - [{h({l})}] {m}{n}")
}

/// Generates the default file appender which uses the default encoder. Creates the output file from the provided file path and outputs to it.
fn generate_file_appender(log_output_file_path: &str) -> FileAppender {
	FileAppender::builder()
		.encoder(Box::new(generate_default_encoder()))
		.build(&log_output_file_path)
		.unwrap()
}

/// Generates the default console appender which uses the default encoder.
fn generate_console_appender() -> ConsoleAppender {
	ConsoleAppender::builder()
		.encoder(Box::new(generate_default_encoder()))
		.build()
}

/// Returns a callback mIntercepts and logs all logs from wlroots-rs into our logging
pub fn generate_wlroots_rs_log_callback() -> LogCallback {
	|level: LogVerbosity, message: String| match level {
		wlr_log_importance::WLR_SILENT | wlr_log_importance::WLR_DEBUG | wlr_log_importance::WLR_LOG_IMPORTANCE_LAST => {
			debug!("{}", message)
		}
		wlr_log_importance::WLR_ERROR => error!("{}", message),
		wlr_log_importance::WLR_INFO => info!("{}", message),
	}
}

/// Generates the default log4rs config which as a default console appender and a default file appender.
/// Both logs to the debug level (Note: Debug level shall be used until we have a stable release).
pub fn generate_log4rs_config() -> Config {
	let log_output_file_path = generate_log_output_file_path();
	let stdout = generate_console_appender();
	let requests = generate_file_appender(&log_output_file_path);
	Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.appender(Appender::builder().build("requests", Box::new(requests)))
		.logger(Logger::builder().build("app::backend::db", LevelFilter::Debug))
		.logger(
			Logger::builder()
				.appender("requests")
				.additive(false)
				.build("app::requests", LevelFilter::Debug),
		).build(
			Root::builder()
				.appender("requests")
				.appender("stdout")
				.build(LevelFilter::Debug),
		).unwrap()
}

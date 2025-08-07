use diesel::query_dsl::InternalJoinDsl;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

pub fn init(logs_dir: &str) -> WorkerGuard {
    let log_prefix = "ipmsg-rs";
    let log_suffix = "log";
    let max_log_files = 14;
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(max_log_files)
        .filename_prefix(log_prefix)
        .filename_suffix(log_suffix)
        .build(&logs_dir)
        .expect("initializing rolling file appender failed");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let format_for_humans = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .compact();

    let log_level_filter = std::env::var("LOG_LEVEL")
        .unwrap_or("info".to_string())
        .to_lowercase()
        .parse()
        .unwrap_or(LevelFilter::INFO);

    let use_colors_in_logs = cfg!(not(feature = "windows"));
    let tmp_path: PathBuf = logs_dir.into();
    let subscriber = tracing_subscriber::registry()
        .with(
            // subscriber for https://github.com/tokio-rs/console
            console_subscriber::ConsoleLayer::builder()
                .server_addr((Ipv4Addr::LOCALHOST, 12345))
                .retention(Duration::from_secs(3600)) // 1h
                .publish_interval(Duration::from_secs(1))
                .recording_path(tmp_path.join("tokio-console"))
                .spawn(),
        )
        .with(
            // subscriber that writes spans to a file
            tracing_subscriber::fmt::layer()
                .event_format(format_for_humans.clone())
                .with_ansi(false)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(file_writer)
                .with_filter(log_level_filter),
        );
    set_global_default(
        subscriber.with(
            // subscriber that writes spans to stdout
            tracing_subscriber::fmt::layer()
                .event_format(format_for_humans)
                .with_ansi(use_colors_in_logs)
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(log_level_filter),
        ),
    )
    .expect("failed to set subscriber");
    guard
}

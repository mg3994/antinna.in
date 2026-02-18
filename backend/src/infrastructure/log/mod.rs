use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_appender::rolling;
use crate::infrastructure::config::{LogConfig,FORMAT_PRETTY,FORMAT_COMPACT,FORMAT_JSON};



/// Initialize tracing log.
///
/// Caller should hold the guard.
pub fn init(config: &LogConfig) -> WorkerGuard {
    // Tracing appender init.
    let file_appender = match &*config.rolling {
        "minutely" => rolling::minutely(&config.directory, &config.file_name),
        "hourly" => rolling::hourly(&config.directory, &config.file_name),
        "daily" => rolling::daily(&config.directory, &config.file_name),
        "never" => rolling::never(&config.directory, &config.file_name),
        _ => rolling::never(&config.directory, &config.file_name),
    };
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Tracing subscriber init.
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(tracing_subscriber::EnvFilter::new(&config.filter_level)),
        )
        .with_ansi(config.with_ansi);

    if config.format == FORMAT_PRETTY {
        let subscriber = subscriber.event_format(
            fmt::format()
                .pretty()
                .with_level(config.with_level)
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_source_location(config.with_source_location),
        );
        if config.stdout {
            subscriber.with_writer(std::io::stdout).init();
        } else {
            subscriber.with_writer(file_writer).init();
        }
    } else if config.format == FORMAT_COMPACT {
        let subscriber = subscriber.event_format(
            fmt::format()
                .compact()
                .with_level(config.with_level)
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_source_location(config.with_source_location),
        );
        if config.stdout {
            subscriber.with_writer(std::io::stdout).init();
        } else {
            subscriber.with_writer(file_writer).init();
        }
    } else if config.format == FORMAT_JSON {
        let subscriber = subscriber.event_format(
            fmt::format()
                .json()
                .with_level(config.with_level)
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_source_location(config.with_source_location),
        );
        if config.stdout {
            subscriber.json().with_writer(std::io::stdout).init();
        } else {
            subscriber.json().with_writer(file_writer).init();
        }
    } else {
        // FORMAT_FULL or fallback
        let subscriber = subscriber.event_format(
            fmt::format()
                .with_level(config.with_level)
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_source_location(config.with_source_location),
        );
        if config.stdout {
            subscriber.with_writer(std::io::stdout).init();
        } else {
            subscriber.with_writer(file_writer).init();
        }
    }

    guard
}

use anyhow::Result;
use std::path::Path;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::logging::{LogConfig, Rotation};


pub fn init_logging(config: &LogConfig) -> Result<tracing_appender::non_blocking::WorkerGuard> {

    let file_path = Path::new(&config.file);
    let directory = file_path.parent().unwrap_or_else(|| Path::new("."));
    let filename = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("app.log");

    // 创建目录（如果不存在）
    std::fs::create_dir_all(directory)?;

    let file_appender = match config.rotation.daily {
        Rotation::Hourly => tracing_appender::rolling::hourly(directory, filename),
        Rotation::Daily => tracing_appender::rolling::daily(directory, filename),
        Rotation::Never => tracing_appender::rolling::never(directory, filename),
    };

    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::new(&config.level);

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking_writer)
            .with_ansi(false)
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true),
        )
        .init();

    info!(
        level = %config.level,
        file = ?config.file,
        "日志系统初始化完成"
    );

    Ok(guard)
}
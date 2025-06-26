use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .without_time()
                .with_target(false),
        )
        .with({
            EnvFilter::builder()
                .with_default_directive(LevelFilter::OFF.into())
                .from_env()?
                .add_directive(
                    format!(
                        "{}={}",
                        env!("CARGO_PKG_NAME"),
                        std::env::var(format!("{}_LOG", env!("CARGO_PKG_NAME").to_uppercase()))
                            .unwrap_or_else(|_| "off".to_string())
                    )
                    .parse()?,
                )
        })
        .init();
    Ok(())
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        {
            use owo_colors::OwoColorize;
            println!("{} {}", "✓".green(), format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        {
            use owo_colors::OwoColorize;
            println!("{} {}", "⚠".yellow(), format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
       {
            use owo_colors::OwoColorize;
            println!("{} {}", "ℹ".blue(), format!($($arg)*))
       }
    };
}

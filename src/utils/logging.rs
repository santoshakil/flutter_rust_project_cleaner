use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

pub fn init_logging(verbosity: u8, quiet: bool, no_color: bool) {
    if no_color {
        colored::control::set_override(false);
    }
    
    if quiet {
        return;
    }
    
    let level = match verbosity {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };
    
    let env_filter = EnvFilter::new(format!("frpc={},error", level));
    
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .without_time()
        .compact()
        .init();
}
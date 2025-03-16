use std::env;
use std::sync::Once;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

static INIT: Once = Once::new();
static FLUTTER_RUST_EMBEDDER_LOG_ENV_VAR: &str = "FLUTTER_RUST_EMBEDDER_LOG";
static FLUTTER_RUST_EMBEDDER_PROFILE_ENV_VAR: &str = "FLUTTER_RUST_EMBEDDER_PROFILE";

/// Initialize tracing for the library
///
/// This should be called early in your library's execution.
/// It will only initialize tracing if:
/// 1. The environment variable FLUTTER_RUST_EMBEDDER_LOG is set
/// 2. Tracing hasn't already been initialized by this function
///
/// Example environment variable values:
/// - FLUTTER_RUST_EMBEDDER_LOG=debug (sets the log level for your crate)
/// - FLUTTER_RUST_EMBEDDER_LOG=my_crate=debug (explicit namespace)
/// - FLUTTER_RUST_EMBEDDER_LOG=my_crate=debug,other_crate=warn (multiple directives)
pub fn init_tracing() {
    // INIT.call_once(|| {
    //     // initialize the env filter for the fmt subscriber
    //     let flutter_fmt_env_var = env::var(FLUTTER_RUST_EMBEDDER_LOG_ENV_VAR).ok();

    //     let flutter_fmt_env_filter = flutter_fmt_env_var.and_then(|env_var_str| {
    //         // Try to parse the environment variable as an EnvFilter
    //         EnvFilter::try_new(env_var_str).ok()
    //     });

    //     // if flutter env filter is not set, use the default
    //     let fmt_env_filter =
    //         flutter_fmt_env_filter.unwrap_or_else(|| EnvFilter::from_default_env());

    //     let flutter_profile_env_var = env::var(FLUTTER_RUST_EMBEDDER_PROFILE_ENV_VAR).ok();

    //     let flutter_profile_env_filter = flutter_profile_env_var.and_then(|env_var_str| {
    //         // Try to parse the environment variable as an EnvFilter
    //         EnvFilter::try_new(env_var_str).ok()
    //     });

    //     let profile_env_filter =
    //         flutter_profile_env_filter.unwrap_or_else(|| EnvFilter::new("off"));

    // let fmt_layer = fmt::layer()
    //     .with_writer(std::io::stdout)
    //     .event_format(Format::default().with_thread_ids(true))
    //     .with_span_events(fmt::format::FmtSpan::FULL);

    // let subscriber = Registry::default().with(fmt_layer);

    // let registry = tracing_subscriber::registry();
    // registry.register_filter(fmt_env_filter);
    // tracing::subscriber::set_global_default(subscriber).unwrap();
    // fmt::Subscriber::builder()
    //     .with_env_filter(env_filter)
    //     .try_init()
    //     .ok();
    // });
}

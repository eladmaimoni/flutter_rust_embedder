use std::env;
use std::sync::Once;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();
static FLUTTER_RUST_EMBEDDER_LOG_ENV_VAR: &str = "FLUTTER_RUST_EMBEDDER_LOG";

/// Initialize tracing for the library
///
/// This should be called early in your library's execution.
/// It will only initialize tracing if:
/// 1. The environment variable MY_CRATE_LOG is set
/// 2. Tracing hasn't already been initialized by this function
///
/// Example environment variable values:
/// - MY_CRATE_LOG=debug (sets the log level for your crate)
/// - MY_CRATE_LOG=my_crate=debug (explicit namespace)
/// - MY_CRATE_LOG=my_crate=debug,other_crate=warn (multiple directives)
pub fn init_tracing() {
    INIT.call_once(|| {
        // Check if the environment variable is set
        if let Ok(env_filter) = env::var(FLUTTER_RUST_EMBEDDER_LOG_ENV_VAR) {
            // Initialize the tracing subscriber with the environment filter
            fmt::Subscriber::builder()
                .with_env_filter(
                    EnvFilter::try_from_default_env()
                        .or_else(|_| EnvFilter::try_new(&env_filter))
                        .unwrap_or_else(|_| {
                            // Default to nothing if env var is malformed
                            EnvFilter::new("off")
                        }),
                )
                .try_init()
                .ok(); // It's okay if initialization fails (might already be initialized elsewhere)

            info!(
                "Tracing initialized for library via environment variable: {}",
                FLUTTER_RUST_EMBEDDER_LOG_ENV_VAR
            );
        } else {
            // Environment variable not set, don't initialize tracing
            // But we might want to set a default no-op subscriber to avoid warnings
            let filter = EnvFilter::new("off");
            fmt::Subscriber::builder()
                .with_env_filter(filter)
                .try_init()
                .ok();
        }
    });
}

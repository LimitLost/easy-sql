use std::sync::Once;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

static INIT: Once = Once::new();

/// Initialize the test logger. This should be called at the beginning of each test.
pub fn init_test_logger() {
    INIT.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_test_writer()
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_span_events(FmtSpan::ACTIVE)
            .compact()
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
    });
}
#[test]
fn setup_logger() {
    init_test_logger();
}

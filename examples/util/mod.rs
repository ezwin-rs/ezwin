use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_log(crate_name: &'static str) {
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_thread_names(true))
    .with(
      tracing_subscriber::filter::Targets::new()
        .with_default(tracing::Level::ERROR)
        .with_targets([
          (crate_name, tracing::Level::TRACE),
          ("witer", tracing::Level::TRACE),
        ]),
    )
    .init();
}

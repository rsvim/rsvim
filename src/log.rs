use tracing::{self, debug, Level};
use tracing_appender;
use tracing_subscriber;

pub fn init(log_level: Level, file_appender: bool, console_appender: bool) {
  let file_appender = tracing_appender::rolling::never(".", "rsvim.log");
  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_level(true)
    .with_ansi(false)
    .with_max_level(log_level)
    .with_writer(file_appender)
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing log");
  debug!("Initialize tracing log");
}

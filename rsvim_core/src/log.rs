//! Logging utils.

use env_filter::Builder;
use jiff::Zoned;

pub const FORMATTER: &str = "%Y-%m-%dT%H:%M:%S%.6f%:z";

/// Initialize file logging, always use file logging.
///
/// It uses `RSVIM_LOG` environment variable to control the logging level.
/// Defaults to `error`.
pub fn init() {
  let env_filter = Builder::from_env("RSVIM_LOG").build();

  let now = Zoned::now();
  let log_name = format!(
    "rsvim_{:0>4}-{:0>2}-{:0>2}_{:0>2}-{:0>2}-{:0>2}-{:0>3}.log",
    now.date().year(),
    now.date().month(),
    now.date().day(),
    now.time().hour(),
    now.time().minute(),
    now.time().second(),
    now.time().millisecond(),
  );

  fern::Dispatch::new()
    .filter(move |metadata| env_filter.enabled(metadata))
    .format(|out, message, record| {
      out.finish(format_args!(
        "[{} {} {}] {}",
        Zoned::now().strftime(FORMATTER),
        record.level(),
        record.target(),
        message
      ))
    })
    .chain(fern::log_file(log_name).unwrap())
    .apply()
    .unwrap();
}

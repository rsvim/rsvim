//! Logging utils.

use log::LevelFilter;

pub const RSVIM_LOG: &str = "RSVIM_LOG";

/// Initialize file logging, always use file logging.
///
/// It uses `RSVIM_LOG` environment variable to control the logging level.
/// Defaults to `error`.
pub fn init() {
  let formatter = "%FT%T%.3f%:z";
  let filter = env_filter::Builder::from_env(RSVIM_LOG).build();

  if filter.filter() >= LevelFilter::Info {
    let now = jiff::Zoned::now();
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
      .filter(move |metadata| filter.enabled(metadata))
      .format(|out, message, record| {
        out.finish(format_args!(
          "{} {:<5} {}:{}| {}",
          jiff::Zoned::now().strftime(formatter),
          record.level(),
          record.target(),
          record.line().unwrap_or(0),
          message
        ))
      })
      .chain(fern::log_file(log_name).unwrap())
      .apply()
      .unwrap();
  }
}

#[cfg(feature = "termcolor")]
use log::Level;
use log::LevelFilter;

pub use chrono::offset::{FixedOffset, Local, Offset, TimeZone, Utc};
use std::borrow::Cow;
#[cfg(feature = "termcolor")]
use termcolor::Color;

#[derive(Debug, Clone, Copy)]
/// Padding to be used for logging the level
pub enum LevelPadding {
    /// Add spaces on the left side
    Left,
    /// Add spaces on the right side
    Right,
    /// Do not pad the level
    Off,
}

#[derive(Debug, Clone, Copy)]
/// Padding to be used for logging the thread id/name
pub enum ThreadPadding {
    /// Add spaces on the left side, up to usize many
    Left(usize),
    /// Add spaces on the right side, up to usize many
    Right(usize),
    /// Do not pad the thread id/name
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Mode for logging the thread name or id or both.
pub enum ThreadLogMode {
    /// Log thread ids only
    IDs,
    /// Log the thread names only
    Names,
    /// If this thread is named, log the name. Otherwise, log the thread id.
    Both,
}

/// Configuration for the Loggers
///
/// All loggers print the message in the following form:
/// `00:00:00 [LEVEL] crate::module: [lib.rs::100] your_message`
/// Every space delimited part except the actual message is optional.
///
/// Pass this struct to your logger to change when these information shall
/// be logged.
///
/// Construct using `Default` or using `ConfigBuilder`
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) time: LevelFilter,
    pub(crate) level: LevelFilter,
    pub(crate) level_padding: LevelPadding,
    pub(crate) thread: LevelFilter,
    pub(crate) thread_log_mode: ThreadLogMode,
    pub(crate) thread_padding: ThreadPadding,
    pub(crate) target: LevelFilter,
    pub(crate) location: LevelFilter,
    pub(crate) time_format: Cow<'static, str>,
    pub(crate) time_offset: FixedOffset,
    pub(crate) time_local: bool,
    pub(crate) filter_allow: Cow<'static, [Cow<'static, str>]>,
    pub(crate) filter_ignore: Cow<'static, [Cow<'static, str>]>,
    #[cfg(feature = "termcolor")]
    pub(crate) level_color: [Option<Color>; 6],
}

/// Builder for the Logger Configurations (`Config`)
///
/// All loggers print the message in the following form:
/// `00:00:00 [LEVEL] crate::module: [lib.rs::100] your_message`
/// Every space delimited part except the actual message is optional.
///
/// Use this struct to create a custom `Config` changing when these information shall
/// be logged. Every part can be enabled for a specific Level and is then
/// automatically enable for all lower levels as well.
///
/// The Result is that the logging gets more detailed the more verbose it gets.
/// E.g. to have one part shown always use `Level::Error`. But if you
/// want to show the source line only on `Trace` use that.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    /// Create a new default ConfigBuilder
    pub fn new() -> ConfigBuilder {
        ConfigBuilder(Config::default())
    }

    /// Set at which level and above (more verbose) the level itself shall be logged (default is Error)
    pub fn set_max_level(&mut self, level: LevelFilter) -> &mut ConfigBuilder {
        self.0.level = level;
        self
    }

    /// Set at which level and  above (more verbose) the current time shall be logged (default is Error)
    pub fn set_time_level(&mut self, time: LevelFilter) -> &mut ConfigBuilder {
        self.0.time = time;
        self
    }

    /// Set at which level and above (more verbose) the thread id shall be logged. (default is Debug)
    pub fn set_thread_level(&mut self, thread: LevelFilter) -> &mut ConfigBuilder {
        self.0.thread = thread;
        self
    }

    /// Set at which level and above (more verbose) the target shall be logged. (default is Debug)
    pub fn set_target_level(&mut self, target: LevelFilter) -> &mut ConfigBuilder {
        self.0.target = target;
        self
    }

    /// Set at which level and above (more verbose) a source code reference shall be logged (default is Trace)
    pub fn set_location_level(&mut self, location: LevelFilter) -> &mut ConfigBuilder {
        self.0.location = location;
        self
    }

    /// Set how the levels should be padded, when logging (default is Off)
    pub fn set_level_padding(&mut self, padding: LevelPadding) -> &mut ConfigBuilder {
        self.0.level_padding = padding;
        self
    }

    /// Set how the thread should be padded
    pub fn set_thread_padding(&mut self, padding: ThreadPadding) -> &mut ConfigBuilder {
        self.0.thread_padding = padding;
        self
    }

    /// Set the mode for logging the thread
    pub fn set_thread_mode(&mut self, mode: ThreadLogMode) -> &mut ConfigBuilder {
        self.0.thread_log_mode = mode;
        self
    }

    /// Set the color used for printing the level (if the logger supports it),
    /// or None to use the default foreground color
    #[cfg(feature = "termcolor")]
    pub fn set_level_color(&mut self, level: Level, color: Option<Color>) -> &mut ConfigBuilder {
        self.0.level_color[level as usize] = color;
        self
    }

    /// Set time chrono [strftime] format string.
    ///
    /// [strftime]: https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html#specifiers
    pub fn set_time_format_str(&mut self, time_format: &'static str) -> &mut ConfigBuilder {
        self.0.time_format = Cow::Borrowed(time_format);
        self
    }

    /// Set time chrono [strftime] format string.
    ///
    /// [strftime]: https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html#specifiers
    pub fn set_time_format(&mut self, time_format: String) -> &mut ConfigBuilder {
        self.0.time_format = Cow::Owned(time_format);
        self
    }

    /// Set offset used for logging time (default is 0)
    pub fn set_time_offset(&mut self, time_offset: FixedOffset) -> &mut ConfigBuilder {
        self.0.time_offset = time_offset;
        self
    }

    /// set if you log in local timezone or UTC (default is UTC)
    pub fn set_time_to_local(&mut self, local: bool) -> &mut ConfigBuilder {
        self.0.time_local = local;
        self
    }

    /// Add allowed module filters.
    /// If any are specified, only records from modules starting with one of these entries will be printed
    ///
    /// For example, `add_filter_allow_str("tokio::uds")` would allow only logging from the `tokio` crates `uds` module.
    pub fn add_filter_allow_str(&mut self, filter_allow: &'static str) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_allow);
        list.push(Cow::Borrowed(filter_allow));
        self.0.filter_allow = Cow::Owned(list);
        self
    }

    /// Add allowed module filters.
    /// If any are specified, only records from modules starting with one of these entries will be printed
    ///
    /// For example, `add_filter_allow(format!("{}{}","tokio", "uds"))` would allow only logging from the `tokio` crates `uds` module.
    pub fn add_filter_allow(&mut self, filter_allow: String) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_allow);
        list.push(Cow::Owned(filter_allow));
        self.0.filter_allow = Cow::Owned(list);
        self
    }

    /// Clear allowed module filters.
    /// If none are specified, nothing is filtered out
    pub fn clear_filter_allow(&mut self) -> &mut ConfigBuilder {
        self.0.filter_allow = Cow::Borrowed(&[]);
        self
    }

    /// Add denied module filters.
    /// If any are specified, records from modules starting with one of these entries will be ignored
    ///
    /// For example, `add_filter_ignore_str("tokio::uds")` would deny logging from the `tokio` crates `uds` module.
    pub fn add_filter_ignore_str(&mut self, filter_ignore: &'static str) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_ignore);
        list.push(Cow::Borrowed(filter_ignore));
        self.0.filter_ignore = Cow::Owned(list);
        self
    }

    /// Add denied module filters.
    /// If any are specified, records from modules starting with one of these entries will be ignored
    ///
    /// For example, `add_filter_ignore(format!("{}{}","tokio", "uds"))` would deny logging from the `tokio` crates `uds` module.
    pub fn add_filter_ignore(&mut self, filter_ignore: String) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_ignore);
        list.push(Cow::Owned(filter_ignore));
        self.0.filter_ignore = Cow::Owned(list);
        self
    }

    /// Clear ignore module filters.
    /// If none are specified, nothing is filtered
    pub fn clear_filter_ignore(&mut self) -> &mut ConfigBuilder {
        self.0.filter_ignore = Cow::Borrowed(&[]);
        self
    }

    /// Build new `Config`
    pub fn build(&mut self) -> Config {
        self.0.clone()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder::new()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            time: LevelFilter::Error,
            level: LevelFilter::Error,
            level_padding: LevelPadding::Off,
            thread: LevelFilter::Debug,
            thread_log_mode: ThreadLogMode::IDs,
            thread_padding: ThreadPadding::Off,
            target: LevelFilter::Debug,
            location: LevelFilter::Trace,
            time_format: Cow::Borrowed("%H:%M:%S"),
            time_offset: FixedOffset::east(0),
            time_local: false,
            filter_allow: Cow::Borrowed(&[]),
            filter_ignore: Cow::Borrowed(&[]),

            #[cfg(feature = "termcolor")]
            level_color: [
                None,                // Default foreground
                Some(Color::Red),    // Error
                Some(Color::Yellow), // Warn
                Some(Color::Blue),   // Info
                Some(Color::Cyan),   // Debug
                Some(Color::White),  // Trace
            ],
        }
    }
}

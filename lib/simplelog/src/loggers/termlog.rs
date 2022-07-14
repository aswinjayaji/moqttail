//! Module providing the TermLogger Implementation

use log::{
    set_boxed_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record, SetLoggerError,
};
use std::io::{Error, Write};
use std::sync::Mutex;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::logging::*;

use crate::{Config, SharedLogger, ThreadLogMode};

enum StdTerminal {
    Stderr(Box<dyn WriteColor + Send>),
    Stdout(Box<dyn WriteColor + Send>),
}

impl StdTerminal {
    fn flush(&mut self) -> Result<(), Error> {
        match self {
            StdTerminal::Stderr(term) => term.flush(),
            StdTerminal::Stdout(term) => term.flush(),
        }
    }
}

struct OutputStreams {
    err: StdTerminal,
    out: StdTerminal,
}

/// Specifies which streams should be used when logging
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TerminalMode {
    /// Only use Stdout
    Stdout,
    /// Only use Stderr
    Stderr,
    /// Use Stderr for Errors and Stdout otherwise
    Mixed,
}

impl Default for TerminalMode {
    fn default() -> TerminalMode {
        TerminalMode::Mixed
    }
}

/// The TermLogger struct. Provides a stderr/out based Logger implementation
///
/// Supports colored output
pub struct TermLogger {
    level: LevelFilter,
    config: Config,
    streams: Mutex<OutputStreams>,
}

impl TermLogger {
    /// init function. Globally initializes the TermLogger as the one and only used log facility.
    ///
    /// Takes the desired `Level` and `Config` as arguments. They cannot be changed later on.
    /// Fails if another Logger was already initialized
    ///
    /// # Examples
    /// ```
    /// # extern crate simplelog;
    /// # use simplelog::*;
    /// # fn main() {
    ///     TermLogger::init(
    ///         LevelFilter::Info,
    ///         Config::default(),
    ///         TerminalMode::Mixed,
    ///         ColorChoice::Auto
    ///     );
    /// # }
    /// ```
    pub fn init(
        log_level: LevelFilter,
        config: Config,
        mode: TerminalMode,
        color_choice: ColorChoice,
    ) -> Result<(), SetLoggerError> {
        let logger = TermLogger::new(log_level, config, mode, color_choice);
        set_max_level(log_level);
        set_boxed_logger(logger)?;
        Ok(())
    }

    /// allows to create a new logger, that can be independently used, no matter whats globally set.
    ///
    /// no macros are provided for this case and you probably
    /// dont want to use this function, but `init()`, if you dont want to build a `CombinedLogger`.
    ///
    /// Takes the desired `Level` and `Config` as arguments. They cannot be changed later on.
    ///
    /// Returns a `Box`ed TermLogger
    ///
    /// # Examples
    /// ```
    /// # extern crate simplelog;
    /// # use simplelog::*;
    /// # fn main() {
    /// let term_logger = TermLogger::new(
    ///     LevelFilter::Info,
    ///     Config::default(),
    ///     TerminalMode::Mixed,
    ///     ColorChoice::Auto
    /// );
    /// # }
    /// ```
    pub fn new(
        log_level: LevelFilter,
        config: Config,
        mode: TerminalMode,
        color_choice: ColorChoice,
    ) -> Box<TermLogger> {
        let streams = match mode {
            TerminalMode::Stdout => OutputStreams {
                err: StdTerminal::Stdout(Box::new(StandardStream::stdout(color_choice))),
                out: StdTerminal::Stdout(Box::new(StandardStream::stdout(color_choice))),
            },
            TerminalMode::Stderr => OutputStreams {
                err: StdTerminal::Stderr(Box::new(StandardStream::stderr(color_choice))),
                out: StdTerminal::Stderr(Box::new(StandardStream::stderr(color_choice))),
            },
            TerminalMode::Mixed => OutputStreams {
                err: StdTerminal::Stderr(Box::new(StandardStream::stderr(color_choice))),
                out: StdTerminal::Stdout(Box::new(StandardStream::stdout(color_choice))),
            },
        };

        Box::new(TermLogger {
            level: log_level,
            config,
            streams: Mutex::new(streams),
        })
    }

    fn try_log_term(
        &self,
        record: &Record<'_>,
        term_lock: &mut Box<dyn WriteColor + Send>,
    ) -> Result<(), Error> {
        let color = self.config.level_color[record.level() as usize];

        if self.config.time <= record.level() && self.config.time != LevelFilter::Off {
            write_time(&mut *term_lock, &self.config)?;
        }

        if self.config.level <= record.level() && self.config.level != LevelFilter::Off {
            term_lock.set_color(ColorSpec::new().set_fg(color))?;
            write_level(record, &mut *term_lock, &self.config)?;
            term_lock.reset()?;
        }

        if self.config.thread <= record.level() && self.config.thread != LevelFilter::Off {
            match self.config.thread_log_mode {
                ThreadLogMode::IDs => {
                    write_thread_id(&mut *term_lock, &self.config)?;
                }
                ThreadLogMode::Names | ThreadLogMode::Both => {
                    write_thread_name(&mut *term_lock, &self.config)?;
                }
            }
        }

        if self.config.target <= record.level() && self.config.target != LevelFilter::Off {
            write_target(record, &mut *term_lock)?;
        }

        if self.config.location <= record.level() && self.config.location != LevelFilter::Off {
            write_location(record, &mut *term_lock)?;
        }

        write_args(record, &mut *term_lock)
    }

    fn try_log(&self, record: &Record<'_>) -> Result<(), Error> {
        if self.enabled(record.metadata()) {
            if should_skip(&self.config, record) {
                return Ok(());
            }

            let mut streams = self.streams.lock().unwrap();

            if record.level() == Level::Error {
                match streams.err {
                    StdTerminal::Stderr(ref mut term) => self.try_log_term(record, term),
                    StdTerminal::Stdout(ref mut term) => self.try_log_term(record, term),
                }
            } else {
                match streams.out {
                    StdTerminal::Stderr(ref mut term) => self.try_log_term(record, term),
                    StdTerminal::Stdout(ref mut term) => self.try_log_term(record, term),
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Log for TermLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        let _ = self.try_log(record);
    }

    fn flush(&self) {
        let mut streams = self.streams.lock().unwrap();
        let _ = streams.out.flush();
        let _ = streams.err.flush();
    }
}

impl SharedLogger for TermLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}

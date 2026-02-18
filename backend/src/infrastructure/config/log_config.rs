// https://github.com/clia/tracing-config/blob/main/src/lib.rs
use serde::Deserialize;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;

use tracing_appender::rolling;

use super::{default_true,default_directory,default_format,default_file_name,default_filter_level,default_rolling, FORMAT_COMPACT, FORMAT_FULL, FORMAT_JSON, FORMAT_PRETTY};



#[derive(Deserialize, Clone, Debug)]
pub struct LogConfig {
    #[serde(default = "default_filter_level")]
    pub filter_level: String,
    #[serde(default = "default_true")]
    pub with_ansi: bool,
    #[serde(default = "default_true")]
    pub stdout: bool,
    #[serde(default = "default_directory")]
    pub directory: String,
    #[serde(default = "default_file_name")]
    pub file_name: String,
    #[serde(default = "default_rolling")]
    pub rolling: String,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub with_level: bool,
    #[serde(default = "default_true")]
    pub with_target: bool,
    #[serde(default = "default_true")]
    pub with_thread_ids: bool,
    #[serde(default = "default_true")]
    pub with_thread_names: bool,
    #[serde(default = "default_true")]
    pub with_source_location: bool,
}


impl Default for LogConfig {
    fn default() -> Self {
        Self {
            filter_level: default_filter_level(),
            with_ansi: true,
            stdout: false,
            directory: default_directory(),
            file_name: default_file_name(),
            rolling: default_rolling(),
            format: default_format(),
            with_level: true,
            with_target: true,
            with_thread_ids: true,
            with_thread_names: true,
            with_source_location: true,
        }
    }
}

#[allow(dead_code)]
impl LogConfig {
    /// Will try_from_default_env while not setted.
    ///
    /// You can use value like "info", or something like "mycrate=trace".
    ///
    /// Default value if "info".
    ///
    pub fn filter_level(mut self, filter_level: &str) -> Self {
        self.filter_level = filter_level.to_owned();
        self
    }

    /// Show ANSI color symbols.
    pub fn with_ansi(mut self, with_ansi: bool) -> Self {
        self.with_ansi = with_ansi;
        self
    }

    /// Will append log to stdout.
    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    /// Set log file directory.
    pub fn directory(mut self, directory: impl Into<String>) -> Self {
        self.directory = directory.into();
        self
    }

    /// Set log file name.
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = file_name.into();
        self
    }

    /// Valid values: minutely | hourly | daily | never
    ///
    /// Will panic on other values.
    pub fn rolling(mut self, rolling: impl Into<String>) -> Self {
        let rolling = rolling.into();
        if !["minutely", "hourly", "daily", "never"].contains(&&*rolling) {
            panic!("Unknown rolling")
        }
        self.rolling = rolling;
        self
    }

    /// Valid values: pretty | compact | json | full
    ///
    /// Will panic on other values.
    pub fn format(mut self, format: impl Into<String>) -> Self {
        let format = format.into();
        if format != FORMAT_PRETTY
            && format != FORMAT_COMPACT
            && format != FORMAT_JSON
            && format != FORMAT_FULL
        {
            panic!("Unknown format")
        }
        self.format = format;
        self
    }

    /// include levels in formatted output
    pub fn with_level(mut self, with_level: bool) -> Self {
        self.with_level = with_level;
        self
    }

    /// include targets
    pub fn with_target(mut self, with_target: bool) -> Self {
        self.with_target = with_target;
        self
    }

    /// include the thread ID of the current thread
    pub fn with_thread_ids(mut self, with_thread_ids: bool) -> Self {
        self.with_thread_ids = with_thread_ids;
        self
    }

    /// include the name of the current thread
    pub fn with_thread_names(mut self, with_thread_names: bool) -> Self {
        self.with_thread_names = with_thread_names;
        self
    }

    /// include source location
    pub fn with_source_location(mut self, with_source_location: bool) -> Self {
        self.with_source_location = with_source_location;
        self
    }

   
    
}

extern crate log;

use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
enum OutputLevel {
    Error,
    Warning,
    Verbose,
    Debug,
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct AppOptions {
    /// Set the log level of the application.
    ///
    /// You can also set this through the LOG_SPECIFICATION environment variable, but this is a
    /// simpler option for the most cases.
    ///
    /// The default level if neither this option or LOG_SPECIFICATION is provided is "warning".
    ///
    /// Explanation of the values:
    ///
    ///     - error: Only show errors from this app.
    ///
    ///     - warning: Show warnings from this app, and errors from dependencies.
    ///
    ///     - verbose: Show info messages from this app, and warnings from dependencies.
    ///
    ///     - debug: Show debug messages from this app and the HTTP library, info from other
    ///     dependencies.
    ///
    /// Debug should only be useful in development and can contain sensitive information so never
    /// activate this output level in a production environment.
    ///
    ///
    #[structopt(
        short = "l",
        long = "log-level",
        value_name = "LEVEL",
        env = "LOG_LEVEL",
        raw(possible_values = "OutputLevel::variants()")
    )]
    output_level: Option<OutputLevel>,

    /// Configure address to bind to. It's recommended to place this service behind a more mature
    /// HTTP server (like nginx, Apache, etc.) for security, so the default option is not to bind
    /// to 0.0.0.0:80.
    ///
    /// You are of course still free to bind to the HTTP port on all interfaces if you so wish.
    #[structopt(
        long = "bind",
        value_name = "ADDR",
        env = "BIND",
        default_value = "127.0.0.1:8002",
    )]
    pub bind: SocketAddr,
}

impl AppOptions {
    pub fn init_logger(&self) {
        use log::LevelFilter;
        let mut builder = env_logger::Builder::from_env("LOG_SPECIFICATION");

        if let Some(output_level) = self.output_level {
            match output_level {
                OutputLevel::Error => {
                    builder.filter_level(LevelFilter::Off);
                    builder.filter_module("prgnome", LevelFilter::Error);
                }
                OutputLevel::Warning => {
                    builder.filter_level(LevelFilter::Error);
                    builder.filter_module("prgnome", LevelFilter::Warn);
                }
                OutputLevel::Verbose => {
                    builder.filter_level(LevelFilter::Warn);
                    builder.filter_module("prgnome", LevelFilter::Info);
                }
                OutputLevel::Debug => {
                    builder.filter_level(LevelFilter::Info);
                    builder.filter_module("actix_web", LevelFilter::Debug);
                    builder.filter_module("prgnome", LevelFilter::Debug);
                }
            };
        }

        builder.init();
    }
}

impl OutputLevel {
    fn variants() -> &'static [&'static str] {
        &["error", "warning", "verbose", "debug"]
    }

    fn as_str(&self) -> &'static str {
        match self {
            OutputLevel::Error => "error",
            OutputLevel::Warning => "warning",
            OutputLevel::Verbose => "verbose",
            OutputLevel::Debug => "debug",
        }
    }
}

impl FromStr for OutputLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "error" => Ok(OutputLevel::Error),
            "warning" => Ok(OutputLevel::Warning),
            "verbose" => Ok(OutputLevel::Verbose),
            "debug" => Ok(OutputLevel::Debug),
            other => Err(format!("Unknown output level: {}", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod output_level {
        use super::*;

        #[test]
        fn it_is_parsed_from_strings() {
            assert_eq!("DEbUg".parse(), Ok(OutputLevel::Debug));
            assert_eq!("VerboSE".parse(), Ok(OutputLevel::Verbose));
            assert_eq!("WarnING".parse(), Ok(OutputLevel::Warning));
            assert_eq!("ErrOR".parse(), Ok(OutputLevel::Error));
            assert_eq!(
                "warn".parse::<OutputLevel>(),
                Err(String::from("Unknown output level: warn"))
            );
        }
    }
}

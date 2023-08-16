use log::{
    info,
    debug,
    warn,
};
use std::{
    env::{
        self,
        VarError,
    },
    error::Error,
    ffi::{
        OsStr,
        OsString,
    },
    num::ParseIntError,
    str::ParseBoolError,
    sync::Once,
};
use servo_config::{
    opts::{
        self,
        Opts,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("failed to parse boolean: {0}, {1}")]
struct ParseBoolFromIntError(ParseBoolError, ParseIntError);

#[derive(Debug, thiserror::Error)]
enum ParseEnvError<E: Error> {
    #[error("Variable did not contain valid unicode: {0:?}")]
    NotUnicode(OsString),
    #[error("Error parsing variable value: {0}")]
    Parse(E),
}

fn parse_bool_or_int<S: AsRef<str>>(s: S) -> Result<bool, ParseBoolFromIntError> {
    let s = s.as_ref();
    s.parse::<bool>()
        .or_else(|bool_e| {
            s.parse::<u8>()
                .map(|v| v != 0)
                .map_err(move |int_e| ParseBoolFromIntError(bool_e, int_e))
        })
}

fn env_boolean<K: AsRef<OsStr>>(key: K, default_value: bool) -> Result<bool, ParseEnvError<ParseBoolFromIntError>> {
    let s = match env::var(key) {
        Err(VarError::NotPresent) => {
            return Ok(default_value);
        },
        Ok(v) => Ok(v),
        Err(VarError::NotUnicode(v)) => Err(ParseEnvError::NotUnicode(v)),
    }?;
    parse_bool_or_int(s)
        .map_err(ParseEnvError::Parse)
}

trait OptsExt {
    /// Loads [`servo_config::opts::Opts`] from defaults, with optional modifications based on
    /// environment variables
    fn from_env() -> Self;
}

impl OptsExt for Opts {
    fn from_env() -> Self {
        let original = opts::get();
        let env_boolean_ensure = |s, v| env_boolean(s, v).ok().unwrap_or(v);
        debug!("original servo options: {:?}", &original);
        Opts {
            multiprocess: env_boolean_ensure("SERVO_GST_MULTIPROCESS", false),
            sandbox: env_boolean_ensure("SERVO_GST_SANDBOX", original.sandbox),
            exit_after_load: false,
            ..original.clone()
        }
    }
}

static INIT_OPTS: Once = Once::new();

/// Makes any runtime configuration changes to [`servo_config::opts::Opts`] since we don't have
/// command line parameters to load them from
pub fn init() {
    INIT_OPTS.call_once(|| {
        let cfg = Opts::from_env();
        info!("reconfigured servo options: {:?}", &cfg);
        opts::set_options(cfg);
    });
}

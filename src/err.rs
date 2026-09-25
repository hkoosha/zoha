use crate::config::cfg::CfgReadError;
use log::SetLoggerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZohaError {
    #[error("dbus failure")]
    DBus(#[from] dbus::Error),

    #[error("unknown dbus error")]
    UnknownDBus,

    #[error("app is already active")]
    AppAlreadyActive,

    #[error("could not find any displays")]
    NoDisplay,

    #[error("window already set")]
    AlreadyWindow,

    #[error("no shell was found to run")]
    NoShell,

    #[error("cfg error: {0}")]
    Cfg(CfgReadError),

    #[error("logger error: {0}")]
    SetLogger(SetLoggerError),

    #[error("logger error: {0}")]
    GlibBool(glib::BoolError),

    #[error("accelerator key is duplicated: {0}")]
    AcceleratorDuplicated(String),

    #[error("accelerator key is invalid: {0}")]
    AcceleratorInvalid(String),

    #[error("unknown error")]
    Unknown,
}

impl From<CfgReadError> for ZohaError {
    fn from(value: CfgReadError) -> Self {
        return Self::Cfg(value);
    }
}

impl From<SetLoggerError> for ZohaError {
    fn from(value: SetLoggerError) -> Self {
        return Self::SetLogger(value);
    }
}

impl From<glib::BoolError> for ZohaError {
    fn from(value: glib::BoolError) -> Self {
        return Self::GlibBool(value);
    }
}

use serde::Deserialize;
use thiserror::Error;

#[cfg(test)]
mod tests;
pub mod util;

pub trait ConfigProvider {
    fn extract<C>(&self, key: impl AsRef<str>) -> ConfigParseResult<C>
    where
        C: for<'a> Deserialize<'a>;
}

pub type ConfigParseResult<T> = Result<T, ConfigParseErr>;

#[derive(Error, Debug)]
pub enum ConfigParseErr {
    #[error("unable to find key")]
    NoKey,
    #[error("unable to deserialize into desired type")]
    DeserializeFail,
    #[error("{0:?}")]
    Other(String),
}

pub trait Mediate<M> {
    type Out;
    fn mediate(self, module: M) -> Self::Out;
}

pub trait Module {
    type Config;
    type Out;

    fn initialize(self, config: Self::Config) -> Self::Out;
}

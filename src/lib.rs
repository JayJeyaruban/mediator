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
}

pub trait Mediate<M> {
    type Out;
    fn mediate(self, module: M) -> Self::Out;
}

impl<T, M> Mediate<Option<M>> for T
where
    T: Mediate<M, Out = Self>,
{
    type Out = Self;

    fn mediate(self, module: Option<M>) -> Self::Out {
        if let Some(module) = module {
            self.mediate(module)
        } else {
            self
        }
    }
}

impl<T, M> Mediate<Vec<M>> for T
where
    T: Mediate<M, Out = Self>,
{
    type Out = Self;

    fn mediate(mut self, modules: Vec<M>) -> Self::Out {
        for module in modules {
            self = self.mediate(module)
        }
        self
    }
}

pub trait Module {
    type Config;
    type Out;

    fn initialize(self, config: Self::Config) -> Self::Out;
}

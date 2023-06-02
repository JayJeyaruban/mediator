use config::Config;
use derive_more::From;
use mediator::{ConfigParseErr, ConfigProvider};
use serde::Deserialize;

#[derive(Default, From)]
pub struct ConfigRsAdapter(pub Config);

impl ConfigProvider for ConfigRsAdapter {
    fn extract<C>(&self, key: impl AsRef<str>) -> mediator::ConfigParseResult<C>
    where
        C: for<'a> Deserialize<'a>,
    {
        self.0.get(key.as_ref()).map_err(|e| match e {
            config::ConfigError::NotFound(_) => ConfigParseErr::NoKey,
            config::ConfigError::PathParse(_) => ConfigParseErr::DeserializeFail,
            other => ConfigParseErr::Other(other.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use config::Config;
    use mediator::ConfigProvider;
    use serde::Deserialize;

    use crate::configrs::ConfigRsAdapter;

    #[test]
    fn valid_yml() {
        #[derive(Deserialize)]
        struct TestConfig {
            enabled: bool,
        }
        let source = Config::builder()
            .add_source(config::File::with_name("mediator-config/src/sample"))
            .build()
            .unwrap();

        let adapter: ConfigRsAdapter = source.into();

        let config: TestConfig = adapter.extract("mediator").unwrap();
        assert!(config.enabled)
    }
}

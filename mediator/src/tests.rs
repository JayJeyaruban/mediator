use std::collections::HashMap;

use crate::{
    util::label::{IntoLabelled, Labelled},
    ConfigParseErr, ConfigParseResult, ConfigProvider, Mediate, Module,
};

use serde::Deserialize;

#[test]
fn base_mediate() {
    #[derive(Default)]
    struct TestApp(i32);
    impl Mediate<i32> for TestApp {
        type Out = Self;

        fn mediate(mut self, module: i32) -> Self::Out {
            self.0 += module;
            self
        }
    }

    assert!(TestApp::default().mediate(123).mediate(1).0 == 124)
}

#[test]
fn base_case() {
    struct TestConfig(i32);
    struct TestModule;
    struct TestOut(String);

    impl Module for TestModule {
        type Config = TestConfig;

        type Out = TestOut;

        fn initialize(self, config: Self::Config) -> Self::Out {
            TestOut(config.0.to_string())
        }
    }

    assert!(TestModule.initialize(TestConfig(2)).0 == "2".to_string())
}

#[test]
fn mediate_app() {
    #[derive(Default)]
    struct TestConfigProvider(HashMap<String, String>);

    impl ConfigProvider for TestConfigProvider {
        fn extract<C>(&self, key: impl AsRef<str>) -> ConfigParseResult<C>
        where
            C: for<'a> Deserialize<'a>,
        {
            let config_str = self
                .0
                .get(key.as_ref())
                .ok_or(ConfigParseErr::NoKey)?
                .to_owned();

            serde_json::from_str(&config_str).map_err(|_| ConfigParseErr::DeserializeFail)
        }
    }

    struct TestApp<CP: ConfigProvider> {
        config_source: CP,
        inited: bool,
    }

    impl TestApp<TestConfigProvider> {
        fn default() -> Self {
            TestApp {
                config_source: TestConfigProvider::default(),
                inited: false,
            }
        }
    }

    struct TestModule;
    struct LoadedTestModule(bool);

    impl Module for TestModule {
        type Config = Option<bool>;

        type Out = LoadedTestModule;

        fn initialize(self, config: Self::Config) -> Self::Out {
            LoadedTestModule(config.unwrap_or(false))
        }
    }

    impl<L: AsRef<str>, M, C, CP> Mediate<Labelled<L, M>> for TestApp<CP>
    where
        M: Module<Config = Option<C>>,
        C: for<'a> Deserialize<'a>,
        CP: ConfigProvider,
        TestApp<CP>: Mediate<M::Out>,
    {
        type Out = <TestApp<CP> as Mediate<M::Out>>::Out;

        fn mediate(self, module: Labelled<L, M>) -> Self::Out {
            let label = module.label.as_ref();
            let config = self.config_source.extract(label).ok();

            let output = module.inner.initialize(config);
            self.mediate(output)
        }
    }

    impl<CP: ConfigProvider> Mediate<LoadedTestModule> for TestApp<CP> {
        type Out = TestApp<CP>;

        fn mediate(mut self, module: LoadedTestModule) -> Self::Out {
            self.inited = module.0;
            self
        }
    }

    let app = TestApp::default().mediate(TestModule.labelled("test"));

    assert!(!app.inited, "inited = {0}", app.inited);

    let mut app = TestApp::default();
    app.config_source
        .0
        .insert("test".to_string(), serde_json::to_string(&true).unwrap());
    app = app.mediate(TestModule.labelled("test"));

    assert!(app.inited, "inited = {0}", app.inited)
}

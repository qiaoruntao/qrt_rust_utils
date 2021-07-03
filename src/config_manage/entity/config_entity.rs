use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub(crate) struct TestConfigEntity {
    pub a: String,
    pub b: TestSubConfigEntity,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub(crate) struct TestSubConfigEntity {
    pub c: u32,
}

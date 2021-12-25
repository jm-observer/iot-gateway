use config::{File, FileFormat};
use std::ops::{Deref, DerefMut};

pub struct MConfig(config::Config);

impl MConfig {
    pub fn init() -> Self {
        let mut c = config::Config::default();
        c.merge(File::new("../config/config.yaml", FileFormat::Yaml))
            .unwrap();
        Self(c)
    }
}

impl Deref for MConfig {
    type Target = config::Config;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for MConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[test]
fn test_mconfig() {
    let c = MConfig::init();
    let modules = c.get_array("module").unwrap();
    for val in modules {
        let val = val.into_table().unwrap();
        println!("module: {:?}", val);
    }
}

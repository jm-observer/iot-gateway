use crate::pub_use::*;
use rumqttc::MqttOptions;
use std::time::Duration;

#[derive(Deserialize)]
pub struct MMqttConfig {
    addr: String,
    port: u16,
    user: String,
    password: String,
    id: String,
    #[serde(default = "default_keep_alive")]
    keep_alive: u64,
}

fn default_keep_alive() -> u64 {
    120
}

impl Into<MqttOptions> for MMqttConfig {
    fn into(self) -> MqttOptions {
        let mut mqttoptions = MqttOptions::new(self.id, self.addr, self.port);
        mqttoptions.set_keep_alive(Duration::from_secs(self.keep_alive));
        mqttoptions.set_credentials(self.user, self.password);
        mqttoptions
    }
}

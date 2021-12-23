pub use log::{debug, error, info, warn};
pub use log4rs;

pub fn init_log_config() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
}

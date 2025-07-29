use mado::services::host::HostService;
use wry_cmd::commands;

use crate::get_rainmeter;

struct Host;

static INSTANCE: Host = Host;

#[commands(name = "host")]
impl HostService for Host {
    fn get_host(&self) -> String {
        return "Shigure/Rainmeter".to_string();
    }
}

#[commands(name = "host")]

impl Host {
    /// **Rainmeter Only**
    /// Read a string from Rainmeter.
    fn read_string(&self, name: String) {
        get_rainmeter().unwrap().log(
            rainmeter::RmLogLevel::LogNotice,
            &format!("Read String {}", name),
        );
    }
}

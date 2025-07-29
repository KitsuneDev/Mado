use mado::services::host::HostService;
use wry_cmd::commands;

use crate::{build_info, get_rainmeter};

struct Host;

static INSTANCE: Host = Host;

//#[commands(name = "host")]
impl HostService for Host {
    fn get_host(&self) -> String {
        return "Rainmeter".to_string();
    }
}

#[commands(name = "host")]
impl Host {
    fn read_string(&self, name: String) {
        get_rainmeter().unwrap().log(
            rainmeter::RmLogLevel::LogNotice,
            &format!("Read String {}", name),
        );
    }
}

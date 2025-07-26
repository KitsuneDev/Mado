use mado::services::host::HostService;
use wry_cmd::commands;

struct Host;

static INSTANCE: Host = Host;

#[commands]
impl HostService for Host {
    fn get_host(&self) -> &'static str {
        return "Rainmeter";
    }
}

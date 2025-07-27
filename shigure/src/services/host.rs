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

    fn get_version(&self) -> String {
        return build_info::VERSION.to_string();
    }

    fn get_tag(&self) -> String {
        return build_info::TAG.to_string();
    }

    fn get_commit(&self) -> String {
        return build_info::COMMIT_HASH.to_string();
    }

    fn get_branch(&self) -> String {
        return build_info::BRANCH.to_string();
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

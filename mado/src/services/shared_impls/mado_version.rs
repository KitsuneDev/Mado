use shadow_rs::shadow;
use wry_cmd::commands;

use crate::services::mado_version::MadoVersionService;

pub struct MadoVersion;
static INSTANCE: MadoVersion = MadoVersion;

shadow!(build_info);

#[commands]
impl MadoVersionService for MadoVersion {
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

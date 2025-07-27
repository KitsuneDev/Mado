pub trait HostService {
    fn get_host(&self) -> String;
    fn get_version(&self) -> String;
    fn get_tag(&self) -> String;
    fn get_commit(&self) -> String;
    fn get_branch(&self) -> String;
    //fn get_full_version_info(&self) -> String;
}

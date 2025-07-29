pub trait MadoVersionService {
    fn get_version(&self) -> String;
    fn get_tag(&self) -> String;
    fn get_commit(&self) -> String;
    fn get_branch(&self) -> String;
}

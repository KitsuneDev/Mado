pub trait HostService {
    /// Returns the name of the Mado Host.
    /// Example:
    /// * Shigure/Rainmeters
    fn get_host(&self) -> String;
    //fn get_full_version_info(&self) -> String;
}

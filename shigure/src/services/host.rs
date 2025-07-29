use mado::services::host::HostService;
use serde::Deserialize;
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
#[derive(Deserialize)]
struct RmReadParameters<T> {
    key: String,
    default: T,
}
#[commands(name = "host")]
impl Host {
    /// **Rainmeter Only**
    /// Read a string from Rainmeter.
    fn read_string(&self, args: RmReadParameters<String>) -> String {
        if let Some(rm) = get_rainmeter() {
            return rm.read_string(&args.key, &args.default);
        }
        args.default
    }

    /// **Rainmeter Only**
    fn read_double(&self, args: RmReadParameters<f64>) -> f64 {
        if let Some(rm) = get_rainmeter() {
            return rm.read_double(&args.key, args.default);
        }
        args.default
    }
    /// **Rainmeter Only**
    fn read_formula(&self, args: RmReadParameters<f64>) -> f64 {
        if let Some(rm) = get_rainmeter() {
            return rm.read_formula(&args.key, args.default);
        }
        args.default
    }
    /// **Rainmeter Only**
    fn read_int(&self, args: RmReadParameters<i32>) -> i32 {
        if let Some(rm) = get_rainmeter() {
            return rm.read_int(&args.key, args.default);
        }
        args.default
    }
    /// **Rainmeter Only**
    fn get_skin_name(&self) -> String {
        if let Some(rm) = get_rainmeter() {
            return rm.get_skin_name();
        }
        "Unknown".to_string()
    }
    /// **Rainmeter Only**
    /// Replace a Rainmeter Variable by its value
    /// var - The Var String, like: #MyVar#
    fn get_variable(&self, var: String) -> String {
        if let Some(rm) = get_rainmeter() {
            return rm.replace_variables(&var);
        }
        "".to_string()
    }
    /// **Rainmeter Only**
    /// Execute a Rainmeter Bang
    /// Example: [!SetVariable SomeVar 10]
    fn execute_bang(&self, bang: String) -> () {
        if let Some(rm) = get_rainmeter() {
            return rm.execute(&bang);
        }
    }
}

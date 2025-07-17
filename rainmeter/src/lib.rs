// src/plugin_interface.rs
//! High-level Rust interface to the Rainmeter C/C++ plugin API.
//!
//! Based directly on RainmeterAPI.h (GPL‑2.0) — all host‑provided functions are declared via FFI,
//! and `RainmeterContext` wraps them in safe, Rust‑native methods.

use std::ffi::{OsStr, c_void};
use std::marker::PhantomData;
use std::os::windows::ffi::OsStrExt;
use windows::core::BOOL;
use windows::core::PCWSTR;
// -----------------------------------------------------------------------
// 1) FFI declarations of host‑provided Rainmeter API functions
//    See https://docs.rainmeter.net/developers/plugin/cpp/api/
// -----------------------------------------------------------------------
#[link(name = "Rainmeter")]
unsafe extern "system" {
    pub fn RmReadString(
        rm: *mut c_void,
        option: PCWSTR,
        def_value: PCWSTR,
        replace_measures: BOOL,
    ) -> PCWSTR;

    pub fn RmReadStringFromSection(
        rm: *mut c_void,
        section: PCWSTR,
        option: PCWSTR,
        def_value: PCWSTR,
        replace_measures: BOOL,
    ) -> PCWSTR;

    pub fn RmReadFormula(rm: *mut c_void, option: PCWSTR, def_value: f64) -> f64;

    pub fn RmReadFormulaFromSection(
        rm: *mut c_void,
        section: PCWSTR,
        option: PCWSTR,
        def_value: f64,
    ) -> f64;

    pub fn RmReplaceVariables(rm: *mut c_void, str: PCWSTR) -> PCWSTR;

    pub fn RmPathToAbsolute(rm: *mut c_void, relative_path: PCWSTR) -> PCWSTR;

    pub fn RmExecute(skin: *mut c_void, command: PCWSTR);

    pub fn RmGet(rm: *mut c_void, what: i32) -> *mut c_void;

    pub fn RmLog(rm: *mut c_void, level: i32, message: PCWSTR);
}

// -----------------------------------------------------------------------
// 2) Helpers: wide‑string conversion
// -----------------------------------------------------------------------
fn to_pcwstr(s: &str) -> PCWSTR {
    let mut wide: Vec<u16> = OsStr::new(s).encode_wide().collect();
    wide.push(0);
    PCWSTR(wide.as_ptr())
}

unsafe fn from_pcwstr(ptr: PCWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // find length
    let mut len = 0;
    while *ptr.0.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16_lossy(slice)
}
pub enum RmLogLevel {
    LogError = 1,
    LogWarning = 2,
    LogNotice = 3,
    LogDebug = 4,
}
// -----------------------------------------------------------------------
// 3) High‑level Rust wrapper around the raw Rainmeter context pointer.
// -----------------------------------------------------------------------
pub struct RainmeterContext {
    raw: *mut c_void,
    //_marker: std::marker::PhantomData<&'a ()>,
}

impl RainmeterContext {
    /// Create a new context from the raw `rm` pointer.
    pub fn new(raw: *mut c_void) -> Self {
        Self {
            raw,
            //_marker: std::marker::PhantomData,
        }
    }

    /// Read an option as a string.
    pub fn read_string(&self, key: &str, default: &str) -> String {
        let k = to_pcwstr(key);
        let d = to_pcwstr(default);
        let ptr = unsafe { RmReadString(self.raw, k, d, BOOL(1)) };
        unsafe { from_pcwstr(ptr) }
    }

    /// Read an option as a string from another section.
    pub fn read_string_section(&self, section: &str, key: &str, default: &str) -> String {
        let s = to_pcwstr(section);
        let k = to_pcwstr(key);
        let d = to_pcwstr(default);
        let ptr = unsafe { RmReadStringFromSection(self.raw, s, k, d, BOOL(1)) };
        unsafe { from_pcwstr(ptr) }
    }

    /// Read an option (formula or number) as f64.
    pub fn read_formula(&self, key: &str, default: f64) -> f64 {
        let k = to_pcwstr(key);
        unsafe { RmReadFormula(self.raw, k, default) }
    }

    /// Read a formula from another section.
    pub fn read_formula_section(&self, section: &str, key: &str, default: f64) -> f64 {
        let s = to_pcwstr(section);
        let k = to_pcwstr(key);
        unsafe { RmReadFormulaFromSection(self.raw, s, k, default) }
    }

    /// Read an integer from a section (via `RmReadFormulaFromSection`).
    pub fn read_int_section(&self, section: &str, key: &str, default: i32) -> i32 {
        self.read_formula_section(section, key, default as f64) as i32
    }

    /// Read a double from a section.
    pub fn read_double_section(&self, section: &str, key: &str, default: f64) -> f64 {
        self.read_formula_section(section, key, default)
    }

    /// Replace variables in-line.
    pub fn replace_variables(&self, input: &str) -> String {
        let i = to_pcwstr(input);
        let ptr = unsafe { RmReplaceVariables(self.raw, i) };
        unsafe { from_pcwstr(ptr) }
    }

    /// Convert a relative path to an absolute path.
    pub fn path_to_absolute(&self, relative: &str) -> String {
        let r = to_pcwstr(relative);
        let ptr = unsafe { RmPathToAbsolute(self.raw, r) };
        unsafe { from_pcwstr(ptr) }
    }

    /// Convenience: read and resolve a path option.
    pub fn read_path(&self, key: &str, default: &str) -> String {
        let rel = self.read_string(key, default);
        self.path_to_absolute(&rel)
    }

    /// Execute a bang command.
    pub fn execute(&self, command: &str) {
        let c = to_pcwstr(command);
        unsafe { RmExecute(self.raw, c) };
    }

    /// Retrieve host‑provided data pointers.
    pub fn get(&self, what: i32) -> *mut c_void {
        unsafe { RmGet(self.raw, what) }
    }

    /// Read the measure name (via `RmGetType::RMG_MEASURENAME`).
    pub fn get_measure_name(&self) -> String {
        let ptr = self.get(0);
        unsafe { from_pcwstr(PCWSTR(ptr as _)) }
    }

    /// Read the settings file path (via RMG_SETTINGSFILE).
    pub fn get_settings_file(&self) -> String {
        let ptr = unsafe { RmGet(std::ptr::null_mut(), 2) };
        unsafe { from_pcwstr(PCWSTR(ptr as _)) }
    }

    /// Read the skin handle (void*).
    pub fn get_skin(&self) -> *mut c_void {
        self.get(1)
    }

    /// Read the skin name (via RMG_SKINNAME).
    pub fn get_skin_name(&self) -> String {
        let ptr = self.get(3);
        unsafe { from_pcwstr(PCWSTR(ptr as _)) }
    }

    /// Read the HWND of the skin window (via RMG_SKINWINDOWHANDLE).
    pub fn get_skin_window(&self) -> usize {
        let ptr = self.get(4);
        ptr as usize
    }

    /// Write a log message.
    pub fn log(&self, level: RmLogLevel, message: &str) {
        let m = to_pcwstr(message);
        unsafe { RmLog(self.raw, level as i32, m) };
    }
}

unsafe impl Send for RainmeterContext {}
unsafe impl Sync for RainmeterContext {}
impl Clone for RainmeterContext {
    fn clone(&self) -> Self {
        Self { raw: self.raw }
    }
}
/// Trait every Rust‑native plugin should implement.
pub trait RainmeterPlugin: Default + 'static {
    /// Called once when the measure is first loaded.
    fn initialize(&mut self, rm: RainmeterContext);
    /// Called when the skin is (re)loaded; `max_value` holds the default numeric value.
    fn reload(&mut self, rm: RainmeterContext, max_value: &mut f64);
    /// Called on every update cycle; return the numeric value.
    fn update(&mut self, rm: RainmeterContext) -> f64;
    /// Called when a string value is requested; return `Some(String)` or `None`.
    fn get_string(&mut self, rm: RainmeterContext) -> Option<String> {
        None
    }
    /// Called when the skin executes a bang on this measure; `args` is the argument string.
    fn execute_bang(&mut self, rm: RainmeterContext, args: &str) {
        // default no-op
    }
    /// Called once when the measure is unloaded.
    fn finalize(&mut self, rm: RainmeterContext);
}

/// Glue macro to expose your Rust `RainmeterPlugin` implementation
/// as the six C ABI entry points Rainmeter expects.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin:ty) => {
        // Wrap everything in a module to avoid polluting the parent namespace
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod plugin_entry {
            use crate::{RainmeterContext, RainmeterPlugin};
            use std::ffi::OsStr;
            use std::ffi::c_void;
            use std::mem;
            use std::os::windows::ffi::OsStrExt;
            use std::panic;
            use std::panic::AssertUnwindSafe;
            use windows::core::BOOL;
            use windows::core::PCWSTR;

            #[repr(C)]
            struct PluginEntry {
                plugin: $plugin,
                rm_raw: *mut c_void,
            }

            fn log_panic(rm_raw: *mut c_void, fn_name: &str, err: Box<dyn std::any::Any + Send>) {
                let msg = if let Some(s) = err.downcast_ref::<&str>() {
                    format!("Panic in {}: {}", fn_name, s)
                } else if let Some(s) = err.downcast_ref::<String>() {
                    format!("Panic in {}: {}", fn_name, s)
                } else {
                    format!("Panic in {}: <non-string>", fn_name)
                };
                let ctx = RainmeterContext::new(rm_raw);
                ctx.log(rainmeter::RmLogLevel::LogError, &msg); // LOG_ERROR level = 1
            }

            #[unsafe(no_mangle)]
            pub extern "stdcall" fn Initialize(data: *mut *mut c_void, rm: *mut c_void) {
                let mut entry = Box::new(PluginEntry {
                    plugin: <$plugin>::default(),
                    rm_raw: rm,
                });
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    entry.plugin.initialize(RainmeterContext::new(rm));
                }));
                if let Err(err) = result {
                    log_panic(rm, "Initialize", err);
                }
                let ptr = Box::into_raw(entry) as *mut c_void;
                unsafe { *data = ptr };
            }

            #[unsafe(no_mangle)]
            pub extern "stdcall" fn Reload(
                data: *mut c_void,
                rm: *mut c_void,
                max_value: *mut f64,
            ) {
                let mut entry = unsafe { &mut *(data as *mut PluginEntry) };
                entry.rm_raw = rm;
                let mut default = unsafe { *max_value };
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    entry.plugin.reload(RainmeterContext::new(rm), &mut default);
                }));
                if let Err(err) = result {
                    log_panic(rm, "Reload", err);
                }
                unsafe { *max_value = default };
            }

            #[unsafe(no_mangle)]
            pub extern "stdcall" fn Update(data: *mut c_void) -> f64 {
                let mut entry = unsafe { &mut *(data as *mut PluginEntry) };
                let mut ret = 0.0;
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    ret = entry.plugin.update(RainmeterContext::new(entry.rm_raw));
                }));
                if let Err(err) = result {
                    log_panic(entry.rm_raw, "Update", err);
                }
                ret
            }

            #[unsafe(no_mangle)]
            pub extern "stdcall" fn GetString(data: *mut c_void) -> PCWSTR {
                let mut entry = unsafe { &mut *(data as *mut PluginEntry) };
                let mut out_ptr = std::ptr::null();
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    if let Some(s) = entry.plugin.get_string(RainmeterContext::new(entry.rm_raw)) {
                        let mut wide: Vec<u16> =
                            OsStr::new(&s).encode_wide().chain(Some(0)).collect();
                        out_ptr = wide.as_mut_ptr();
                        mem::forget(wide);
                    }
                }));
                if let Err(err) = result {
                    log_panic(entry.rm_raw, "GetString", err);
                }
                PCWSTR(out_ptr)
            }

            #[unsafe(no_mangle)]
            pub extern "stdcall" fn ExecuteBang(data: *mut c_void, args: PCWSTR) {
                let mut entry = unsafe { &mut *(data as *mut PluginEntry) };
                let mut arg_string = String::new();
                if !args.is_null() {
                    let mut len = 0;
                    unsafe {
                        while *args.0.add(len) != 0 {
                            len += 1;
                        }
                        arg_string =
                            String::from_utf16_lossy(std::slice::from_raw_parts(args.0, len));
                    }
                }
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    entry
                        .plugin
                        .execute_bang(RainmeterContext::new(entry.rm_raw), &arg_string);
                }));
                if let Err(err) = result {
                    log_panic(entry.rm_raw, "ExecuteBang", err);
                }
            }

            #[unsafe(no_mangle)]
            pub extern "stdcall" fn Finalize(data: *mut c_void) {
                let mut entry = unsafe { Box::from_raw(data as *mut PluginEntry) };
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    entry.plugin.finalize(RainmeterContext::new(entry.rm_raw));
                }));
                if let Err(err) = result {
                    log_panic(entry.rm_raw, "Finalize", err);
                }
            }
        }
    };
}

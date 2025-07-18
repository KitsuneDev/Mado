//! High-level Rust interface to the Rainmeter C/C++ plugin API.
//! Simply implement the `RainmeterPlugin` trait in your plugin,
//! and use the `declare_plugin!` macro to expose it to Rainmeter.
//! Don't forget to add to your `Cargo.toml`:
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]```

use std::ffi::{OsStr, c_void};
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::HWND;
use windows::core::{BOOL, PCWSTR};

// -----------------------------------------------------------------------
// FFI declarations of host‑provided Rainmeter API functions
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

// Additional cdecl APIs for formatted and deprecated logging
#[link(name = "Rainmeter")]
unsafe extern "C" {
    pub fn RmLogF(rm: *mut c_void, level: i32, format: PCWSTR, ...);
    pub fn LSLog(level: i32, unused: PCWSTR, message: PCWSTR) -> BOOL;
}

// -----------------------------------------------------------------------
// Helpers: wide‑string conversion
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
    let mut len = 0;
    while *ptr.0.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len))
}

/// Log levels matching Rainmeter's LOG_* constants
pub enum RmLogLevel {
    LogError = 1,
    LogWarning = 2,
    LogNotice = 3,
    LogDebug = 4,
}

/// Types for data retrieval via RmGet()
pub enum RmGetType {
    MeasureName = 0,
    Skin = 1,
    SettingsFile = 2,
    SkinName = 3,
    SkinWindowHandle = 4,
}

// -----------------------------------------------------------------------
// High‑level Rust wrapper around the raw Rainmeter context pointer.
// -----------------------------------------------------------------------

pub struct RainmeterContext {
    raw: *mut c_void,
}

impl RainmeterContext {
    /// Create a new context from the raw `rm` pointer.
    pub fn new(raw: *mut c_void) -> Self {
        Self { raw }
    }

    // --- Section readers ---
    pub fn read_string(&self, key: &str, default: &str) -> String {
        let k = to_pcwstr(key);
        let d = to_pcwstr(default);
        let ptr = unsafe { RmReadString(self.raw, k, d, BOOL(1)) };
        unsafe { from_pcwstr(ptr) }
    }

    pub fn read_string_section(&self, section: &str, key: &str, default: &str) -> String {
        let s = to_pcwstr(section);
        let k = to_pcwstr(key);
        let d = to_pcwstr(default);
        let ptr = unsafe { RmReadStringFromSection(self.raw, s, k, d, BOOL(1)) };
        unsafe { from_pcwstr(ptr) }
    }

    pub fn read_formula(&self, key: &str, default: f64) -> f64 {
        let k = to_pcwstr(key);
        unsafe { RmReadFormula(self.raw, k, default) }
    }

    pub fn read_formula_section(&self, section: &str, key: &str, default: f64) -> f64 {
        let s = to_pcwstr(section);
        let k = to_pcwstr(key);
        unsafe { RmReadFormulaFromSection(self.raw, s, k, default) }
    }

    pub fn read_int(&self, key: &str, default: i32) -> i32 {
        self.read_formula(key, default as f64) as i32
    }

    pub fn read_int_section(&self, section: &str, key: &str, default: i32) -> i32 {
        self.read_formula_section(section, key, default as f64) as i32
    }

    pub fn read_double(&self, key: &str, default: f64) -> f64 {
        self.read_formula(key, default)
    }

    pub fn read_double_section(&self, section: &str, key: &str, default: f64) -> f64 {
        self.read_formula_section(section, key, default)
    }

    pub fn replace_variables(&self, input: &str) -> String {
        let i = to_pcwstr(input);
        let ptr = unsafe { RmReplaceVariables(self.raw, i) };
        unsafe { from_pcwstr(ptr) }
    }

    pub fn path_to_absolute(&self, relative: &str) -> String {
        let r = to_pcwstr(relative);
        let ptr = unsafe { RmPathToAbsolute(self.raw, r) };
        unsafe { from_pcwstr(ptr) }
    }

    pub fn read_path(&self, key: &str, default: &str) -> String {
        let rel = self.read_string(key, default);
        self.path_to_absolute(&rel)
    }

    pub fn execute(&self, command: &str) {
        let c = to_pcwstr(command);
        unsafe { RmExecute(self.raw, c) };
    }

    /// Raw RmGet with integer code
    pub fn get_raw(&self, what: RmGetType) -> *mut c_void {
        unsafe { RmGet(self.raw, what as i32) }
    }

    /// Retrieve raw PCWSTR for measure name
    pub fn get_measure_name_raw(&self) -> PCWSTR {
        PCWSTR(self.get_raw(RmGetType::MeasureName) as _)
    }

    /// Measure name as Rust String
    pub fn get_measure_name(&self) -> String {
        let ptr = self.get_measure_name_raw();
        unsafe { from_pcwstr(ptr) }
    }

    /// Raw skin pointer (void*)
    pub fn get_skin_raw(&self) -> *mut c_void {
        self.get_raw(RmGetType::Skin)
    }

    /// Skin pointer (void*)
    pub fn get_skin(&self) -> *mut c_void {
        self.get_skin_raw()
    }

    /// Raw PCWSTR for settings file path
    pub fn get_settings_file_raw(&self) -> PCWSTR {
        PCWSTR(unsafe { RmGet(std::ptr::null_mut(), RmGetType::SettingsFile as i32) } as _)
    }

    /// Settings file path as Rust String
    pub fn get_settings_file(&self) -> String {
        let ptr = self.get_settings_file_raw();
        unsafe { from_pcwstr(ptr) }
    }

    /// Raw PCWSTR for skin name
    pub fn get_skin_name_raw(&self) -> PCWSTR {
        PCWSTR(self.get_raw(RmGetType::SkinName) as _)
    }

    /// Skin name as Rust String
    pub fn get_skin_name(&self) -> String {
        let ptr = self.get_skin_name_raw();
        unsafe { from_pcwstr(ptr) }
    }

    /// Raw window-handle pointer
    pub fn get_skin_window_raw(&self) -> *mut c_void {
        self.get_raw(RmGetType::SkinWindowHandle)
    }

    /// HWND of the skin window
    pub fn get_skin_window(&self) -> HWND {
        HWND(self.get_skin_window_raw())
    }

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
    fn initialize(&mut self, rm: RainmeterContext);
    fn reload(&mut self, rm: RainmeterContext, max_value: &mut f64);
    fn update(&mut self, rm: RainmeterContext) -> f64;
    fn get_string(&mut self, rm: RainmeterContext) -> Option<String> {
        None
    }
    fn execute_bang(&mut self, rm: RainmeterContext, args: &str) {}
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

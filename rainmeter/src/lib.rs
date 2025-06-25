
use std::cell::RefCell;
use once_cell::unsync::OnceCell;

#[allow(non_snake_case)]
pub trait RainmeterPlugin: 'static {
    fn initialize(&mut self) {}
    fn update(&mut self) -> f64;
    fn set_option(&mut self, _key: &str, _val: &str) {}
    fn reload(&mut self) {}
    fn finalize(&mut self) {}
}

// Thread-local storage for the plugin instance
thread_local! {
    pub(crate) static PLUGIN: RefCell<OnceCell<Box<dyn RainmeterPlugin>>> = RefCell::new(OnceCell::new());
}

/// Access the plugin instance
pub fn get_plugin<'a>() -> &'a mut dyn RainmeterPlugin {
    let ptr = PLUGIN.with(|cell| {
        let cell = cell.borrow();
        let boxed = cell.get().expect("Plugin not initialized");
        boxed.as_ref() as *const dyn RainmeterPlugin as *mut dyn RainmeterPlugin
    });
    unsafe { &mut *ptr }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin:ty) => {
        use std::ffi::{c_void, CStr};
        use std::cell::RefCell;
        use once_cell::unsync::OnceCell;

        // Define a static mutable plugin instance
        static mut PLUGIN_INSTANCE: Option<Box<$plugin>> = None;

        // Function to safely access the plugin instance
        fn get_plugin<'a>() -> &'a mut dyn $crate::RainmeterPlugin {
            unsafe {
                PLUGIN_INSTANCE.as_mut().expect("Plugin not initialized").as_mut()
            }
        }

        #[unsafe(no_mangle)]
        pub extern "stdcall" fn Initialize(data: *mut *mut c_void, _rm: *mut c_void) {
            let mut plugin = <$plugin>::default();
            plugin.initialize();

            // Store the plugin instance
            unsafe {
                PLUGIN_INSTANCE = Some(Box::new(plugin));
                // Get a reference to the plugin instance
                let plugin_ref = PLUGIN_INSTANCE.as_ref().unwrap().as_ref() as &dyn $crate::RainmeterPlugin;
                // Cast to a raw pointer with explicit types
                let raw_ptr = plugin_ref as *const dyn $crate::RainmeterPlugin;
                let void_ptr = raw_ptr as *const c_void;
                *data = void_ptr as *mut c_void;
            }
        }

        #[unsafe(no_mangle)]
        pub extern "stdcall" fn Update(_data: *mut c_void) -> f64 {
            get_plugin().update()
        }

        #[unsafe(no_mangle)]
        pub extern "stdcall" fn SetOption(_data: *mut c_void, key: *const u8, val: *const u8) {
            let k = unsafe { CStr::from_ptr(key as *const i8) }.to_string_lossy();
            let v = unsafe { CStr::from_ptr(val as *const i8) }.to_string_lossy();
            get_plugin().set_option(&k, &v);
        }

        #[unsafe(no_mangle)]
        pub extern "stdcall" fn Reload(_data: *mut c_void) {
            get_plugin().reload();
        }

        #[unsafe(no_mangle)]
        pub extern "stdcall" fn Finalize(_data: *mut c_void) {
            get_plugin().finalize();

            // Clean up the plugin instance
            unsafe {
                PLUGIN_INSTANCE = None;
            }
        }
    };
}

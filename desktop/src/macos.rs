//! macOS-specific functionality for handling file open events.
//!
//! This module injects an `application:openFiles:` method into winit's
//! application delegate to handle file open events from Finder.

use crate::custom_event::RuffleEvent;
use crate::player::LaunchOptions;
use crate::preferences::GlobalPreferences;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Bool, Sel};
use objc2::{class, msg_send, sel};
use objc2_app_kit::NSApplication;
use objc2_foundation::NSString;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use url::Url;
use winit::event_loop::EventLoopProxy;

/// Global state for the file open handler.
static HANDLER_STATE: OnceLock<Mutex<HandlerState>> = OnceLock::new();

struct HandlerState {
    /// URLs received before the event loop was ready
    queued_urls: Vec<Url>,
    /// Event loop proxy (set when event loop is ready)
    event_loop_proxy: Option<EventLoopProxy<RuffleEvent>>,
    /// Preferences (set when event loop is ready)
    preferences: Option<GlobalPreferences>,
}

/// Injects the file open handler into winit's application delegate.
///
/// This must be called AFTER EventLoop::new() so that winit's delegate exists,
/// but BEFORE EventLoop::run() so we catch any pending file open events.
pub fn setup_file_open_handler(
    event_loop_proxy: EventLoopProxy<RuffleEvent>,
    preferences: GlobalPreferences,
) {
    eprintln!("[ruffle-macos] setup_file_open_handler called");

    // Initialize the handler state
    let _ = HANDLER_STATE.set(Mutex::new(HandlerState {
        queued_urls: Vec::new(),
        event_loop_proxy: Some(event_loop_proxy),
        preferences: Some(preferences),
    }));

    // Get NSApp and its delegate
    unsafe {
        let app: Option<Retained<NSApplication>> = msg_send![class!(NSApplication), sharedApplication];
        let Some(app) = app else {
            eprintln!("[ruffle-macos] NSApplication not available");
            return;
        };

        let delegate: *mut AnyObject = msg_send![&app, delegate];
        if delegate.is_null() {
            eprintln!("[ruffle-macos] No application delegate found");
            return;
        }

        // Get the delegate's class
        let delegate_class: *const AnyClass = msg_send![delegate, class];
        if delegate_class.is_null() {
            eprintln!("[ruffle-macos] Could not get delegate class");
            return;
        }

        // Get the class name for debugging
        let class_name: *const std::ffi::c_char = msg_send![delegate_class, name];
        if !class_name.is_null() {
            let name = std::ffi::CStr::from_ptr(class_name);
            eprintln!("[ruffle-macos] Delegate class name: {:?}", name);
        }

        // Check if the method already exists
        let selector = sel!(application:openFiles:);
        let responds: Bool = msg_send![delegate, respondsToSelector: selector];
        eprintln!("[ruffle-macos] Delegate responds to application:openFiles: {}", responds.as_bool());

        // Add our method to the delegate's class
        let method_added = add_open_files_method(delegate_class as *mut AnyClass);
        eprintln!("[ruffle-macos] Method injection result: {}", method_added);
    }
}

/// Adds the application:openFiles: method to the given class.
unsafe fn add_open_files_method(class: *mut AnyClass) -> bool {
    // The type encoding for - (void)application:(NSApplication *)app openFiles:(NSArray<NSString *> *)files
    // v = void return, @ = object (self), : = selector, @ = NSApplication*, @ = NSArray*
    let types = c"v@:@@";

    // Get the selector
    let sel: Sel = Sel::register(c"application:openFiles:");

    // Cast the handler function to the expected function pointer type
    let imp: unsafe extern "C-unwind" fn() = std::mem::transmute::<
        unsafe extern "C" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject),
        unsafe extern "C-unwind" fn(),
    >(open_files_handler);

    // Use class_addMethod from objc runtime
    let success: Bool = objc2::ffi::class_addMethod(
        class as *mut _,
        std::mem::transmute(sel),
        imp,
        types.as_ptr(),
    );

    if success.as_bool() {
        eprintln!("[ruffle-macos] Successfully added application:openFiles: method");
    } else {
        eprintln!("[ruffle-macos] Failed to add method (may already exist)");
        // If the method already exists, we might need to replace it with method_setImplementation
        // But that's risky - for now just log and continue
    }

    success.as_bool()
}

/// The actual handler for application:openFiles:
unsafe extern "C" fn open_files_handler(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    files: *mut AnyObject,
) {
    eprintln!("[ruffle-macos] application:openFiles: called!");

    let Some(state) = HANDLER_STATE.get() else {
        eprintln!("[ruffle-macos] Handler state not initialized");
        return;
    };

    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());

    // files is an NSArray<NSString *>
    if files.is_null() {
        eprintln!("[ruffle-macos] files array is null");
        return;
    }

    // Get count using msg_send
    let count: usize = msg_send![files, count];
    eprintln!("[ruffle-macos] Received {} files", count);

    for i in 0..count {
        // Get object at index using msg_send
        let file: *mut AnyObject = msg_send![files, objectAtIndex: i];
        if file.is_null() {
            continue;
        }

        // Cast to NSString and get the string value
        let file_str: &NSString = unsafe { &*(file as *const NSString) };
        let path_str: String = file_str.to_string();
        eprintln!("[ruffle-macos] File {}: {}", i, path_str);

        let path = PathBuf::from(&path_str);
        if let Ok(url) = Url::from_file_path(&path) {
            if let (Some(proxy), Some(prefs)) = (&state.event_loop_proxy, &state.preferences) {
                eprintln!("[ruffle-macos] Sending file to event loop: {}", url);
                let launch_options = LaunchOptions::from(prefs);
                let _ = proxy.send_event(RuffleEvent::Open(url, Box::new(launch_options)));
            } else {
                eprintln!("[ruffle-macos] Event loop not ready, queueing: {}", url);
                state.queued_urls.push(url);
            }
        }
    }
}

#![allow(unexpected_cfgs)]

#[macro_use]
extern crate objc;

use objc::runtime::{Object, Sel};
use std::ffi::CString;
use std::os::raw::c_char;

type Id = *mut Object;

const NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY: isize = 1;
const NS_VARIABLE_STATUS_ITEM_LENGTH: f64 = -1.0;

fn main() {
    env_logger::init();
    log::info!("rs_ice starting...");

    unsafe {
        let app = shared_application();
        let _: bool =
            msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY];
        create_status_item(app);

        log::info!("rs_ice ready.");
        let _: () = msg_send![app, run];
    }
}

unsafe fn shared_application() -> Id {
    msg_send![class!(NSApplication), sharedApplication]
}

unsafe fn create_status_item(app: Id) {
    let status_bar: Id = msg_send![class!(NSStatusBar), systemStatusBar];
    let item: Id = msg_send![status_bar, statusItemWithLength: NS_VARIABLE_STATUS_ITEM_LENGTH];
    let button: Id = msg_send![item, button];

    if !button.is_null() {
        let image: Id = msg_send![
            class!(NSImage),
            imageWithSystemSymbolName: ns_string("snowflake")
            accessibilityDescription: ns_string("Ice")
        ];

        if !image.is_null() {
            let _: () = msg_send![button, setImage: image];
        } else {
            let _: () = msg_send![button, setTitle: ns_string("Ice")];
        }
    }

    let menu = build_menu(app);
    let _: () = msg_send![item, setMenu: menu];
}

unsafe fn build_menu(app: Id) -> Id {
    let menu: Id = msg_send![class!(NSMenu), new];
    let quit = menu_item("Quit Ice", sel!(terminate:), "q");
    let _: () = msg_send![quit, setTarget: app];
    let _: () = msg_send![menu, addItem: quit];
    menu
}

unsafe fn menu_item(title: &str, action: Sel, key: &str) -> Id {
    let item: Id = msg_send![class!(NSMenuItem), alloc];
    msg_send![
        item,
        initWithTitle: ns_string(title)
        action: action
        keyEquivalent: ns_string(key)
    ]
}

unsafe fn ns_string(value: &str) -> Id {
    let value = CString::new(value).expect("NSString input must not contain interior nul bytes");
    let string: Id = msg_send![class!(NSString), alloc];
    msg_send![string, initWithUTF8String: value.as_ptr() as *const c_char]
}

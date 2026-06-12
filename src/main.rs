#![allow(unexpected_cfgs)]

#[macro_use]
extern crate objc;

use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use rs_ice::app_state::AppState;
use rs_ice::menu_model::MenuSnapshot;
use rs_ice::settings::{RehideStrategy, SettingsStore};
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

type Id = *mut Object;

const NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY: isize = 1;
const NS_VARIABLE_STATUS_ITEM_LENGTH: f64 = -1.0;
const NS_CONTROL_STATE_VALUE_OFF: i64 = 0;
const NS_CONTROL_STATE_VALUE_ON: i64 = 1;

static RUNTIME: OnceLock<Mutex<AppRuntime>> = OnceLock::new();

struct AppRuntime {
    app: usize,
    status_item: usize,
    target: usize,
    state: AppState,
    store: CocoaSettingsStore,
}

fn main() {
    env_logger::init();
    log::info!("rs_ice starting...");

    unsafe {
        let target_class = register_menu_target_class();
        let app = shared_application();
        let _: bool =
            msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY];
        let status_item = create_status_item();
        let target: Id = msg_send![target_class, new];
        let store = CocoaSettingsStore;
        let state = AppState::load(&store);

        RUNTIME
            .set(Mutex::new(AppRuntime {
                app: app as usize,
                status_item: status_item as usize,
                target: target as usize,
                state,
                store,
            }))
            .unwrap_or_else(|_| panic!("runtime should be initialized once"));

        rebuild_status_item();

        log::info!("rs_ice ready.");
        let _: () = msg_send![app, run];
    }
}

unsafe fn shared_application() -> Id {
    msg_send![class!(NSApplication), sharedApplication]
}

unsafe fn create_status_item() -> Id {
    let status_bar: Id = msg_send![class!(NSStatusBar), systemStatusBar];
    let item: Id = msg_send![status_bar, statusItemWithLength: NS_VARIABLE_STATUS_ITEM_LENGTH];
    let _: () = msg_send![item, setAutosaveName: ns_string("rs_ice.statusItem")];
    item
}

unsafe fn rebuild_status_item() {
    let mut runtime = RUNTIME
        .get()
        .expect("runtime must exist before rebuilding the menu")
        .lock()
        .expect("runtime mutex should not be poisoned");
    runtime.state.tick(Instant::now());

    let status_item = runtime.status_item as Id;
    let target = runtime.target as Id;
    let app = runtime.app as Id;
    let snapshot = MenuSnapshot::from_state(&runtime.state);

    configure_status_button(status_item, &snapshot);
    let menu = build_menu(app, target, &snapshot);
    let _: () = msg_send![status_item, setMenu: menu];
}

unsafe fn configure_status_button(item: Id, snapshot: &MenuSnapshot) {
    let button: Id = msg_send![item, button];

    if !button.is_null() {
        if snapshot.ice_icon_visible {
            let image: Id = msg_send![
                class!(NSImage),
                imageWithSystemSymbolName: ns_string("snowflake")
                accessibilityDescription: ns_string("Ice")
            ];

            if !image.is_null() {
                let _: () = msg_send![button, setImage: image];
                let _: () = msg_send![button, setTitle: ns_string("")];
            } else {
                let _: () = msg_send![button, setTitle: ns_string("Ice")];
            }
        } else {
            let _: () = msg_send![button, setImage: std::ptr::null_mut::<Object>()];
            let _: () = msg_send![button, setTitle: ns_string("Ice")];
        }
    }
}

unsafe fn build_menu(app: Id, target: Id, snapshot: &MenuSnapshot) -> Id {
    let menu: Id = msg_send![class!(NSMenu), new];

    add_item(
        menu,
        &format!(
            "{} Hidden Section",
            if snapshot.hidden_section_visible {
                "Hide"
            } else {
                "Show"
            }
        ),
        sel!(toggleHiddenSection:),
        "",
        target,
    );
    add_separator(menu);

    add_check_item(
        menu,
        "Show Ice Icon",
        snapshot.ice_icon_visible,
        sel!(toggleShowIceIcon:),
        target,
    );
    add_check_item(
        menu,
        "Show On Click",
        snapshot.show_on_click,
        sel!(toggleShowOnClick:),
        target,
    );
    add_check_item(
        menu,
        "Auto Rehide",
        snapshot.auto_rehide,
        sel!(toggleAutoRehide:),
        target,
    );
    add_check_item(
        menu,
        "Context Menu On Right Click",
        snapshot.show_context_menu_on_right_click,
        sel!(toggleContextMenuOnRightClick:),
        target,
    );
    add_check_item(
        menu,
        "Hide Application Menus",
        snapshot.hide_application_menus,
        sel!(toggleHideApplicationMenus:),
        target,
    );
    add_check_item(
        menu,
        "Show Section Dividers",
        snapshot.show_section_dividers,
        sel!(toggleShowSectionDividers:),
        target,
    );
    add_check_item(
        menu,
        "Enable Always-Hidden Section",
        snapshot.enable_always_hidden_section,
        sel!(toggleEnableAlwaysHiddenSection:),
        target,
    );
    add_check_item(
        menu,
        "Always-Hidden Section Can Be Shown",
        snapshot.can_toggle_always_hidden_section,
        sel!(toggleCanToggleAlwaysHiddenSection:),
        target,
    );
    add_check_item(
        menu,
        "Show All Sections On User Drag",
        snapshot.show_all_sections_on_user_drag,
        sel!(toggleShowAllSectionsOnUserDrag:),
        target,
    );

    let strategy_menu: Id = msg_send![class!(NSMenu), new];
    for strategy in RehideStrategy::ALL {
        add_check_item(
            strategy_menu,
            strategy.title(),
            snapshot.rehide_strategy == strategy,
            strategy_selector(strategy),
            target,
        );
    }
    add_submenu(menu, "Rehide Strategy", strategy_menu);

    let interval_menu: Id = msg_send![class!(NSMenu), new];
    for secs in [5.0, 10.0, 15.0, 30.0, 60.0] {
        add_check_item(
            interval_menu,
            &format!("{secs:.0} seconds"),
            (snapshot.rehide_interval_secs - secs).abs() < f64::EPSILON,
            interval_selector(secs),
            target,
        );
    }
    add_submenu(menu, "Rehide Interval", interval_menu);

    add_separator(menu);

    let quit = menu_item("Quit Ice", sel!(terminate:), "q");
    let _: () = msg_send![quit, setTarget: app];
    let _: () = msg_send![menu, addItem: quit];
    menu
}

unsafe fn add_item(menu: Id, title: &str, action: Sel, key: &str, target: Id) {
    let item = menu_item(title, action, key);
    let _: () = msg_send![item, setTarget: target];
    let _: () = msg_send![menu, addItem: item];
}

unsafe fn add_check_item(menu: Id, title: &str, checked: bool, action: Sel, target: Id) {
    let item = menu_item(title, action, "");
    let _: () = msg_send![item, setTarget: target];
    let state = if checked {
        NS_CONTROL_STATE_VALUE_ON
    } else {
        NS_CONTROL_STATE_VALUE_OFF
    };
    let _: () = msg_send![item, setState: state];
    let _: () = msg_send![menu, addItem: item];
}

unsafe fn add_submenu(menu: Id, title: &str, submenu: Id) {
    let item = menu_item(title, sel!(noop:), "");
    let _: () = msg_send![item, setSubmenu: submenu];
    let _: () = msg_send![menu, addItem: item];
}

unsafe fn add_separator(menu: Id) {
    let item: Id = msg_send![class!(NSMenuItem), separatorItem];
    let _: () = msg_send![menu, addItem: item];
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

fn strategy_selector(strategy: RehideStrategy) -> Sel {
    match strategy {
        RehideStrategy::Smart => sel!(setRehideStrategySmart:),
        RehideStrategy::Timed => sel!(setRehideStrategyTimed:),
        RehideStrategy::FocusedApp => sel!(setRehideStrategyFocusedApp:),
    }
}

fn interval_selector(secs: f64) -> Sel {
    match secs as i64 {
        5 => sel!(setRehideInterval5:),
        10 => sel!(setRehideInterval10:),
        15 => sel!(setRehideInterval15:),
        30 => sel!(setRehideInterval30:),
        60 => sel!(setRehideInterval60:),
        _ => sel!(setRehideInterval15:),
    }
}

unsafe fn register_menu_target_class() -> *const objc::runtime::Class {
    if let Some(class) = objc::runtime::Class::get("RsIceMenuTarget") {
        return class;
    }

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("RsIceMenuTarget", superclass)
        .expect("menu target class should be registered once");

    decl.add_method(
        sel!(toggleHiddenSection:),
        toggle_hidden_section as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowIceIcon:),
        toggle_show_ice_icon as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowOnClick:),
        toggle_show_on_click as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleAutoRehide:),
        toggle_auto_rehide as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleContextMenuOnRightClick:),
        toggle_context_menu_on_right_click as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleHideApplicationMenus:),
        toggle_hide_application_menus as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowSectionDividers:),
        toggle_show_section_dividers as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleEnableAlwaysHiddenSection:),
        toggle_enable_always_hidden_section as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleCanToggleAlwaysHiddenSection:),
        toggle_can_toggle_always_hidden_section as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowAllSectionsOnUserDrag:),
        toggle_show_all_sections_on_user_drag as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideStrategySmart:),
        set_rehide_strategy_smart as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideStrategyTimed:),
        set_rehide_strategy_timed as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideStrategyFocusedApp:),
        set_rehide_strategy_focused_app as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideInterval5:),
        set_rehide_interval_5 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideInterval10:),
        set_rehide_interval_10 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideInterval15:),
        set_rehide_interval_15 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideInterval30:),
        set_rehide_interval_30 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setRehideInterval60:),
        set_rehide_interval_60 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(rehideTimerFired:),
        rehide_timer_fired as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(sel!(noop:), noop as extern "C" fn(&Object, Sel, Id));
    decl.register()
}

extern "C" fn toggle_hidden_section(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| runtime.state.toggle_hidden_section());
}

extern "C" fn toggle_show_ice_icon(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_show_ice_icon(&mut runtime.store);
    });
}

extern "C" fn toggle_show_on_click(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_show_on_click(&mut runtime.store);
    });
}

extern "C" fn toggle_auto_rehide(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_auto_rehide(&mut runtime.store);
    });
}

extern "C" fn toggle_context_menu_on_right_click(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_context_menu_on_right_click(&mut runtime.store);
    });
}

extern "C" fn toggle_hide_application_menus(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_hide_application_menus(&mut runtime.store);
    });
}

extern "C" fn toggle_show_section_dividers(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_show_section_dividers(&mut runtime.store);
    });
}

extern "C" fn toggle_enable_always_hidden_section(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_enable_always_hidden_section(&mut runtime.store);
    });
}

extern "C" fn toggle_can_toggle_always_hidden_section(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_can_toggle_always_hidden_section(&mut runtime.store);
    });
}

extern "C" fn toggle_show_all_sections_on_user_drag(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_show_all_sections_on_user_drag(&mut runtime.store);
    });
}

extern "C" fn set_rehide_strategy_smart(_: &Object, _: Sel, _: Id) {
    set_rehide_strategy(RehideStrategy::Smart);
}

extern "C" fn set_rehide_strategy_timed(_: &Object, _: Sel, _: Id) {
    set_rehide_strategy(RehideStrategy::Timed);
}

extern "C" fn set_rehide_strategy_focused_app(_: &Object, _: Sel, _: Id) {
    set_rehide_strategy(RehideStrategy::FocusedApp);
}

extern "C" fn set_rehide_interval_5(_: &Object, _: Sel, _: Id) {
    set_rehide_interval(5.0);
}

extern "C" fn set_rehide_interval_10(_: &Object, _: Sel, _: Id) {
    set_rehide_interval(10.0);
}

extern "C" fn set_rehide_interval_15(_: &Object, _: Sel, _: Id) {
    set_rehide_interval(15.0);
}

extern "C" fn set_rehide_interval_30(_: &Object, _: Sel, _: Id) {
    set_rehide_interval(30.0);
}

extern "C" fn set_rehide_interval_60(_: &Object, _: Sel, _: Id) {
    set_rehide_interval(60.0);
}

extern "C" fn rehide_timer_fired(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| runtime.state.tick(Instant::now()));
}

extern "C" fn noop(_: &Object, _: Sel, _: Id) {}

fn set_rehide_strategy(strategy: RehideStrategy) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .set_rehide_strategy(&mut runtime.store, strategy);
    });
}

fn set_rehide_interval(secs: f64) {
    mutate_runtime(|runtime| {
        runtime.state.set_rehide_interval(&mut runtime.store, secs);
    });
}

fn mutate_runtime(body: impl FnOnce(&mut AppRuntime)) {
    let timer = {
        let mut runtime = RUNTIME
            .get()
            .expect("runtime must exist before menu actions")
            .lock()
            .expect("runtime mutex should not be poisoned");
        body(&mut runtime);
        runtime
            .state
            .rehide_deadline()
            .map(|deadline| (deadline, runtime.target as Id))
    };

    if let Some((deadline, target)) = timer {
        schedule_rehide_timer(deadline, target);
    }

    unsafe {
        rebuild_status_item();
    }
}

fn schedule_rehide_timer(deadline: Instant, target: Id) {
    let delay = deadline.saturating_duration_since(Instant::now());
    unsafe {
        let _: Id = msg_send![
            class!(NSTimer),
            scheduledTimerWithTimeInterval: delay.as_secs_f64()
            target: target
            selector: sel!(rehideTimerFired:)
            userInfo: std::ptr::null_mut::<Object>()
            repeats: false
        ];
    }
}

struct CocoaSettingsStore;

impl CocoaSettingsStore {
    unsafe fn defaults(&self) -> Id {
        msg_send![class!(NSUserDefaults), standardUserDefaults]
    }

    unsafe fn key(value: &str) -> Id {
        ns_string(value)
    }

    unsafe fn has_key(&self, key: &str) -> bool {
        let object: Id = msg_send![self.defaults(), objectForKey: Self::key(key)];
        !object.is_null()
    }
}

impl SettingsStore for CocoaSettingsStore {
    fn bool_for_key(&self, key: &str) -> Option<bool> {
        unsafe {
            if !self.has_key(key) {
                return None;
            }
            let value: bool = msg_send![self.defaults(), boolForKey: Self::key(key)];
            Some(value)
        }
    }

    fn integer_for_key(&self, key: &str) -> Option<i64> {
        unsafe {
            if !self.has_key(key) {
                return None;
            }
            let value: i64 = msg_send![self.defaults(), integerForKey: Self::key(key)];
            Some(value)
        }
    }

    fn double_for_key(&self, key: &str) -> Option<f64> {
        unsafe {
            if !self.has_key(key) {
                return None;
            }
            let value: f64 = msg_send![self.defaults(), doubleForKey: Self::key(key)];
            Some(value)
        }
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        unsafe {
            let _: () = msg_send![self.defaults(), setBool: value forKey: Self::key(key)];
        }
    }

    fn set_integer(&mut self, key: &str, value: i64) {
        unsafe {
            let _: () = msg_send![self.defaults(), setInteger: value forKey: Self::key(key)];
        }
    }

    fn set_double(&mut self, key: &str, value: f64) {
        unsafe {
            let _: () = msg_send![self.defaults(), setDouble: value forKey: Self::key(key)];
        }
    }
}

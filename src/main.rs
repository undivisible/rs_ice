#![allow(unexpected_cfgs)]

#[macro_use]
extern crate objc;

use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use rs_ice::app_state::AppState;
use rs_ice::menu_model::MenuSnapshot;
use rs_ice::permissions::PermissionChecker;
use rs_ice::settings::{IceBarLocation, RehideStrategy, SettingsStore};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

type Id = *mut Object;

const NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY: isize = 1;
const NS_VARIABLE_STATUS_ITEM_LENGTH: f64 = -1.0;
const NS_CONTROL_STATE_VALUE_OFF: i64 = 0;
const NS_CONTROL_STATE_VALUE_ON: i64 = 1;
const NSEVENT_MODIFIER_FLAG_CONTROL: u64 = 1 << 18;

#[repr(C)]
#[derive(Clone, Copy)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

static RUNTIME: OnceLock<Mutex<AppRuntime>> = OnceLock::new();

struct AppRuntime {
    app: usize,
    status_item: usize,
    target: usize,
    state: AppState,
    store: CocoaSettingsStore,
    permissions: CocoaPermissionChecker,
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

#[link(name = "Foundation", kind = "framework")]
extern "C" {}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: Id) -> bool;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
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
        let permissions = CocoaPermissionChecker;
        let mut state = AppState::load(&store);
        state.refresh_permissions(&permissions);
        if !state.permissions().accessibility.granted {
            request_accessibility_permission_prompt();
            state.refresh_permissions(&permissions);
        }

        RUNTIME
            .set(Mutex::new(AppRuntime {
                app: app as usize,
                status_item: status_item as usize,
                target: target as usize,
                state,
                store,
                permissions,
            }))
            .unwrap_or_else(|_| panic!("runtime should be initialized once"));

        rebuild_status_item();
        schedule_hover_timer(target);

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
    let permissions = runtime.permissions;
    runtime.state.refresh_permissions(&permissions);

    let status_item = runtime.status_item as Id;
    let target = runtime.target as Id;
    let snapshot = MenuSnapshot::from_state(&runtime.state);

    configure_status_button(status_item, target, &snapshot);
    let _: () = msg_send![status_item, setMenu: std::ptr::null_mut::<Object>()];
}

unsafe fn configure_status_button(item: Id, target: Id, snapshot: &MenuSnapshot) {
    let button: Id = msg_send![item, button];

    if !button.is_null() {
        let _: () = msg_send![button, setTarget: target];
        let _: () = msg_send![button, setAction: sel!(iceButtonPressed:)];

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
    add_item(
        menu,
        snapshot.always_hidden_toggle_title(),
        sel!(toggleAlwaysHiddenSection:),
        "",
        target,
    );
    add_separator(menu);
    add_item(
        menu,
        snapshot.permissions_title(),
        sel!(refreshPermissions:),
        "",
        target,
    );
    let permissions_menu: Id = msg_send![class!(NSMenu), new];
    add_check_item(
        permissions_menu,
        "Accessibility",
        snapshot.permissions.accessibility.granted,
        sel!(openAccessibilitySettings:),
        target,
    );
    add_check_item(
        permissions_menu,
        "Screen Recording",
        snapshot.permissions.screen_recording.granted,
        sel!(requestScreenRecordingPermission:),
        target,
    );
    add_submenu(menu, "Permissions", permissions_menu);
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
        "Custom Ice Icon Is Template",
        snapshot.custom_ice_icon_is_template,
        sel!(toggleCustomIceIconIsTemplate:),
        target,
    );
    add_check_item(
        menu,
        "Use Ice Bar",
        snapshot.use_ice_bar,
        sel!(toggleUseIceBar:),
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
        "Show On Hover",
        snapshot.show_on_hover,
        sel!(toggleShowOnHover:),
        target,
    );
    add_check_item(
        menu,
        "Show On Scroll",
        snapshot.show_on_scroll,
        sel!(toggleShowOnScroll:),
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

    let ice_bar_location_menu: Id = msg_send![class!(NSMenu), new];
    for location in IceBarLocation::ALL {
        add_check_item(
            ice_bar_location_menu,
            location.title(),
            snapshot.ice_bar_location == location,
            ice_bar_location_selector(location),
            target,
        );
    }
    add_submenu(menu, "Ice Bar Location", ice_bar_location_menu);

    let item_spacing_menu: Id = msg_send![class!(NSMenu), new];
    for offset in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        add_check_item(
            item_spacing_menu,
            &format!("{offset:.0} pt"),
            (snapshot.item_spacing_offset - offset).abs() < f64::EPSILON,
            item_spacing_selector(offset),
            target,
        );
    }
    add_submenu(menu, "Item Spacing Offset", item_spacing_menu);

    let hover_delay_menu: Id = msg_send![class!(NSMenu), new];
    for secs in [0.0, 0.2, 0.5, 1.0] {
        add_check_item(
            hover_delay_menu,
            &format!("{secs:.1} seconds"),
            (snapshot.show_on_hover_delay_secs - secs).abs() < f64::EPSILON,
            hover_delay_selector(secs),
            target,
        );
    }
    add_submenu(menu, "Show On Hover Delay", hover_delay_menu);

    let temp_show_menu: Id = msg_send![class!(NSMenu), new];
    for secs in [5.0, 10.0, 15.0, 30.0, 60.0] {
        add_check_item(
            temp_show_menu,
            &format!("{secs:.0} seconds"),
            (snapshot.temp_show_interval_secs - secs).abs() < f64::EPSILON,
            temp_show_interval_selector(secs),
            target,
        );
    }
    add_submenu(menu, "Temporary Show Interval", temp_show_menu);

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

fn ice_bar_location_selector(location: IceBarLocation) -> Sel {
    match location {
        IceBarLocation::Dynamic => sel!(setIceBarLocationDynamic:),
        IceBarLocation::MousePointer => sel!(setIceBarLocationMousePointer:),
        IceBarLocation::IceIcon => sel!(setIceBarLocationIceIcon:),
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

fn item_spacing_selector(offset: f64) -> Sel {
    match offset as i64 {
        -2 => sel!(setItemSpacingOffsetMinus2:),
        -1 => sel!(setItemSpacingOffsetMinus1:),
        0 => sel!(setItemSpacingOffset0:),
        1 => sel!(setItemSpacingOffset1:),
        2 => sel!(setItemSpacingOffset2:),
        _ => sel!(setItemSpacingOffset0:),
    }
}

fn hover_delay_selector(secs: f64) -> Sel {
    match (secs * 10.0).round() as i64 {
        0 => sel!(setShowOnHoverDelay0:),
        2 => sel!(setShowOnHoverDelay02:),
        5 => sel!(setShowOnHoverDelay05:),
        10 => sel!(setShowOnHoverDelay1:),
        _ => sel!(setShowOnHoverDelay02:),
    }
}

fn temp_show_interval_selector(secs: f64) -> Sel {
    match secs as i64 {
        5 => sel!(setTempShowInterval5:),
        10 => sel!(setTempShowInterval10:),
        15 => sel!(setTempShowInterval15:),
        30 => sel!(setTempShowInterval30:),
        60 => sel!(setTempShowInterval60:),
        _ => sel!(setTempShowInterval15:),
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
        sel!(iceButtonPressed:),
        ice_button_pressed as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleHiddenSection:),
        toggle_hidden_section as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleAlwaysHiddenSection:),
        toggle_always_hidden_section as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(refreshPermissions:),
        refresh_permissions as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(openAccessibilitySettings:),
        open_accessibility_settings as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(requestScreenRecordingPermission:),
        request_screen_recording_permission as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowIceIcon:),
        toggle_show_ice_icon as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleCustomIceIconIsTemplate:),
        toggle_custom_ice_icon_is_template as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleUseIceBar:),
        toggle_use_ice_bar as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowOnClick:),
        toggle_show_on_click as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowOnHover:),
        toggle_show_on_hover as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(toggleShowOnScroll:),
        toggle_show_on_scroll as extern "C" fn(&Object, Sel, Id),
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
        sel!(setIceBarLocationDynamic:),
        set_ice_bar_location_dynamic as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setIceBarLocationMousePointer:),
        set_ice_bar_location_mouse_pointer as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setIceBarLocationIceIcon:),
        set_ice_bar_location_ice_icon as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setItemSpacingOffsetMinus2:),
        set_item_spacing_offset_minus_2 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setItemSpacingOffsetMinus1:),
        set_item_spacing_offset_minus_1 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setItemSpacingOffset0:),
        set_item_spacing_offset_0 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setItemSpacingOffset1:),
        set_item_spacing_offset_1 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setItemSpacingOffset2:),
        set_item_spacing_offset_2 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setShowOnHoverDelay0:),
        set_show_on_hover_delay_0 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setShowOnHoverDelay02:),
        set_show_on_hover_delay_02 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setShowOnHoverDelay05:),
        set_show_on_hover_delay_05 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setShowOnHoverDelay1:),
        set_show_on_hover_delay_1 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setTempShowInterval5:),
        set_temp_show_interval_5 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setTempShowInterval10:),
        set_temp_show_interval_10 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setTempShowInterval15:),
        set_temp_show_interval_15 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setTempShowInterval30:),
        set_temp_show_interval_30 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(setTempShowInterval60:),
        set_temp_show_interval_60 as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(rehideTimerFired:),
        rehide_timer_fired as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(
        sel!(hoverTimerFired:),
        hover_timer_fired as extern "C" fn(&Object, Sel, Id),
    );
    decl.add_method(sel!(noop:), noop as extern "C" fn(&Object, Sel, Id));
    decl.register()
}

extern "C" fn ice_button_pressed(_: &Object, _: Sel, _: Id) {
    if current_event_is_control_click() {
        unsafe {
            show_settings_menu();
        }
        return;
    }

    mutate_runtime(|runtime| {
        if !runtime.state.permissions().accessibility.granted {
            request_accessibility_permission_prompt();
            let permissions = runtime.permissions;
            runtime.state.refresh_permissions(&permissions);
        }
        runtime.state.handle_ice_button_click(Instant::now());
    });
}

extern "C" fn toggle_hidden_section(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| runtime.state.toggle_hidden_section());
}

extern "C" fn toggle_always_hidden_section(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_always_hidden_section(Instant::now());
    });
}

extern "C" fn refresh_permissions(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        let permissions = runtime.permissions;
        runtime.state.refresh_permissions(&permissions);
    });
}

extern "C" fn open_accessibility_settings(_: &Object, _: Sel, _: Id) {
    open_system_settings(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    );
    mutate_runtime(|runtime| {
        let permissions = runtime.permissions;
        runtime.state.refresh_permissions(&permissions);
    });
}

extern "C" fn request_screen_recording_permission(_: &Object, _: Sel, _: Id) {
    unsafe {
        let _: bool = CGRequestScreenCaptureAccess();
    }
    open_system_settings(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
    );
    mutate_runtime(|runtime| {
        let permissions = runtime.permissions;
        runtime.state.refresh_permissions(&permissions);
    });
}

extern "C" fn toggle_show_ice_icon(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_show_ice_icon(&mut runtime.store);
    });
}

extern "C" fn toggle_custom_ice_icon_is_template(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .toggle_custom_ice_icon_is_template(&mut runtime.store);
    });
}

extern "C" fn toggle_use_ice_bar(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_use_ice_bar(&mut runtime.store);
    });
}

extern "C" fn toggle_show_on_click(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_show_on_click(&mut runtime.store);
    });
}

extern "C" fn toggle_show_on_hover(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_show_on_hover(&mut runtime.store);
    });
}

extern "C" fn toggle_show_on_scroll(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        runtime.state.toggle_show_on_scroll(&mut runtime.store);
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

extern "C" fn set_ice_bar_location_dynamic(_: &Object, _: Sel, _: Id) {
    set_ice_bar_location(IceBarLocation::Dynamic);
}

extern "C" fn set_ice_bar_location_mouse_pointer(_: &Object, _: Sel, _: Id) {
    set_ice_bar_location(IceBarLocation::MousePointer);
}

extern "C" fn set_ice_bar_location_ice_icon(_: &Object, _: Sel, _: Id) {
    set_ice_bar_location(IceBarLocation::IceIcon);
}

extern "C" fn set_item_spacing_offset_minus_2(_: &Object, _: Sel, _: Id) {
    set_item_spacing_offset(-2.0);
}

extern "C" fn set_item_spacing_offset_minus_1(_: &Object, _: Sel, _: Id) {
    set_item_spacing_offset(-1.0);
}

extern "C" fn set_item_spacing_offset_0(_: &Object, _: Sel, _: Id) {
    set_item_spacing_offset(0.0);
}

extern "C" fn set_item_spacing_offset_1(_: &Object, _: Sel, _: Id) {
    set_item_spacing_offset(1.0);
}

extern "C" fn set_item_spacing_offset_2(_: &Object, _: Sel, _: Id) {
    set_item_spacing_offset(2.0);
}

extern "C" fn set_show_on_hover_delay_0(_: &Object, _: Sel, _: Id) {
    set_show_on_hover_delay(0.0);
}

extern "C" fn set_show_on_hover_delay_02(_: &Object, _: Sel, _: Id) {
    set_show_on_hover_delay(0.2);
}

extern "C" fn set_show_on_hover_delay_05(_: &Object, _: Sel, _: Id) {
    set_show_on_hover_delay(0.5);
}

extern "C" fn set_show_on_hover_delay_1(_: &Object, _: Sel, _: Id) {
    set_show_on_hover_delay(1.0);
}

extern "C" fn set_temp_show_interval_5(_: &Object, _: Sel, _: Id) {
    set_temp_show_interval(5.0);
}

extern "C" fn set_temp_show_interval_10(_: &Object, _: Sel, _: Id) {
    set_temp_show_interval(10.0);
}

extern "C" fn set_temp_show_interval_15(_: &Object, _: Sel, _: Id) {
    set_temp_show_interval(15.0);
}

extern "C" fn set_temp_show_interval_30(_: &Object, _: Sel, _: Id) {
    set_temp_show_interval(30.0);
}

extern "C" fn set_temp_show_interval_60(_: &Object, _: Sel, _: Id) {
    set_temp_show_interval(60.0);
}

extern "C" fn rehide_timer_fired(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| runtime.state.tick(Instant::now()));
}

extern "C" fn hover_timer_fired(_: &Object, _: Sel, _: Id) {
    mutate_runtime(|runtime| {
        let hovering = unsafe { pointer_is_over_status_item(runtime.status_item as Id) };
        runtime
            .state
            .handle_ice_button_hover(Instant::now(), hovering);
    });
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

fn set_ice_bar_location(location: IceBarLocation) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .set_ice_bar_location(&mut runtime.store, location);
    });
}

fn set_item_spacing_offset(offset: f64) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .set_item_spacing_offset(&mut runtime.store, offset);
    });
}

fn set_show_on_hover_delay(secs: f64) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .set_show_on_hover_delay(&mut runtime.store, secs);
    });
}

fn set_temp_show_interval(secs: f64) {
    mutate_runtime(|runtime| {
        runtime
            .state
            .set_temp_show_interval(&mut runtime.store, secs);
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

unsafe fn show_settings_menu() {
    let runtime = RUNTIME
        .get()
        .expect("runtime must exist before showing settings")
        .lock()
        .expect("runtime mutex should not be poisoned");
    let status_item = runtime.status_item as Id;
    let target = runtime.target as Id;
    let app = runtime.app as Id;
    let snapshot = MenuSnapshot::from_state(&runtime.state);
    let menu = build_menu(app, target, &snapshot);
    let _: () = msg_send![status_item, popUpStatusItemMenu: menu];
}

fn current_event_is_control_click() -> bool {
    unsafe {
        let app = shared_application();
        let event: Id = msg_send![app, currentEvent];
        if event.is_null() {
            return false;
        }
        let flags: u64 = msg_send![event, modifierFlags];
        flags & NSEVENT_MODIFIER_FLAG_CONTROL != 0
    }
}

fn request_accessibility_permission_prompt() -> bool {
    unsafe {
        let key = ns_string("AXTrustedCheckOptionPrompt");
        let value: Id = msg_send![class!(NSNumber), numberWithBool: true];
        let options: Id = msg_send![class!(NSDictionary), dictionaryWithObject: value forKey: key];
        AXIsProcessTrustedWithOptions(options)
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

fn schedule_hover_timer(target: Id) {
    unsafe {
        let _: Id = msg_send![
            class!(NSTimer),
            scheduledTimerWithTimeInterval: Duration::from_millis(100).as_secs_f64()
            target: target
            selector: sel!(hoverTimerFired:)
            userInfo: std::ptr::null_mut::<Object>()
            repeats: true
        ];
    }
}

unsafe fn pointer_is_over_status_item(status_item: Id) -> bool {
    let button: Id = msg_send![status_item, button];
    if button.is_null() {
        return false;
    }

    let window: Id = msg_send![button, window];
    if window.is_null() {
        return false;
    }

    let frame: NSRect = msg_send![window, frame];
    let point: NSPoint = msg_send![class!(NSEvent), mouseLocation];
    point.x >= frame.origin.x
        && point.x <= frame.origin.x + frame.size.width
        && point.y >= frame.origin.y
        && point.y <= frame.origin.y + frame.size.height
}

fn open_system_settings(url: &str) {
    unsafe {
        let ns_url: Id = msg_send![class!(NSURL), URLWithString: ns_string(url)];
        if !ns_url.is_null() {
            let workspace: Id = msg_send![class!(NSWorkspace), sharedWorkspace];
            let _: bool = msg_send![workspace, openURL: ns_url];
        }
    }
}

#[derive(Clone, Copy)]
struct CocoaPermissionChecker;

impl PermissionChecker for CocoaPermissionChecker {
    fn has_accessibility_permission(&self) -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    fn has_screen_recording_permission(&self) -> bool {
        unsafe { CGPreflightScreenCaptureAccess() }
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

    fn data_for_key(&self, key: &str) -> Option<Vec<u8>> {
        unsafe {
            if !self.has_key(key) {
                return None;
            }
            let data: Id = msg_send![self.defaults(), dataForKey: Self::key(key)];
            if data.is_null() {
                return None;
            }
            let len: usize = msg_send![data, length];
            let bytes: *const c_void = msg_send![data, bytes];
            if bytes.is_null() {
                return None;
            }
            let slice = std::slice::from_raw_parts(bytes as *const u8, len);
            Some(slice.to_vec())
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

    fn set_data(&mut self, key: &str, value: &[u8]) {
        unsafe {
            let data: Id = msg_send![
                class!(NSData),
                dataWithBytes: value.as_ptr() as *const c_void
                length: value.len()
            ];
            let _: () = msg_send![self.defaults(), setObject: data forKey: Self::key(key)];
        }
    }
}

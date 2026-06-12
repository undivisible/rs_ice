# Rust Ice Parity Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current mixed Swift-host behavior with a Rust-owned Ice rewrite that ports upstream Ice semantics from `jordanbaird/Ice@11edd39115f3f43a83ae114b5348df6a0e1741cf`.

**Architecture:** Rust owns the runtime app, settings, permissions, private macOS window/item APIs, event taps, movement, sections, triggers, rehide, hotkeys, search, Ice Bar, and appearance. Swift/Aurorality remains only a native settings surface invoked by the Rust app on control-click.

**Tech Stack:** Rust 2021, Cargo, AppKit/Foundation/ApplicationServices/CoreGraphics FFI, `objc`, SwiftUI/Aurorality settings host, upstream Ice Swift source as behavioral reference.

---

## Current Audit

Current launched path:
- `crepus aurora dev` launches `macos/Sources/App.swift`.
- `src/main.rs` is a separate Rust app binary and was not linked to AppKit/Foundation until this plan began.
- The Swift host still owns the visible status item in the normal dev workflow.

Current drift from upstream Ice:
- Upstream `Ice/Main/AppDelegate.swift` and `Ice/Main/AppState.swift` establish one shared app state and setup ordering. Current `rs_ice` has split state between `src/app_state.rs`, `src/main.rs`, and `macos/Sources/App.swift`.
- Upstream `Ice/Bridging/Bridging.swift` and `Ice/Bridging/Shims/Private.swift` use private CGS window APIs. Current Rust has no equivalent FFI module.
- Upstream `Ice/MenuBar/MenuBarItems/MenuBarItem.swift`, `MenuBarItemInfo.swift`, and `MenuBarItemManager.swift` model real menu-bar item identity, active-space filtering, movement retries, event matching, and restoration. Current Rust only has pure data models and planning tests.
- Upstream movement posts command-modified targeted events with `eventTargetUnixProcessID`, `eventSourceUserData`, `mouseEventWindowUnderMousePointer`, `mouseEventWindowUnderMousePointerThatCanHandleThisEvent`, and private field `0x33`. Current Swift movement attempt did not faithfully reproduce the full manager pipeline and caused pointer side effects.
- Upstream `Ice/MenuBar/ControlItem/ControlItem.swift` creates three NSStatusItems for visible, hidden, and always-hidden control items. Current Swift host has one visible status item and no Rust-owned divider/control items.
- Upstream `Ice/Events/*` and `EventTap.swift` provide local/global/run-loop monitors and event taps. Current Rust has timer polling only.
- Upstream settings are managed through `Ice/Settings/SettingsManagers/*` and `Ice/Utilities/Defaults.swift`. Current Rust settings cover keys, but SwiftUI also reads defaults directly and can diverge from Rust state.

## Rewrite Rules

- Rust is the runtime host.
- SwiftUI/Aurorality is settings UI only.
- No direct menu-bar-item movement lives in `macos/Sources/App.swift`.
- Every native behavior port references the upstream source file listed in the task.
- Each task ends with `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets --all-features`, `cargo test`, `swift build`, `crepus aurora dev`, and a commit.

## File Structure

- Modify `src/main.rs`: keep only Rust AppKit app startup and high-level runtime wiring.
- Create `src/macos/mod.rs`: macOS module exports.
- Create `src/macos/appkit.rs`: typed Objective-C AppKit helpers for `NSApplication`, `NSStatusItem`, `NSMenu`, and status button actions.
- Create `src/macos/permissions.rs`: Accessibility and Screen Recording prompts/checks.
- Create `src/macos/private.rs`: private CGS and CoreGraphics FFI wrappers matching upstream `Bridging/Shims/Private.swift` and `Bridging/Bridging.swift`.
- Create `src/macos/window_info.rs`: Rust `WindowInfo` matching upstream `Utilities/WindowInfo.swift`.
- Create `src/macos/event.rs`: CGEvent construction and event-tap posting matching upstream `MenuBarItemManager.swift`.
- Modify `src/menu_bar.rs`: replace pure snapshots with real `WindowInfo` conversion and upstream predicates.
- Create `src/item_manager.rs`: Rust port of upstream `MenuBarItemManager.swift`.
- Create `src/control_item.rs`: Rust port of upstream `ControlItem.swift`.
- Modify `src/app_state.rs`: make Rust state the only visibility/settings state.
- Modify `macos/Sources/App.swift`: remove status item ownership; keep SwiftUI settings view.
- Modify `crepus.toml`: launch Rust runtime or a Rust-owned app bundle once available.
- Modify `PARITY.md`: track completed ports by upstream source file.

## Task 1: Stop Swift Host From Owning Movement

**Files:**
- Modify: `macos/Sources/App.swift`
- Modify: `PARITY.md`

- [ ] **Step 1: Remove movement from Swift app**

Remove private CGS declarations, `moveDiscoveredMenuBarItems`, `discoveredMenuBarItemWindows`, `getMenuBarWindowIDs`, `getOnScreenWindowIDs`, `currentFrame`, `windowInfo`, `moveWindow`, and `MenuBarWindow` from `macos/Sources/App.swift`.

- [ ] **Step 2: Keep settings-only interactions**

Keep control-click/right-click popover presentation in `macos/Sources/App.swift`, but do not mutate hidden menu-bar item positions there.

- [ ] **Step 3: Verify**

Run:

```bash
swift build
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features
cargo test
crepus aurora dev
```

Expected:
- Swift host builds.
- Rust gates pass.
- Dev app launches.
- Clicking the Swift status item does not synthesize menu-bar movement events.

- [ ] **Step 4: Commit**

```bash
git add macos/Sources/App.swift PARITY.md
git commit -m "Stop Swift host movement attempts"
git push
```

## Task 2: Make Rust Runtime Launchable

**Files:**
- Modify: `src/main.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Link macOS frameworks**

Add framework links for AppKit and Foundation in `src/main.rs`:

```rust
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

#[link(name = "Foundation", kind = "framework")]
extern "C" {}
```

- [ ] **Step 2: Verify Rust app starts**

Run:

```bash
cargo run
```

Expected:
- The app reaches the AppKit run loop.
- It no longer panics with `Class with name NSApplication could not be found`.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs Cargo.toml
git commit -m "Link Rust runtime to AppKit"
git push
```

## Task 3: Port Private CGS Window API To Rust

**Files:**
- Create: `src/macos/mod.rs`
- Create: `src/macos/private.rs`
- Create: `src/macos/window_info.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add macOS module exports**

Create `src/macos/mod.rs`:

```rust
pub mod private;
pub mod window_info;
```

Modify `src/lib.rs`:

```rust
pub mod app_state;
pub mod macos;
pub mod menu_bar;
pub mod menu_model;
pub mod permissions;
pub mod settings;
```

- [ ] **Step 2: Port private function signatures**

Create `src/macos/private.rs` with Rust FFI equivalents for:
- `CGSMainConnectionID`
- `CGSGetWindowCount`
- `CGSGetOnScreenWindowCount`
- `CGSGetWindowList`
- `CGSGetOnScreenWindowList`
- `CGSGetProcessMenuBarWindowList`
- `CGSGetScreenRectForWindow`
- `CGSCopySpacesForWindows`
- `CGSGetActiveSpace`

- [ ] **Step 3: Add safe wrappers**

Expose safe Rust functions:

```rust
pub fn window_count() -> Result<usize, MacOsError>;
pub fn window_list() -> Result<Vec<u32>, MacOsError>;
pub fn on_screen_window_list() -> Result<Vec<u32>, MacOsError>;
pub fn menu_bar_window_list() -> Result<Vec<u32>, MacOsError>;
pub fn on_screen_menu_bar_window_list() -> Result<Vec<u32>, MacOsError>;
pub fn window_frame(window_id: u32) -> Result<crate::menu_bar::Rect, MacOsError>;
```

- [ ] **Step 4: Port `WindowInfo`**

Create `src/macos/window_info.rs` with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub window_id: u32,
    pub frame: crate::menu_bar::Rect,
    pub title: Option<String>,
    pub layer: i64,
    pub alpha: f64,
    pub owner_pid: i32,
    pub owner_name: Option<String>,
    pub is_on_screen: bool,
}
```

- [ ] **Step 5: Test wrappers without requiring a stable menu bar layout**

Add tests that only assert wrapper shape:

```rust
#[test]
fn private_window_list_calls_return_or_error_without_panicking() {
    let _ = crate::macos::private::window_count();
    let _ = crate::macos::private::window_list();
}
```

- [ ] **Step 6: Verify and commit**

Run all gates, then:

```bash
git add src/lib.rs src/macos
git commit -m "Port private menu bar window APIs"
git push
```

## Task 4: Port Ice Event Construction

**Files:**
- Create: `src/macos/event.rs`
- Modify: `src/macos/mod.rs`
- Modify: `src/menu_bar.rs`

- [ ] **Step 1: Add event types**

Create Rust equivalents for upstream:
- `MenuBarItemEventButtonState`
- `MenuBarItemEventType`
- `EventTapLocation`

- [ ] **Step 2: Implement `menu_bar_item_event`**

The function must set:
- `eventTargetUnixProcessID`
- `eventSourceUserData`
- `mouseEventWindowUnderMousePointer`
- `mouseEventWindowUnderMousePointerThatCanHandleThisEvent`
- private field `0x33`
- command flag only for move mouse-down
- click state only for click events

- [ ] **Step 3: Add unit tests for field derivation**

Test pure mapping functions for event type, button, command flag, and click state.

- [ ] **Step 4: Verify and commit**

Run all gates, then:

```bash
git add src/macos/event.rs src/macos/mod.rs src/menu_bar.rs
git commit -m "Port Ice menu bar event construction"
git push
```

## Task 5: Port MenuBarItemManager Movement

**Files:**
- Create: `src/item_manager.rs`
- Modify: `src/lib.rs`
- Modify: `src/menu_bar.rs`
- Modify: `src/app_state.rs`

- [ ] **Step 1: Implement item cache**

Port upstream `ItemCache` fields:
- visible section items
- hidden section items
- always-hidden section items
- managed items

- [ ] **Step 2: Implement movement primitives**

Port:
- `get_current_frame`
- `get_end_point`
- `get_fallback_point`
- `item_has_correct_position`
- `move_item_without_restoring_mouse_location`
- `move`
- retry loop with five attempts

- [ ] **Step 3: Implement event pause/resume boundaries**

Movement must temporarily stop global/local event monitors once the Rust monitor layer exists. Until then, movement must not run automatically from click handling.

- [ ] **Step 4: Verify and commit**

Run all gates, then:

```bash
git add src/item_manager.rs src/lib.rs src/menu_bar.rs src/app_state.rs
git commit -m "Port menu bar item movement manager"
git push
```

## Task 6: Replace Swift Status Item With Rust Control Items

**Files:**
- Create: `src/control_item.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Modify: `macos/Sources/App.swift`
- Modify: `crepus.toml`

- [ ] **Step 1: Port control item names**

Create Rust control items:
- Ice icon visible control item
- hidden divider control item
- always-hidden divider control item

- [ ] **Step 2: Make Rust own NSStatusItems**

Rust creates and updates all status items. Swift does not call `NSStatusBar.system.statusItem`.

- [ ] **Step 3: Make Swift settings UI host-only**

`macos/Sources/App.swift` must expose only SwiftUI settings UI and no menu-bar ownership.

- [ ] **Step 4: Update dev launch**

Update `crepus.toml` so local development launches the Rust runtime path. If Crepus cannot launch a Rust app bundle directly, create the smallest bundle wrapper that runs the Rust binary and embeds the Swift settings surface.

- [ ] **Step 5: Verify and commit**

Run all gates and manually verify:
- normal click toggles hidden section from Rust
- control-click opens settings
- settings changes persist to upstream keys

Then:

```bash
git add src/control_item.rs src/main.rs src/lib.rs macos/Sources/App.swift crepus.toml
git commit -m "Move status item ownership to Rust"
git push
```

## Task 7: Port Events, Triggers, And Rehide

**Files:**
- Create: `src/events.rs`
- Modify: `src/app_state.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Port local/global/run-loop event monitors**

Reference:
- `Ice/Events/EventManager.swift`
- `Ice/Events/EventTap.swift`
- `Ice/Events/EventMonitors/*.swift`

- [ ] **Step 2: Implement triggers**

Implement:
- empty menu-bar click
- Ice icon click
- hover with configured delay
- scroll/swipe
- focused-app rehide
- avoid rehide while menus/settings are open

- [ ] **Step 3: Verify and commit**

Run all gates and manual trigger parity checks, then commit and push.

## Task 8: Continue Full Parity

Implement these in order after movement works:
- Layout editor from `Ice/UI/LayoutBar/*`
- Ice Bar from `Ice/UI/IceBar/*`
- Search from `Ice/MenuBar/Search/MenuBarSearchPanel.swift`
- Appearance from `Ice/MenuBar/Appearance/*`
- Hotkeys from `Ice/Hotkeys/*`
- Launch at login, notifications, updates, migrations, and about from upstream utility modules

Each feature must be Rust-owned for behavior and may use SwiftUI only for rich UI surfaces.

## Verification Checklist

Run after every task:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features
cargo test
swift build
crepus aurora dev
```

Manual checks:
- `cargo run` creates a Rust status item.
- `crepus aurora dev` launches the intended runtime target.
- normal click does not open settings.
- control-click opens settings.
- menu bar item movement does not move the pointer.
- hidden, visible, and always-hidden item positions match upstream Ice after toggles.
- permissions are requested before movement.
- denying permissions leaves clear UI state and no attempted movement.


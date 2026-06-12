# Ice Parity Scope

Upstream reference: `jordanbaird/Ice` at `11edd39115f3f43a83ae114b5348df6a0e1741cf`.

The Rust rewrite should keep full feature and settings parity with upstream Ice. The intended product change is that settings are hosted from the menu bar instead of a standalone settings window.

## Current Rust State

- AppKit accessory app starts from Rust.
- One menu bar status item is created.
- The status item uses a snowflake symbol when available and falls back to text.
- The status item menu can toggle the hidden section and simple upstream-compatible settings.
- The Swift/Aurorality menu-bar popover exposes native groups for General, Ice Bar, and Advanced simple settings.
- Rust settings persistence covers upstream defaults keys for general, advanced, Ice Bar, hotkeys, menu bar appearance raw data, and migration flags.
- Rust preserves upstream-compatible raw data for Codable settings that do not yet have full behavioral editors.
- Rust models accessibility and screen recording permission state and exposes native permission actions from the status menu.
- The Swift/Aurorality menu-bar popover includes a native Permissions tab with current status and system settings actions.
- Rust menu-bar item snapshots now model upstream identity, owner, display, active-space, on-screen, movability, hideability, section partitioning, and debug inventory output.
- Rust models visible, hidden, and always-hidden section caches plus item movement plans with ordering, movability, and hideability validation.
- Rust app state tracks hidden and always-hidden section visibility, temporary show deadlines, and exposes menu actions for both section toggles.
- The status item now uses normal click for the Ice show/hide action, control-click for the settings menu, startup Accessibility prompting, and hover-delay state from the real status item frame.
- The Rust binary now links AppKit and Foundation so it can start the AppKit runtime directly.
- Swift-hosted direct menu-bar movement attempts were removed; movement must be implemented in Rust by porting upstream Ice's private CGS window discovery and targeted event pipeline.
- The project-specific Swift status-item host was removed; settings are now loaded through an Aurorality-only SwiftUI entrypoint while Rust owns menu bar behavior.

## Parity Work Left

- App lifecycle and shared state:
  - Rust AppKit app delegate lifecycle.
  - permission-gated setup.
  - accessory and regular activation policy transitions.
  - app-wide state object equivalent.
  - setup ordering for menu bar, settings, events, permissions, hotkeys, updates, notifications, image cache, and migrations.

- Permissions:
  - background cursor permission behavior.
  - permission-gated setup ordering for features that require accessibility or screen recording.

- Menu bar item model:
  - Rust native enumeration of menu bar items across displays and spaces using Ice's private CGS window-list approach.
  - capture app icons.
  - cache menu bar item images.
  - handle fullscreen spaces and system-hidden menu bar state.

- Menu bar section behavior:
  - visible section.
  - hidden section.
  - always-hidden section.
  - section divider control items.
  - native section divider show/hide execution for other menu bar items.
  - Option-key always-hidden toggling.
  - drag-aware section visibility.

- Item movement and layout:
  - drag-and-drop layout interface.
  - Rust native execution for moving individual items between sections.
  - Ice-style targeted command-modified event construction.
  - event tap matching for movement success/failure.
  - handle movement retries and failure states.
  - support menu bar item groups.
  - support individual spacer items.

- Show triggers:
  - show on empty menu bar click.
  - show on menu bar hover.
  - configurable hover delay.
  - show and hide on menu bar scroll.
  - trigger-condition based item visibility.

- Rehide:
  - smart strategy.
  - timed strategy.
  - focused-app strategy.
  - configurable rehide interval.
  - temporary show interval.
  - avoid rehiding while relevant menus are open.

- Ice Bar:
  - separate bar below the menu bar.
  - dynamic location.
  - centered below mouse pointer.
  - centered below Ice icon.
  - item rendering with cached icons.
  - hover and click handling.
  - screen recording permission fallback.

- Search:
  - menu bar item search panel.
  - correct item names.
  - correct app icons.
  - keyboard navigation and activation.

- Menu bar appearance:
  - overlay panel.
  - light, dark, and static configurations.
  - tint color and gradient.
  - border.
  - shadow.
  - full and split shape modes.
  - notch-aware rendering.
  - average menu bar or wallpaper color sampling.

- Menu bar spacing:
  - item spacing offset.
  - third-party item spacing handling.
  - persistence and logout-sensitive state.

- Settings parity:
  - Rust-to-Aurorality settings bridge actions.
  - rich Ice icon selection and custom icon picker.
  - Ice Bar pinned location editor.
  - Menu Bar Appearance Configuration V2 editor and renderer.
  - Hotkey recorder UI and registration.

- Hotkeys:
  - toggle hidden section.
  - toggle always-hidden section.
  - search menu bar items.
  - enable Ice Bar.
  - show section dividers.
  - toggle application menus.
  - key recording UI.
  - reserved shortcut validation.
  - persistence.

- App integrations:
  - launch at login.
  - automatic updates.
  - acknowledgements/about surface.
  - user notifications.
  - migrations for old defaults.
  - logging.

## Settings UI Change

The Rust app should not recreate the standalone settings window as the primary settings surface. Settings should be reachable from the menu bar status item and organized there with equivalent controls, choices, persistence, validation, and side effects.

Where an upstream setting currently opens a rich editor, the menu bar version still needs full behavior:

- layout editing may use a menu-bar popover or panel.
- hotkey recording may use a menu-bar popover or panel.
- appearance editing may use a menu-bar popover or panel.
- permissions may use a menu-bar popover or panel when system prompts are insufficient.

## Implementation Order

1. Replace the current single-file shell with a small Rust app architecture: app state, status item, settings store, and menu construction.
2. Port settings storage with the same keys and defaults where possible.
3. Implement the complete settings menu skeleton with disabled entries for behavior not wired yet.
4. Port permission checks.
5. Port menu bar item enumeration and image caching.
6. Port sections and section divider items.
7. Port show, hide, trigger, and rehide behavior.
8. Port layout editing.
9. Port Ice Bar.
10. Port search.
11. Port appearance rendering.
12. Port hotkeys.
13. Port launch at login, updates, notifications, migrations, and about.

use crate::permissions::{PermissionChecker, PermissionSnapshot};
use crate::settings::{IceBarLocation, RehideStrategy, Settings, SettingsStore};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionName {
    Visible,
    Hidden,
    AlwaysHidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionVisibility {
    Shown,
    Hidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionState {
    pub name: SectionName,
    pub visibility: SectionVisibility,
}

#[derive(Debug)]
pub struct AppState {
    settings: Settings,
    visible_section: SectionState,
    hidden_section: SectionState,
    always_hidden_section: SectionState,
    rehide_deadline: Option<Instant>,
    temporary_show_deadline: Option<Instant>,
    permissions: PermissionSnapshot,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            visible_section: SectionState {
                name: SectionName::Visible,
                visibility: SectionVisibility::Shown,
            },
            hidden_section: SectionState {
                name: SectionName::Hidden,
                visibility: SectionVisibility::Hidden,
            },
            always_hidden_section: SectionState {
                name: SectionName::AlwaysHidden,
                visibility: SectionVisibility::Hidden,
            },
            rehide_deadline: None,
            temporary_show_deadline: None,
            permissions: PermissionSnapshot::default(),
        }
    }

    pub fn load(store: &impl SettingsStore) -> Self {
        Self::new(store.load_settings())
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn hidden_section(&self) -> &SectionState {
        &self.hidden_section
    }

    pub fn always_hidden_section(&self) -> &SectionState {
        &self.always_hidden_section
    }

    pub fn visible_section(&self) -> &SectionState {
        &self.visible_section
    }

    pub fn hidden_section_is_shown(&self) -> bool {
        self.hidden_section.visibility == SectionVisibility::Shown
    }

    pub fn always_hidden_section_is_shown(&self) -> bool {
        self.always_hidden_section.visibility == SectionVisibility::Shown
    }

    pub fn rehide_deadline(&self) -> Option<Instant> {
        self.rehide_deadline
    }

    pub fn temporary_show_deadline(&self) -> Option<Instant> {
        self.temporary_show_deadline
    }

    pub fn permissions(&self) -> PermissionSnapshot {
        self.permissions
    }

    pub fn refresh_permissions(&mut self, checker: &impl PermissionChecker) {
        self.permissions = PermissionSnapshot::from_checker(checker);
    }

    pub fn toggle_hidden_section(&mut self) {
        if self.hidden_section_is_shown() {
            self.hide_hidden_section();
        } else {
            self.show_hidden_section(Instant::now());
        }
    }

    pub fn toggle_always_hidden_section(&mut self, now: Instant) {
        if !self.settings.enable_always_hidden_section
            || !self.settings.can_toggle_always_hidden_section
        {
            return;
        }

        if self.always_hidden_section_is_shown() {
            self.hide_always_hidden_section();
        } else {
            self.show_always_hidden_section(now);
        }
    }

    pub fn show_hidden_section(&mut self, now: Instant) {
        self.hidden_section.visibility = SectionVisibility::Shown;
        self.rehide_deadline = self.next_rehide_deadline(now);
    }

    pub fn show_hidden_section_temporarily(&mut self, now: Instant) {
        self.show_hidden_section(now);
        self.temporary_show_deadline = Some(self.next_temporary_show_deadline(now));
    }

    pub fn hide_hidden_section(&mut self) {
        self.hidden_section.visibility = SectionVisibility::Hidden;
        self.hide_always_hidden_section();
        self.rehide_deadline = None;
        self.temporary_show_deadline = None;
    }

    pub fn show_always_hidden_section(&mut self, now: Instant) {
        self.show_hidden_section(now);
        self.always_hidden_section.visibility = SectionVisibility::Shown;
        self.temporary_show_deadline = Some(self.next_temporary_show_deadline(now));
    }

    pub fn hide_always_hidden_section(&mut self) {
        self.always_hidden_section.visibility = SectionVisibility::Hidden;
        self.temporary_show_deadline = None;
    }

    pub fn handle_empty_menu_bar_click(&mut self, now: Instant) {
        if self.settings.show_on_click {
            if self.hidden_section_is_shown() {
                self.hide_hidden_section();
            } else {
                self.show_hidden_section(now);
            }
        }
    }

    pub fn tick(&mut self, now: Instant) {
        if self
            .temporary_show_deadline
            .is_some_and(|deadline| now >= deadline)
        {
            self.hide_always_hidden_section();
        }

        if self.rehide_deadline.is_some_and(|deadline| now >= deadline) {
            self.hide_hidden_section();
        }
    }

    pub fn toggle_show_ice_icon(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_ice_icon = !self.settings.show_ice_icon;
        store.save_settings(&self.settings);
    }

    pub fn toggle_show_on_click(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_on_click = !self.settings.show_on_click;
        store.save_settings(&self.settings);
    }

    pub fn toggle_show_on_hover(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_on_hover = !self.settings.show_on_hover;
        store.save_settings(&self.settings);
    }

    pub fn toggle_show_on_scroll(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_on_scroll = !self.settings.show_on_scroll;
        store.save_settings(&self.settings);
    }

    pub fn toggle_use_ice_bar(&mut self, store: &mut impl SettingsStore) {
        self.settings.use_ice_bar = !self.settings.use_ice_bar;
        store.save_settings(&self.settings);
    }

    pub fn toggle_custom_ice_icon_is_template(&mut self, store: &mut impl SettingsStore) {
        self.settings.custom_ice_icon_is_template = !self.settings.custom_ice_icon_is_template;
        store.save_settings(&self.settings);
    }

    pub fn toggle_auto_rehide(&mut self, store: &mut impl SettingsStore) {
        self.settings.auto_rehide = !self.settings.auto_rehide;
        if !self.settings.auto_rehide {
            self.rehide_deadline = None;
        } else if self.hidden_section_is_shown() {
            self.rehide_deadline = self.next_rehide_deadline(Instant::now());
        }
        store.save_settings(&self.settings);
    }

    pub fn toggle_context_menu_on_right_click(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_context_menu_on_right_click =
            !self.settings.show_context_menu_on_right_click;
        store.save_settings(&self.settings);
    }

    pub fn toggle_hide_application_menus(&mut self, store: &mut impl SettingsStore) {
        self.settings.hide_application_menus = !self.settings.hide_application_menus;
        store.save_settings(&self.settings);
    }

    pub fn toggle_show_section_dividers(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_section_dividers = !self.settings.show_section_dividers;
        store.save_settings(&self.settings);
    }

    pub fn toggle_enable_always_hidden_section(&mut self, store: &mut impl SettingsStore) {
        self.settings.enable_always_hidden_section = !self.settings.enable_always_hidden_section;
        if !self.settings.enable_always_hidden_section {
            self.hide_always_hidden_section();
        }
        store.save_settings(&self.settings);
    }

    pub fn toggle_can_toggle_always_hidden_section(&mut self, store: &mut impl SettingsStore) {
        self.settings.can_toggle_always_hidden_section =
            !self.settings.can_toggle_always_hidden_section;
        if !self.settings.can_toggle_always_hidden_section {
            self.hide_always_hidden_section();
        }
        store.save_settings(&self.settings);
    }

    pub fn toggle_show_all_sections_on_user_drag(&mut self, store: &mut impl SettingsStore) {
        self.settings.show_all_sections_on_user_drag =
            !self.settings.show_all_sections_on_user_drag;
        store.save_settings(&self.settings);
    }

    pub fn set_rehide_strategy(
        &mut self,
        store: &mut impl SettingsStore,
        strategy: RehideStrategy,
    ) {
        self.settings.rehide_strategy = strategy;
        if self.hidden_section_is_shown() {
            self.rehide_deadline = self.next_rehide_deadline(Instant::now());
        }
        store.save_settings(&self.settings);
    }

    pub fn set_rehide_interval(&mut self, store: &mut impl SettingsStore, secs: f64) {
        if secs.is_finite() && secs > 0.0 {
            self.settings.rehide_interval_secs = secs;
            if self.hidden_section_is_shown() {
                self.rehide_deadline = self.next_rehide_deadline(Instant::now());
            }
            store.save_settings(&self.settings);
        }
    }

    pub fn set_ice_bar_location(
        &mut self,
        store: &mut impl SettingsStore,
        location: IceBarLocation,
    ) {
        self.settings.ice_bar_location = location;
        store.save_settings(&self.settings);
    }

    pub fn set_item_spacing_offset(&mut self, store: &mut impl SettingsStore, offset: f64) {
        if offset.is_finite() {
            self.settings.item_spacing_offset = offset;
            store.save_settings(&self.settings);
        }
    }

    pub fn set_show_on_hover_delay(&mut self, store: &mut impl SettingsStore, secs: f64) {
        if secs.is_finite() && secs >= 0.0 {
            self.settings.show_on_hover_delay_secs = secs;
            store.save_settings(&self.settings);
        }
    }

    pub fn set_temp_show_interval(&mut self, store: &mut impl SettingsStore, secs: f64) {
        if secs.is_finite() && secs > 0.0 {
            self.settings.temp_show_interval_secs = secs;
            store.save_settings(&self.settings);
        }
    }

    fn next_rehide_deadline(&self, now: Instant) -> Option<Instant> {
        if !self.settings.auto_rehide || self.settings.rehide_strategy != RehideStrategy::Timed {
            return None;
        }

        Some(now + Duration::from_secs_f64(self.settings.rehide_interval_secs))
    }

    fn next_temporary_show_deadline(&self, now: Instant) -> Instant {
        now + Duration::from_secs_f64(self.settings.temp_show_interval_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::tests_support::MemorySettingsStore;

    #[test]
    fn hidden_section_starts_hidden() {
        let state = AppState::new(Settings::default());

        assert!(!state.hidden_section_is_shown());
        assert!(!state.always_hidden_section_is_shown());
    }

    #[test]
    fn click_trigger_respects_setting() {
        let mut state = AppState::new(Settings {
            show_on_click: false,
            ..Settings::default()
        });

        state.handle_empty_menu_bar_click(Instant::now());

        assert!(!state.hidden_section_is_shown());
    }

    #[test]
    fn timed_rehide_hides_after_interval() {
        let now = Instant::now();
        let mut state = AppState::new(Settings {
            rehide_strategy: RehideStrategy::Timed,
            rehide_interval_secs: 5.0,
            ..Settings::default()
        });

        state.show_hidden_section(now);
        state.tick(now + Duration::from_secs(4));
        assert!(state.hidden_section_is_shown());

        state.tick(now + Duration::from_secs(5));
        assert!(!state.hidden_section_is_shown());
    }

    #[test]
    fn always_hidden_toggle_respects_settings_and_temp_interval() {
        let now = Instant::now();
        let mut state = AppState::new(Settings {
            enable_always_hidden_section: true,
            can_toggle_always_hidden_section: true,
            temp_show_interval_secs: 5.0,
            ..Settings::default()
        });

        state.toggle_always_hidden_section(now);

        assert!(state.hidden_section_is_shown());
        assert!(state.always_hidden_section_is_shown());
        assert_eq!(
            state.temporary_show_deadline(),
            Some(now + Duration::from_secs(5))
        );

        state.tick(now + Duration::from_secs(5));

        assert!(state.hidden_section_is_shown());
        assert!(!state.always_hidden_section_is_shown());
    }

    #[test]
    fn always_hidden_toggle_is_ignored_when_disabled() {
        let mut state = AppState::new(Settings {
            enable_always_hidden_section: false,
            can_toggle_always_hidden_section: true,
            ..Settings::default()
        });

        state.toggle_always_hidden_section(Instant::now());

        assert!(!state.always_hidden_section_is_shown());
    }

    #[test]
    fn hiding_hidden_section_hides_always_hidden_section() {
        let now = Instant::now();
        let mut state = AppState::new(Settings {
            enable_always_hidden_section: true,
            can_toggle_always_hidden_section: true,
            ..Settings::default()
        });

        state.show_always_hidden_section(now);
        state.hide_hidden_section();

        assert!(!state.hidden_section_is_shown());
        assert!(!state.always_hidden_section_is_shown());
        assert_eq!(state.temporary_show_deadline(), None);
    }

    #[test]
    fn toggles_persist_settings() {
        let mut store = MemorySettingsStore::default();
        let mut state = AppState::load(&store);

        state.toggle_show_on_click(&mut store);
        let reloaded = AppState::load(&store);

        assert!(!reloaded.settings().show_on_click);
    }
}

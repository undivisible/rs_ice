use crate::settings::{RehideStrategy, Settings, SettingsStore};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionName {
    Visible,
    Hidden,
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
    rehide_deadline: Option<Instant>,
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
            rehide_deadline: None,
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

    pub fn visible_section(&self) -> &SectionState {
        &self.visible_section
    }

    pub fn hidden_section_is_shown(&self) -> bool {
        self.hidden_section.visibility == SectionVisibility::Shown
    }

    pub fn rehide_deadline(&self) -> Option<Instant> {
        self.rehide_deadline
    }

    pub fn toggle_hidden_section(&mut self) {
        if self.hidden_section_is_shown() {
            self.hide_hidden_section();
        } else {
            self.show_hidden_section(Instant::now());
        }
    }

    pub fn show_hidden_section(&mut self, now: Instant) {
        self.hidden_section.visibility = SectionVisibility::Shown;
        self.rehide_deadline = self.next_rehide_deadline(now);
    }

    pub fn hide_hidden_section(&mut self) {
        self.hidden_section.visibility = SectionVisibility::Hidden;
        self.rehide_deadline = None;
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

    fn next_rehide_deadline(&self, now: Instant) -> Option<Instant> {
        if !self.settings.auto_rehide || self.settings.rehide_strategy != RehideStrategy::Timed {
            return None;
        }

        Some(now + Duration::from_secs_f64(self.settings.rehide_interval_secs))
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
    fn toggles_persist_settings() {
        let mut store = MemorySettingsStore::default();
        let mut state = AppState::load(&store);

        state.toggle_show_on_click(&mut store);
        let reloaded = AppState::load(&store);

        assert!(!reloaded.settings().show_on_click);
    }
}

use crate::app_state::AppState;
use crate::settings::RehideStrategy;

#[derive(Debug, Clone, PartialEq)]
pub struct MenuSnapshot {
    pub ice_icon_visible: bool,
    pub hidden_section_visible: bool,
    pub show_on_click: bool,
    pub auto_rehide: bool,
    pub rehide_strategy: RehideStrategy,
    pub rehide_interval_secs: f64,
    pub show_context_menu_on_right_click: bool,
}

impl MenuSnapshot {
    pub fn from_state(state: &AppState) -> Self {
        let settings = state.settings();

        Self {
            ice_icon_visible: settings.show_ice_icon,
            hidden_section_visible: state.hidden_section_is_shown(),
            show_on_click: settings.show_on_click,
            auto_rehide: settings.auto_rehide,
            rehide_strategy: settings.rehide_strategy,
            rehide_interval_secs: settings.rehide_interval_secs,
            show_context_menu_on_right_click: settings.show_context_menu_on_right_click,
        }
    }

    pub fn hidden_toggle_title(&self) -> &'static str {
        if self.hidden_section_visible {
            "Hide Hidden Section"
        } else {
            "Show Hidden Section"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use crate::settings::Settings;

    #[test]
    fn hidden_toggle_title_tracks_state() {
        let mut state = AppState::new(Settings::default());
        let hidden = MenuSnapshot::from_state(&state);
        assert_eq!(hidden.hidden_toggle_title(), "Show Hidden Section");

        state.toggle_hidden_section();
        let shown = MenuSnapshot::from_state(&state);
        assert_eq!(shown.hidden_toggle_title(), "Hide Hidden Section");
    }
}

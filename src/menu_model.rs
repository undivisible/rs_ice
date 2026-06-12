use crate::app_state::AppState;
use crate::settings::{IceBarLocation, RehideStrategy};

#[derive(Debug, Clone, PartialEq)]
pub struct MenuSnapshot {
    pub ice_icon_visible: bool,
    pub custom_ice_icon_is_template: bool,
    pub use_ice_bar: bool,
    pub ice_bar_location: IceBarLocation,
    pub hidden_section_visible: bool,
    pub show_on_click: bool,
    pub show_on_hover: bool,
    pub show_on_scroll: bool,
    pub item_spacing_offset: f64,
    pub auto_rehide: bool,
    pub rehide_strategy: RehideStrategy,
    pub rehide_interval_secs: f64,
    pub hide_application_menus: bool,
    pub show_section_dividers: bool,
    pub enable_always_hidden_section: bool,
    pub can_toggle_always_hidden_section: bool,
    pub show_on_hover_delay_secs: f64,
    pub temp_show_interval_secs: f64,
    pub show_all_sections_on_user_drag: bool,
    pub show_context_menu_on_right_click: bool,
}

impl MenuSnapshot {
    pub fn from_state(state: &AppState) -> Self {
        let settings = state.settings();

        Self {
            ice_icon_visible: settings.show_ice_icon,
            custom_ice_icon_is_template: settings.custom_ice_icon_is_template,
            use_ice_bar: settings.use_ice_bar,
            ice_bar_location: settings.ice_bar_location,
            hidden_section_visible: state.hidden_section_is_shown(),
            show_on_click: settings.show_on_click,
            show_on_hover: settings.show_on_hover,
            show_on_scroll: settings.show_on_scroll,
            item_spacing_offset: settings.item_spacing_offset,
            auto_rehide: settings.auto_rehide,
            rehide_strategy: settings.rehide_strategy,
            rehide_interval_secs: settings.rehide_interval_secs,
            hide_application_menus: settings.hide_application_menus,
            show_section_dividers: settings.show_section_dividers,
            enable_always_hidden_section: settings.enable_always_hidden_section,
            can_toggle_always_hidden_section: settings.can_toggle_always_hidden_section,
            show_on_hover_delay_secs: settings.show_on_hover_delay_secs,
            temp_show_interval_secs: settings.temp_show_interval_secs,
            show_all_sections_on_user_drag: settings.show_all_sections_on_user_drag,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RehideStrategy {
    Smart,
    Timed,
    FocusedApp,
}

impl RehideStrategy {
    pub const ALL: [Self; 3] = [Self::Smart, Self::Timed, Self::FocusedApp];

    pub fn raw_value(self) -> i64 {
        match self {
            Self::Smart => 0,
            Self::Timed => 1,
            Self::FocusedApp => 2,
        }
    }

    pub fn from_raw_value(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Smart),
            1 => Some(Self::Timed),
            2 => Some(Self::FocusedApp),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Smart => "Smart",
            Self::Timed => "Timed",
            Self::FocusedApp => "Focused App",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub show_ice_icon: bool,
    pub show_on_click: bool,
    pub auto_rehide: bool,
    pub rehide_strategy: RehideStrategy,
    pub rehide_interval_secs: f64,
    pub show_context_menu_on_right_click: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_ice_icon: true,
            show_on_click: true,
            auto_rehide: true,
            rehide_strategy: RehideStrategy::Smart,
            rehide_interval_secs: 15.0,
            show_context_menu_on_right_click: true,
        }
    }
}

pub mod keys {
    pub const SHOW_ICE_ICON: &str = "ShowIceIcon";
    pub const SHOW_ON_CLICK: &str = "ShowOnClick";
    pub const AUTO_REHIDE: &str = "AutoRehide";
    pub const REHIDE_STRATEGY: &str = "RehideStrategy";
    pub const REHIDE_INTERVAL: &str = "RehideInterval";
    pub const SHOW_CONTEXT_MENU_ON_RIGHT_CLICK: &str = "ShowContextMenuOnRightClick";
}

pub trait SettingsStore {
    fn bool_for_key(&self, key: &str) -> Option<bool>;
    fn integer_for_key(&self, key: &str) -> Option<i64>;
    fn double_for_key(&self, key: &str) -> Option<f64>;
    fn set_bool(&mut self, key: &str, value: bool);
    fn set_integer(&mut self, key: &str, value: i64);
    fn set_double(&mut self, key: &str, value: f64);

    fn load_settings(&self) -> Settings {
        let defaults = Settings::default();

        Settings {
            show_ice_icon: self
                .bool_for_key(keys::SHOW_ICE_ICON)
                .unwrap_or(defaults.show_ice_icon),
            show_on_click: self
                .bool_for_key(keys::SHOW_ON_CLICK)
                .unwrap_or(defaults.show_on_click),
            auto_rehide: self
                .bool_for_key(keys::AUTO_REHIDE)
                .unwrap_or(defaults.auto_rehide),
            rehide_strategy: self
                .integer_for_key(keys::REHIDE_STRATEGY)
                .and_then(RehideStrategy::from_raw_value)
                .unwrap_or(defaults.rehide_strategy),
            rehide_interval_secs: self
                .double_for_key(keys::REHIDE_INTERVAL)
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(defaults.rehide_interval_secs),
            show_context_menu_on_right_click: self
                .bool_for_key(keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK)
                .unwrap_or(defaults.show_context_menu_on_right_click),
        }
    }

    fn save_settings(&mut self, settings: &Settings) {
        self.set_bool(keys::SHOW_ICE_ICON, settings.show_ice_icon);
        self.set_bool(keys::SHOW_ON_CLICK, settings.show_on_click);
        self.set_bool(keys::AUTO_REHIDE, settings.auto_rehide);
        self.set_integer(keys::REHIDE_STRATEGY, settings.rehide_strategy.raw_value());
        self.set_double(keys::REHIDE_INTERVAL, settings.rehide_interval_secs);
        self.set_bool(
            keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK,
            settings.show_context_menu_on_right_click,
        );
    }
}

#[cfg(test)]
pub mod tests_support {
    use super::SettingsStore;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Bool(bool),
        Integer(i64),
        Double(f64),
    }

    #[derive(Debug, Default)]
    pub struct MemorySettingsStore {
        values: HashMap<String, Value>,
    }

    impl MemorySettingsStore {
        pub fn set(&mut self, key: &str, value: Value) {
            self.values.insert(key.to_string(), value);
        }
    }

    impl SettingsStore for MemorySettingsStore {
        fn bool_for_key(&self, key: &str) -> Option<bool> {
            match self.values.get(key) {
                Some(Value::Bool(value)) => Some(*value),
                _ => None,
            }
        }

        fn integer_for_key(&self, key: &str) -> Option<i64> {
            match self.values.get(key) {
                Some(Value::Integer(value)) => Some(*value),
                _ => None,
            }
        }

        fn double_for_key(&self, key: &str) -> Option<f64> {
            match self.values.get(key) {
                Some(Value::Double(value)) => Some(*value),
                _ => None,
            }
        }

        fn set_bool(&mut self, key: &str, value: bool) {
            self.set(key, Value::Bool(value));
        }

        fn set_integer(&mut self, key: &str, value: i64) {
            self.set(key, Value::Integer(value));
        }

        fn set_double(&mut self, key: &str, value: f64) {
            self.set(key, Value::Double(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tests_support::{MemorySettingsStore, Value};
    use super::*;

    #[test]
    fn defaults_match_upstream_vertical_slice() {
        let settings = Settings::default();

        assert!(settings.show_ice_icon);
        assert!(settings.show_on_click);
        assert!(settings.auto_rehide);
        assert_eq!(settings.rehide_strategy, RehideStrategy::Smart);
        assert_eq!(settings.rehide_interval_secs, 15.0);
        assert!(settings.show_context_menu_on_right_click);
    }

    #[test]
    fn loads_persisted_values_by_upstream_keys() {
        let mut store = MemorySettingsStore::default();
        store.set(keys::SHOW_ICE_ICON, Value::Bool(false));
        store.set(keys::SHOW_ON_CLICK, Value::Bool(false));
        store.set(keys::AUTO_REHIDE, Value::Bool(false));
        store.set(keys::REHIDE_STRATEGY, Value::Integer(1));
        store.set(keys::REHIDE_INTERVAL, Value::Double(30.0));
        store.set(keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK, Value::Bool(false));

        let settings = store.load_settings();

        assert!(!settings.show_ice_icon);
        assert!(!settings.show_on_click);
        assert!(!settings.auto_rehide);
        assert_eq!(settings.rehide_strategy, RehideStrategy::Timed);
        assert_eq!(settings.rehide_interval_secs, 30.0);
        assert!(!settings.show_context_menu_on_right_click);
    }

    #[test]
    fn invalid_rehide_values_fall_back_to_defaults() {
        let mut store = MemorySettingsStore::default();
        store.set(keys::REHIDE_STRATEGY, Value::Integer(99));
        store.set(keys::REHIDE_INTERVAL, Value::Double(-1.0));

        let settings = store.load_settings();

        assert_eq!(settings.rehide_strategy, RehideStrategy::Smart);
        assert_eq!(settings.rehide_interval_secs, 15.0);
    }

    #[test]
    fn saves_values_to_upstream_keys() {
        let mut store = MemorySettingsStore::default();
        let settings = Settings {
            show_ice_icon: false,
            show_on_click: false,
            auto_rehide: false,
            rehide_strategy: RehideStrategy::FocusedApp,
            rehide_interval_secs: 60.0,
            show_context_menu_on_right_click: false,
        };

        store.save_settings(&settings);

        assert_eq!(store.bool_for_key(keys::SHOW_ICE_ICON), Some(false));
        assert_eq!(store.bool_for_key(keys::SHOW_ON_CLICK), Some(false));
        assert_eq!(store.bool_for_key(keys::AUTO_REHIDE), Some(false));
        assert_eq!(
            store.integer_for_key(keys::REHIDE_STRATEGY),
            Some(RehideStrategy::FocusedApp.raw_value())
        );
        assert_eq!(store.double_for_key(keys::REHIDE_INTERVAL), Some(60.0));
        assert_eq!(
            store.bool_for_key(keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK),
            Some(false)
        );
    }
}

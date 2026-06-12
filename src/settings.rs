use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceBarLocation {
    Dynamic,
    MousePointer,
    IceIcon,
}

impl IceBarLocation {
    pub const ALL: [Self; 3] = [Self::Dynamic, Self::MousePointer, Self::IceIcon];

    pub fn raw_value(self) -> i64 {
        match self {
            Self::Dynamic => 0,
            Self::MousePointer => 1,
            Self::IceIcon => 2,
        }
    }

    pub fn from_raw_value(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Dynamic),
            1 => Some(Self::MousePointer),
            2 => Some(Self::IceIcon),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dynamic => "Dynamic",
            Self::MousePointer => "Mouse pointer",
            Self::IceIcon => "Ice icon",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyAction {
    #[serde(rename = "ToggleHiddenSection")]
    ToggleHiddenSection,
    #[serde(rename = "ToggleAlwaysHiddenSection")]
    ToggleAlwaysHiddenSection,
    #[serde(rename = "SearchMenuBarItems")]
    SearchMenuBarItems,
    #[serde(rename = "EnableIceBar")]
    EnableIceBar,
    #[serde(rename = "ShowSectionDividers")]
    ShowSectionDividers,
    #[serde(rename = "ToggleApplicationMenus")]
    ToggleApplicationMenus,
}

impl HotkeyAction {
    pub const ALL: [Self; 6] = [
        Self::ToggleHiddenSection,
        Self::ToggleAlwaysHiddenSection,
        Self::SearchMenuBarItems,
        Self::EnableIceBar,
        Self::ShowSectionDividers,
        Self::ToggleApplicationMenus,
    ];

    pub fn raw_value(self) -> &'static str {
        match self {
            Self::ToggleHiddenSection => "ToggleHiddenSection",
            Self::ToggleAlwaysHiddenSection => "ToggleAlwaysHiddenSection",
            Self::SearchMenuBarItems => "SearchMenuBarItems",
            Self::EnableIceBar => "EnableIceBar",
            Self::ShowSectionDividers => "ShowSectionDividers",
            Self::ToggleApplicationMenus => "ToggleApplicationMenus",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCombination {
    pub key_code: i64,
    pub modifiers: i64,
}

impl Serialize for KeyCombination {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.key_code)?;
        tuple.serialize_element(&self.modifiers)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for KeyCombination {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyCombinationVisitor;

        impl<'de> Visitor<'de> for KeyCombinationVisitor {
            type Value = KeyCombination;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a two-item key combination array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let key_code = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("missing key code"))?;
                let modifiers = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("missing modifiers"))?;
                Ok(KeyCombination {
                    key_code,
                    modifiers,
                })
            }
        }

        deserializer.deserialize_tuple(2, KeyCombinationVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub action: HotkeyAction,
    #[serde(rename = "keyCombination")]
    pub key_combination: Option<KeyCombination>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuBarAppearanceConfigurationV2 {
    pub raw_json: Vec<u8>,
}

impl Default for MenuBarAppearanceConfigurationV2 {
    fn default() -> Self {
        Self {
            raw_json: br#"{"lightModeConfiguration":{"hasShadow":false,"hasBorder":false,"borderWidth":1,"tintKind":0},"darkModeConfiguration":{"hasShadow":false,"hasBorder":false,"borderWidth":1,"tintKind":0},"staticConfiguration":{"hasShadow":false,"hasBorder":false,"borderWidth":1,"tintKind":0},"shapeKind":0,"fullShapeInfo":{"leadingEndCap":1,"trailingEndCap":1},"splitShapeInfo":{"leading":{"leadingEndCap":1,"trailingEndCap":1},"trailing":{"leadingEndCap":1,"trailingEndCap":1}},"isInset":true,"isDynamic":false}"#.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub show_ice_icon: bool,
    pub ice_icon: Option<Vec<u8>>,
    pub custom_ice_icon_is_template: bool,
    pub use_ice_bar: bool,
    pub ice_bar_location: IceBarLocation,
    pub ice_bar_pinned_location: Option<Vec<u8>>,
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
    pub menu_bar_appearance_configuration_v2: MenuBarAppearanceConfigurationV2,
    pub hotkeys: Vec<HotkeyBinding>,
    pub has_migrated_0_8_0: bool,
    pub has_migrated_0_10_0: bool,
    pub has_migrated_0_10_1: bool,
    pub has_migrated_0_11_10: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_ice_icon: true,
            ice_icon: None,
            custom_ice_icon_is_template: false,
            use_ice_bar: false,
            ice_bar_location: IceBarLocation::Dynamic,
            ice_bar_pinned_location: None,
            show_on_click: true,
            show_on_hover: false,
            show_on_scroll: true,
            item_spacing_offset: 0.0,
            auto_rehide: true,
            rehide_strategy: RehideStrategy::Smart,
            rehide_interval_secs: 15.0,
            hide_application_menus: true,
            show_section_dividers: false,
            enable_always_hidden_section: false,
            can_toggle_always_hidden_section: true,
            show_on_hover_delay_secs: 0.2,
            temp_show_interval_secs: 15.0,
            show_all_sections_on_user_drag: true,
            show_context_menu_on_right_click: true,
            menu_bar_appearance_configuration_v2: MenuBarAppearanceConfigurationV2::default(),
            hotkeys: HotkeyAction::ALL
                .into_iter()
                .map(|action| HotkeyBinding {
                    action,
                    key_combination: None,
                })
                .collect(),
            has_migrated_0_8_0: false,
            has_migrated_0_10_0: false,
            has_migrated_0_10_1: false,
            has_migrated_0_11_10: false,
        }
    }
}

pub mod keys {
    pub const SHOW_ICE_ICON: &str = "ShowIceIcon";
    pub const ICE_ICON: &str = "IceIcon";
    pub const CUSTOM_ICE_ICON_IS_TEMPLATE: &str = "CustomIceIconIsTemplate";
    pub const USE_ICE_BAR: &str = "UseIceBar";
    pub const ICE_BAR_LOCATION: &str = "IceBarLocation";
    pub const ICE_BAR_PINNED_LOCATION: &str = "IceBarPinnedLocation";
    pub const SHOW_ON_CLICK: &str = "ShowOnClick";
    pub const SHOW_ON_HOVER: &str = "ShowOnHover";
    pub const SHOW_ON_SCROLL: &str = "ShowOnScroll";
    pub const ITEM_SPACING_OFFSET: &str = "ItemSpacingOffset";
    pub const AUTO_REHIDE: &str = "AutoRehide";
    pub const REHIDE_STRATEGY: &str = "RehideStrategy";
    pub const REHIDE_INTERVAL: &str = "RehideInterval";
    pub const HIDE_APPLICATION_MENUS: &str = "HideApplicationMenus";
    pub const SHOW_SECTION_DIVIDERS: &str = "ShowSectionDividers";
    pub const ENABLE_ALWAYS_HIDDEN_SECTION: &str = "EnableAlwaysHiddenSection";
    pub const CAN_TOGGLE_ALWAYS_HIDDEN_SECTION: &str = "CanToggleAlwaysHiddenSection";
    pub const SHOW_ON_HOVER_DELAY: &str = "ShowOnHoverDelay";
    pub const TEMP_SHOW_INTERVAL: &str = "TempShowInterval";
    pub const SHOW_ALL_SECTIONS_ON_USER_DRAG: &str = "ShowAllSectionsOnUserDrag";
    pub const SHOW_CONTEXT_MENU_ON_RIGHT_CLICK: &str = "ShowContextMenuOnRightClick";
    pub const MENU_BAR_APPEARANCE_CONFIGURATION_V2: &str = "MenuBarAppearanceConfigurationV2";
    pub const HOTKEYS: &str = "Hotkeys";
    pub const HAS_MIGRATED_0_8_0: &str = "hasMigrated0_8_0";
    pub const HAS_MIGRATED_0_10_0: &str = "hasMigrated0_10_0";
    pub const HAS_MIGRATED_0_10_1: &str = "hasMigrated0_10_1";
    pub const HAS_MIGRATED_0_11_10: &str = "hasMigrated0_11_10";
}

pub trait SettingsStore {
    fn bool_for_key(&self, key: &str) -> Option<bool>;
    fn integer_for_key(&self, key: &str) -> Option<i64>;
    fn double_for_key(&self, key: &str) -> Option<f64>;
    fn data_for_key(&self, key: &str) -> Option<Vec<u8>>;
    fn set_bool(&mut self, key: &str, value: bool);
    fn set_integer(&mut self, key: &str, value: i64);
    fn set_double(&mut self, key: &str, value: f64);
    fn set_data(&mut self, key: &str, value: &[u8]);

    fn load_settings(&self) -> Settings {
        let defaults = Settings::default();

        Settings {
            show_ice_icon: self
                .bool_for_key(keys::SHOW_ICE_ICON)
                .unwrap_or(defaults.show_ice_icon),
            ice_icon: self.data_for_key(keys::ICE_ICON).or(defaults.ice_icon),
            custom_ice_icon_is_template: self
                .bool_for_key(keys::CUSTOM_ICE_ICON_IS_TEMPLATE)
                .unwrap_or(defaults.custom_ice_icon_is_template),
            use_ice_bar: self
                .bool_for_key(keys::USE_ICE_BAR)
                .unwrap_or(defaults.use_ice_bar),
            ice_bar_location: self
                .integer_for_key(keys::ICE_BAR_LOCATION)
                .and_then(IceBarLocation::from_raw_value)
                .unwrap_or(defaults.ice_bar_location),
            ice_bar_pinned_location: self
                .data_for_key(keys::ICE_BAR_PINNED_LOCATION)
                .or(defaults.ice_bar_pinned_location),
            show_on_click: self
                .bool_for_key(keys::SHOW_ON_CLICK)
                .unwrap_or(defaults.show_on_click),
            show_on_hover: self
                .bool_for_key(keys::SHOW_ON_HOVER)
                .unwrap_or(defaults.show_on_hover),
            show_on_scroll: self
                .bool_for_key(keys::SHOW_ON_SCROLL)
                .unwrap_or(defaults.show_on_scroll),
            item_spacing_offset: self
                .double_for_key(keys::ITEM_SPACING_OFFSET)
                .filter(|value| value.is_finite())
                .unwrap_or(defaults.item_spacing_offset),
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
            hide_application_menus: self
                .bool_for_key(keys::HIDE_APPLICATION_MENUS)
                .unwrap_or(defaults.hide_application_menus),
            show_section_dividers: self
                .bool_for_key(keys::SHOW_SECTION_DIVIDERS)
                .unwrap_or(defaults.show_section_dividers),
            enable_always_hidden_section: self
                .bool_for_key(keys::ENABLE_ALWAYS_HIDDEN_SECTION)
                .unwrap_or(defaults.enable_always_hidden_section),
            can_toggle_always_hidden_section: self
                .bool_for_key(keys::CAN_TOGGLE_ALWAYS_HIDDEN_SECTION)
                .unwrap_or(defaults.can_toggle_always_hidden_section),
            show_on_hover_delay_secs: self
                .double_for_key(keys::SHOW_ON_HOVER_DELAY)
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(defaults.show_on_hover_delay_secs),
            temp_show_interval_secs: self
                .double_for_key(keys::TEMP_SHOW_INTERVAL)
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(defaults.temp_show_interval_secs),
            show_all_sections_on_user_drag: self
                .bool_for_key(keys::SHOW_ALL_SECTIONS_ON_USER_DRAG)
                .unwrap_or(defaults.show_all_sections_on_user_drag),
            show_context_menu_on_right_click: self
                .bool_for_key(keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK)
                .unwrap_or(defaults.show_context_menu_on_right_click),
            menu_bar_appearance_configuration_v2: self
                .data_for_key(keys::MENU_BAR_APPEARANCE_CONFIGURATION_V2)
                .filter(|data| serde_json::from_slice::<serde_json::Value>(data).is_ok())
                .map(|raw_json| MenuBarAppearanceConfigurationV2 { raw_json })
                .unwrap_or(defaults.menu_bar_appearance_configuration_v2),
            hotkeys: self
                .data_for_key(keys::HOTKEYS)
                .and_then(|data| serde_json::from_slice::<Vec<HotkeyBinding>>(&data).ok())
                .unwrap_or(defaults.hotkeys),
            has_migrated_0_8_0: self
                .bool_for_key(keys::HAS_MIGRATED_0_8_0)
                .unwrap_or(defaults.has_migrated_0_8_0),
            has_migrated_0_10_0: self
                .bool_for_key(keys::HAS_MIGRATED_0_10_0)
                .unwrap_or(defaults.has_migrated_0_10_0),
            has_migrated_0_10_1: self
                .bool_for_key(keys::HAS_MIGRATED_0_10_1)
                .unwrap_or(defaults.has_migrated_0_10_1),
            has_migrated_0_11_10: self
                .bool_for_key(keys::HAS_MIGRATED_0_11_10)
                .unwrap_or(defaults.has_migrated_0_11_10),
        }
    }

    fn save_settings(&mut self, settings: &Settings) {
        self.set_bool(keys::SHOW_ICE_ICON, settings.show_ice_icon);
        if let Some(ice_icon) = settings.ice_icon.as_deref() {
            self.set_data(keys::ICE_ICON, ice_icon);
        }
        self.set_bool(
            keys::CUSTOM_ICE_ICON_IS_TEMPLATE,
            settings.custom_ice_icon_is_template,
        );
        self.set_bool(keys::USE_ICE_BAR, settings.use_ice_bar);
        self.set_integer(
            keys::ICE_BAR_LOCATION,
            settings.ice_bar_location.raw_value(),
        );
        if let Some(location) = settings.ice_bar_pinned_location.as_deref() {
            self.set_data(keys::ICE_BAR_PINNED_LOCATION, location);
        }
        self.set_bool(keys::SHOW_ON_CLICK, settings.show_on_click);
        self.set_bool(keys::SHOW_ON_HOVER, settings.show_on_hover);
        self.set_bool(keys::SHOW_ON_SCROLL, settings.show_on_scroll);
        self.set_double(keys::ITEM_SPACING_OFFSET, settings.item_spacing_offset);
        self.set_bool(keys::AUTO_REHIDE, settings.auto_rehide);
        self.set_integer(keys::REHIDE_STRATEGY, settings.rehide_strategy.raw_value());
        self.set_double(keys::REHIDE_INTERVAL, settings.rehide_interval_secs);
        self.set_bool(
            keys::HIDE_APPLICATION_MENUS,
            settings.hide_application_menus,
        );
        self.set_bool(keys::SHOW_SECTION_DIVIDERS, settings.show_section_dividers);
        self.set_bool(
            keys::ENABLE_ALWAYS_HIDDEN_SECTION,
            settings.enable_always_hidden_section,
        );
        self.set_bool(
            keys::CAN_TOGGLE_ALWAYS_HIDDEN_SECTION,
            settings.can_toggle_always_hidden_section,
        );
        self.set_double(keys::SHOW_ON_HOVER_DELAY, settings.show_on_hover_delay_secs);
        self.set_double(keys::TEMP_SHOW_INTERVAL, settings.temp_show_interval_secs);
        self.set_bool(
            keys::SHOW_ALL_SECTIONS_ON_USER_DRAG,
            settings.show_all_sections_on_user_drag,
        );
        self.set_bool(
            keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK,
            settings.show_context_menu_on_right_click,
        );
        self.set_data(
            keys::MENU_BAR_APPEARANCE_CONFIGURATION_V2,
            &settings.menu_bar_appearance_configuration_v2.raw_json,
        );
        if let Ok(data) = serde_json::to_vec(&settings.hotkeys) {
            self.set_data(keys::HOTKEYS, &data);
        }
        self.set_bool(keys::HAS_MIGRATED_0_8_0, settings.has_migrated_0_8_0);
        self.set_bool(keys::HAS_MIGRATED_0_10_0, settings.has_migrated_0_10_0);
        self.set_bool(keys::HAS_MIGRATED_0_10_1, settings.has_migrated_0_10_1);
        self.set_bool(keys::HAS_MIGRATED_0_11_10, settings.has_migrated_0_11_10);
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
        Data(Vec<u8>),
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

        fn data_for_key(&self, key: &str) -> Option<Vec<u8>> {
            match self.values.get(key) {
                Some(Value::Data(value)) => Some(value.clone()),
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

        fn set_data(&mut self, key: &str, value: &[u8]) {
            self.set(key, Value::Data(value.to_vec()));
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
        assert_eq!(settings.ice_icon, None);
        assert!(!settings.custom_ice_icon_is_template);
        assert!(!settings.use_ice_bar);
        assert_eq!(settings.ice_bar_location, IceBarLocation::Dynamic);
        assert_eq!(settings.ice_bar_pinned_location, None);
        assert!(settings.show_on_click);
        assert!(!settings.show_on_hover);
        assert!(settings.show_on_scroll);
        assert_eq!(settings.item_spacing_offset, 0.0);
        assert!(settings.auto_rehide);
        assert_eq!(settings.rehide_strategy, RehideStrategy::Smart);
        assert_eq!(settings.rehide_interval_secs, 15.0);
        assert!(settings.hide_application_menus);
        assert!(!settings.show_section_dividers);
        assert!(!settings.enable_always_hidden_section);
        assert!(settings.can_toggle_always_hidden_section);
        assert_eq!(settings.show_on_hover_delay_secs, 0.2);
        assert_eq!(settings.temp_show_interval_secs, 15.0);
        assert!(settings.show_all_sections_on_user_drag);
        assert!(settings.show_context_menu_on_right_click);
        assert_eq!(settings.hotkeys.len(), HotkeyAction::ALL.len());
        assert!(!settings.has_migrated_0_8_0);
        assert!(!settings.has_migrated_0_10_0);
        assert!(!settings.has_migrated_0_10_1);
        assert!(!settings.has_migrated_0_11_10);
    }

    #[test]
    fn loads_persisted_values_by_upstream_keys() {
        let mut store = MemorySettingsStore::default();
        store.set(keys::SHOW_ICE_ICON, Value::Bool(false));
        store.set(
            keys::ICE_ICON,
            Value::Data(br#"{"name":"custom"}"#.to_vec()),
        );
        store.set(keys::CUSTOM_ICE_ICON_IS_TEMPLATE, Value::Bool(true));
        store.set(keys::USE_ICE_BAR, Value::Bool(true));
        store.set(keys::ICE_BAR_LOCATION, Value::Integer(2));
        store.set(
            keys::ICE_BAR_PINNED_LOCATION,
            Value::Data(br#"{"x":1,"y":2}"#.to_vec()),
        );
        store.set(keys::SHOW_ON_CLICK, Value::Bool(false));
        store.set(keys::SHOW_ON_HOVER, Value::Bool(true));
        store.set(keys::SHOW_ON_SCROLL, Value::Bool(false));
        store.set(keys::ITEM_SPACING_OFFSET, Value::Double(3.0));
        store.set(keys::AUTO_REHIDE, Value::Bool(false));
        store.set(keys::REHIDE_STRATEGY, Value::Integer(1));
        store.set(keys::REHIDE_INTERVAL, Value::Double(30.0));
        store.set(keys::HIDE_APPLICATION_MENUS, Value::Bool(false));
        store.set(keys::SHOW_SECTION_DIVIDERS, Value::Bool(true));
        store.set(keys::ENABLE_ALWAYS_HIDDEN_SECTION, Value::Bool(true));
        store.set(keys::CAN_TOGGLE_ALWAYS_HIDDEN_SECTION, Value::Bool(false));
        store.set(keys::SHOW_ON_HOVER_DELAY, Value::Double(0.5));
        store.set(keys::TEMP_SHOW_INTERVAL, Value::Double(45.0));
        store.set(keys::SHOW_ALL_SECTIONS_ON_USER_DRAG, Value::Bool(false));
        store.set(keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK, Value::Bool(false));
        store.set(
            keys::MENU_BAR_APPEARANCE_CONFIGURATION_V2,
            Value::Data(br#"{"isInset":false}"#.to_vec()),
        );
        store.set(
            keys::HOTKEYS,
            Value::Data(br#"[{"action":"ToggleHiddenSection","keyCombination":[49,8]}]"#.to_vec()),
        );
        store.set(keys::HAS_MIGRATED_0_8_0, Value::Bool(true));
        store.set(keys::HAS_MIGRATED_0_10_0, Value::Bool(true));
        store.set(keys::HAS_MIGRATED_0_10_1, Value::Bool(true));
        store.set(keys::HAS_MIGRATED_0_11_10, Value::Bool(true));

        let settings = store.load_settings();

        assert!(!settings.show_ice_icon);
        assert_eq!(settings.ice_icon, Some(br#"{"name":"custom"}"#.to_vec()));
        assert!(settings.custom_ice_icon_is_template);
        assert!(settings.use_ice_bar);
        assert_eq!(settings.ice_bar_location, IceBarLocation::IceIcon);
        assert_eq!(
            settings.ice_bar_pinned_location,
            Some(br#"{"x":1,"y":2}"#.to_vec())
        );
        assert!(!settings.show_on_click);
        assert!(settings.show_on_hover);
        assert!(!settings.show_on_scroll);
        assert_eq!(settings.item_spacing_offset, 3.0);
        assert!(!settings.auto_rehide);
        assert_eq!(settings.rehide_strategy, RehideStrategy::Timed);
        assert_eq!(settings.rehide_interval_secs, 30.0);
        assert!(!settings.hide_application_menus);
        assert!(settings.show_section_dividers);
        assert!(settings.enable_always_hidden_section);
        assert!(!settings.can_toggle_always_hidden_section);
        assert_eq!(settings.show_on_hover_delay_secs, 0.5);
        assert_eq!(settings.temp_show_interval_secs, 45.0);
        assert!(!settings.show_all_sections_on_user_drag);
        assert!(!settings.show_context_menu_on_right_click);
        assert_eq!(
            settings.menu_bar_appearance_configuration_v2.raw_json,
            br#"{"isInset":false}"#.to_vec()
        );
        assert_eq!(settings.hotkeys.len(), 1);
        assert_eq!(
            settings.hotkeys[0].action,
            HotkeyAction::ToggleHiddenSection
        );
        assert_eq!(
            settings.hotkeys[0].key_combination,
            Some(KeyCombination {
                key_code: 49,
                modifiers: 8,
            })
        );
        assert!(settings.has_migrated_0_8_0);
        assert!(settings.has_migrated_0_10_0);
        assert!(settings.has_migrated_0_10_1);
        assert!(settings.has_migrated_0_11_10);
    }

    #[test]
    fn invalid_raw_values_fall_back_to_defaults() {
        let mut store = MemorySettingsStore::default();
        store.set(keys::ICE_BAR_LOCATION, Value::Integer(99));
        store.set(keys::REHIDE_STRATEGY, Value::Integer(99));
        store.set(keys::REHIDE_INTERVAL, Value::Double(-1.0));
        store.set(keys::SHOW_ON_HOVER_DELAY, Value::Double(-1.0));
        store.set(keys::TEMP_SHOW_INTERVAL, Value::Double(0.0));
        store.set(
            keys::MENU_BAR_APPEARANCE_CONFIGURATION_V2,
            Value::Data(b"not json".to_vec()),
        );
        store.set(keys::HOTKEYS, Value::Data(b"not json".to_vec()));

        let settings = store.load_settings();

        assert_eq!(settings.ice_bar_location, IceBarLocation::Dynamic);
        assert_eq!(settings.rehide_strategy, RehideStrategy::Smart);
        assert_eq!(settings.rehide_interval_secs, 15.0);
        assert_eq!(settings.show_on_hover_delay_secs, 0.2);
        assert_eq!(settings.temp_show_interval_secs, 15.0);
        assert_eq!(
            settings.menu_bar_appearance_configuration_v2,
            MenuBarAppearanceConfigurationV2::default()
        );
        assert_eq!(settings.hotkeys.len(), HotkeyAction::ALL.len());
    }

    #[test]
    fn saves_values_to_upstream_keys() {
        let mut store = MemorySettingsStore::default();
        let settings = Settings {
            show_ice_icon: false,
            ice_icon: Some(br#"{"name":"custom"}"#.to_vec()),
            custom_ice_icon_is_template: true,
            use_ice_bar: true,
            ice_bar_location: IceBarLocation::MousePointer,
            ice_bar_pinned_location: Some(br#"{"x":3,"y":4}"#.to_vec()),
            show_on_click: false,
            show_on_hover: true,
            show_on_scroll: false,
            item_spacing_offset: 4.0,
            auto_rehide: false,
            rehide_strategy: RehideStrategy::FocusedApp,
            rehide_interval_secs: 60.0,
            hide_application_menus: false,
            show_section_dividers: true,
            enable_always_hidden_section: true,
            can_toggle_always_hidden_section: false,
            show_on_hover_delay_secs: 0.7,
            temp_show_interval_secs: 20.0,
            show_all_sections_on_user_drag: false,
            show_context_menu_on_right_click: false,
            menu_bar_appearance_configuration_v2: MenuBarAppearanceConfigurationV2 {
                raw_json: br#"{"isDynamic":true}"#.to_vec(),
            },
            hotkeys: vec![HotkeyBinding {
                action: HotkeyAction::SearchMenuBarItems,
                key_combination: Some(KeyCombination {
                    key_code: 35,
                    modifiers: 8,
                }),
            }],
            has_migrated_0_8_0: true,
            has_migrated_0_10_0: true,
            has_migrated_0_10_1: true,
            has_migrated_0_11_10: true,
        };

        store.save_settings(&settings);

        assert_eq!(store.bool_for_key(keys::SHOW_ICE_ICON), Some(false));
        assert_eq!(
            store.data_for_key(keys::ICE_ICON),
            Some(br#"{"name":"custom"}"#.to_vec())
        );
        assert_eq!(
            store.bool_for_key(keys::CUSTOM_ICE_ICON_IS_TEMPLATE),
            Some(true)
        );
        assert_eq!(store.bool_for_key(keys::USE_ICE_BAR), Some(true));
        assert_eq!(
            store.integer_for_key(keys::ICE_BAR_LOCATION),
            Some(IceBarLocation::MousePointer.raw_value())
        );
        assert_eq!(
            store.data_for_key(keys::ICE_BAR_PINNED_LOCATION),
            Some(br#"{"x":3,"y":4}"#.to_vec())
        );
        assert_eq!(store.bool_for_key(keys::SHOW_ON_CLICK), Some(false));
        assert_eq!(store.bool_for_key(keys::SHOW_ON_HOVER), Some(true));
        assert_eq!(store.bool_for_key(keys::SHOW_ON_SCROLL), Some(false));
        assert_eq!(store.double_for_key(keys::ITEM_SPACING_OFFSET), Some(4.0));
        assert_eq!(store.bool_for_key(keys::AUTO_REHIDE), Some(false));
        assert_eq!(
            store.integer_for_key(keys::REHIDE_STRATEGY),
            Some(RehideStrategy::FocusedApp.raw_value())
        );
        assert_eq!(store.double_for_key(keys::REHIDE_INTERVAL), Some(60.0));
        assert_eq!(
            store.bool_for_key(keys::HIDE_APPLICATION_MENUS),
            Some(false)
        );
        assert_eq!(store.bool_for_key(keys::SHOW_SECTION_DIVIDERS), Some(true));
        assert_eq!(
            store.bool_for_key(keys::ENABLE_ALWAYS_HIDDEN_SECTION),
            Some(true)
        );
        assert_eq!(
            store.bool_for_key(keys::CAN_TOGGLE_ALWAYS_HIDDEN_SECTION),
            Some(false)
        );
        assert_eq!(store.double_for_key(keys::SHOW_ON_HOVER_DELAY), Some(0.7));
        assert_eq!(store.double_for_key(keys::TEMP_SHOW_INTERVAL), Some(20.0));
        assert_eq!(
            store.bool_for_key(keys::SHOW_ALL_SECTIONS_ON_USER_DRAG),
            Some(false)
        );
        assert_eq!(
            store.bool_for_key(keys::SHOW_CONTEXT_MENU_ON_RIGHT_CLICK),
            Some(false)
        );
        assert_eq!(
            store.data_for_key(keys::MENU_BAR_APPEARANCE_CONFIGURATION_V2),
            Some(br#"{"isDynamic":true}"#.to_vec())
        );
        assert!(store
            .data_for_key(keys::HOTKEYS)
            .is_some_and(|data| data.contains(&b'S')));
        assert_eq!(store.bool_for_key(keys::HAS_MIGRATED_0_8_0), Some(true));
        assert_eq!(store.bool_for_key(keys::HAS_MIGRATED_0_10_0), Some(true));
        assert_eq!(store.bool_for_key(keys::HAS_MIGRATED_0_10_1), Some(true));
        assert_eq!(store.bool_for_key(keys::HAS_MIGRATED_0_11_10), Some(true));
    }
}

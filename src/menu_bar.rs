use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn max_x(self) -> f64 {
        self.min_x + self.width
    }

    pub fn intersects(self, other: Self) -> bool {
        self.min_x < other.max_x()
            && self.max_x() > other.min_x
            && self.min_y < other.min_y + other.height
            && self.min_y + self.height > other.min_y
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuBarItemInfo {
    pub namespace: String,
    pub title: String,
}

impl MenuBarItemInfo {
    pub fn new(namespace: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            title: title.into(),
        }
    }

    pub fn ice_icon() -> Self {
        Self::new(namespaces::ICE, control_items::ICE_ICON)
    }

    pub fn hidden_control_item() -> Self {
        Self::new(namespaces::ICE, control_items::HIDDEN)
    }

    pub fn always_hidden_control_item() -> Self {
        Self::new(namespaces::ICE, control_items::ALWAYS_HIDDEN)
    }

    pub fn clock() -> Self {
        Self::new(namespaces::CONTROL_CENTER, "Clock")
    }

    pub fn siri() -> Self {
        Self::new(namespaces::SYSTEM_UI_SERVER, "Siri")
    }

    pub fn control_center() -> Self {
        Self::new(namespaces::CONTROL_CENTER, "BentoBox")
    }

    pub fn audio_video_module() -> Self {
        Self::new(namespaces::CONTROL_CENTER, "AudioVideoModule")
    }

    pub fn face_time() -> Self {
        Self::new(namespaces::CONTROL_CENTER, "FaceTime")
    }

    pub fn music_recognition() -> Self {
        Self::new(namespaces::CONTROL_CENTER, "MusicRecognition")
    }

    pub fn is_immovable(&self) -> bool {
        [Self::clock(), Self::siri(), Self::control_center()].contains(self)
    }

    pub fn can_be_hidden(&self) -> bool {
        ![
            Self::audio_video_module(),
            Self::face_time(),
            Self::music_recognition(),
        ]
        .contains(self)
    }

    pub fn is_ice_control_item(&self) -> bool {
        [
            Self::ice_icon(),
            Self::hidden_control_item(),
            Self::always_hidden_control_item(),
        ]
        .contains(self)
    }

    pub fn encoded(&self) -> String {
        format!("{}:{}", self.namespace, self.title)
    }

    pub fn display_name(
        &self,
        owner_name: Option<&str>,
        bundle_identifier: Option<&str>,
    ) -> String {
        let fallback = "Unknown";
        let best_name = owner_name
            .or(bundle_identifier)
            .unwrap_or(fallback)
            .to_string();

        match bundle_identifier.unwrap_or(&self.namespace) {
            namespaces::CONTROL_CENTER => match self.title.as_str() {
                "AccessibilityShortcuts" => "Accessibility Shortcuts".to_string(),
                "BentoBox" => best_name,
                "FocusModes" => "Focus".to_string(),
                "KeyboardBrightness" => "Keyboard Brightness".to_string(),
                "MusicRecognition" => "Music Recognition".to_string(),
                "NowPlaying" => "Now Playing".to_string(),
                "ScreenMirroring" => "Screen Mirroring".to_string(),
                "StageManager" => "Stage Manager".to_string(),
                "UserSwitcher" => "Fast User Switching".to_string(),
                "WiFi" => "Wi-Fi".to_string(),
                _ => self.title.clone(),
            },
            namespaces::SYSTEM_UI_SERVER => match self.title.as_str() {
                "TimeMachine.TMMenuExtraHost" | "TimeMachineMenuExtra.TMMenuExtraHost" => {
                    "Time Machine".to_string()
                }
                _ => self.title.clone(),
            },
            "com.apple.Passwords.MenuBarExtra" => "Passwords".to_string(),
            _ => best_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuBarItemSnapshot {
    pub window_id: u32,
    pub frame: Rect,
    pub info: MenuBarItemInfo,
    pub owner_pid: i32,
    pub owner_name: Option<String>,
    pub bundle_identifier: Option<String>,
    pub display_id: Option<u32>,
    pub is_on_screen: bool,
    pub is_on_active_space: bool,
}

impl MenuBarItemSnapshot {
    pub fn new(window_id: u32, frame: Rect, info: MenuBarItemInfo) -> Self {
        Self {
            window_id,
            frame,
            info,
            owner_pid: 0,
            owner_name: None,
            bundle_identifier: None,
            display_id: None,
            is_on_screen: true,
            is_on_active_space: true,
        }
    }

    pub fn is_hidden_control_item(&self) -> bool {
        self.info == MenuBarItemInfo::hidden_control_item()
    }

    pub fn is_always_hidden_control_item(&self) -> bool {
        self.info == MenuBarItemInfo::always_hidden_control_item()
    }

    pub fn is_ice_control_item(&self) -> bool {
        self.info.is_ice_control_item()
    }

    pub fn is_movable(&self) -> bool {
        !self.info.is_immovable()
    }

    pub fn can_be_hidden(&self) -> bool {
        self.info.can_be_hidden()
    }

    pub fn display_name(&self) -> String {
        self.info.display_name(
            self.owner_name.as_deref(),
            self.bundle_identifier.as_deref(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionPartition {
    pub visible: Vec<MenuBarItemSnapshot>,
    pub hidden: Vec<MenuBarItemSnapshot>,
    pub always_hidden: Vec<MenuBarItemSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarSectionName {
    Visible,
    Hidden,
    AlwaysHidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionItemCache {
    pub visible: Vec<MenuBarItemSnapshot>,
    pub hidden: Vec<MenuBarItemSnapshot>,
    pub always_hidden: Vec<MenuBarItemSnapshot>,
}

impl SectionItemCache {
    pub fn new(partition: SectionPartition) -> Self {
        Self {
            visible: partition.visible,
            hidden: partition.hidden,
            always_hidden: partition.always_hidden,
        }
    }

    pub fn managed_items(&self, section: MenuBarSectionName) -> Vec<MenuBarItemSnapshot> {
        self.items(section)
            .iter()
            .filter(|item| item.is_movable() && item.can_be_hidden())
            .filter(|item| !item.is_ice_control_item() || item.info == MenuBarItemInfo::ice_icon())
            .cloned()
            .collect()
    }

    pub fn all_managed_items(&self) -> Vec<MenuBarItemSnapshot> {
        [
            MenuBarSectionName::Visible,
            MenuBarSectionName::Hidden,
            MenuBarSectionName::AlwaysHidden,
        ]
        .into_iter()
        .flat_map(|section| self.managed_items(section))
        .collect()
    }

    pub fn section_for_info(&self, info: &MenuBarItemInfo) -> Option<MenuBarSectionName> {
        [
            (MenuBarSectionName::Visible, &self.visible),
            (MenuBarSectionName::Hidden, &self.hidden),
            (MenuBarSectionName::AlwaysHidden, &self.always_hidden),
        ]
        .into_iter()
        .find_map(|(section, items)| {
            items
                .iter()
                .any(|item| item.info == *info)
                .then_some(section)
        })
    }

    pub fn apply_move_plan(&self, plan: &MovePlan) -> Result<Self, MovePlanError> {
        let source_section = self
            .section_for_info(&plan.item)
            .ok_or_else(|| MovePlanError::ItemNotFound(plan.item.clone()))?;
        let destination_section = self.destination_section(&plan.destination)?;
        let moving_item = self
            .items(source_section)
            .iter()
            .find(|item| item.info == plan.item)
            .cloned()
            .ok_or_else(|| MovePlanError::ItemNotFound(plan.item.clone()))?;

        if !moving_item.is_movable() {
            return Err(MovePlanError::ItemNotMovable(plan.item.clone()));
        }

        if destination_section != MenuBarSectionName::Visible && !moving_item.can_be_hidden() {
            return Err(MovePlanError::ItemCannotBeHidden(plan.item.clone()));
        }

        let mut next = self.clone();
        next.items_mut(source_section)
            .retain(|item| item.info != plan.item);
        next.insert_moved_item(destination_section, moving_item, &plan.destination)?;
        Ok(next)
    }

    fn items(&self, section: MenuBarSectionName) -> &[MenuBarItemSnapshot] {
        match section {
            MenuBarSectionName::Visible => &self.visible,
            MenuBarSectionName::Hidden => &self.hidden,
            MenuBarSectionName::AlwaysHidden => &self.always_hidden,
        }
    }

    fn items_mut(&mut self, section: MenuBarSectionName) -> &mut Vec<MenuBarItemSnapshot> {
        match section {
            MenuBarSectionName::Visible => &mut self.visible,
            MenuBarSectionName::Hidden => &mut self.hidden,
            MenuBarSectionName::AlwaysHidden => &mut self.always_hidden,
        }
    }

    fn destination_section(
        &self,
        destination: &MoveDestination,
    ) -> Result<MenuBarSectionName, MovePlanError> {
        match destination.target() {
            target if target == &MenuBarItemInfo::hidden_control_item() => match destination {
                MoveDestination::LeftOfItem(_) => Ok(MenuBarSectionName::Hidden),
                MoveDestination::RightOfItem(_) => Ok(MenuBarSectionName::Visible),
            },
            target if target == &MenuBarItemInfo::always_hidden_control_item() => match destination
            {
                MoveDestination::LeftOfItem(_) => Ok(MenuBarSectionName::AlwaysHidden),
                MoveDestination::RightOfItem(_) => Ok(MenuBarSectionName::Hidden),
            },
            target => self
                .section_for_info(target)
                .ok_or_else(|| MovePlanError::TargetNotFound(target.clone())),
        }
    }

    fn insert_moved_item(
        &mut self,
        section: MenuBarSectionName,
        item: MenuBarItemSnapshot,
        destination: &MoveDestination,
    ) -> Result<(), MovePlanError> {
        let target = destination.target();
        let items = self.items_mut(section);

        if target == &MenuBarItemInfo::hidden_control_item()
            || target == &MenuBarItemInfo::always_hidden_control_item()
        {
            match destination {
                MoveDestination::LeftOfItem(_) => items.push(item),
                MoveDestination::RightOfItem(_) => items.insert(0, item),
            }
            return Ok(());
        }

        let target_index = items
            .iter()
            .position(|candidate| candidate.info == *target)
            .ok_or_else(|| MovePlanError::TargetNotFound(target.clone()))?;
        let insert_index = match destination {
            MoveDestination::LeftOfItem(_) => target_index,
            MoveDestination::RightOfItem(_) => target_index + 1,
        };
        items.insert(insert_index, item);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveDestination {
    LeftOfItem(MenuBarItemInfo),
    RightOfItem(MenuBarItemInfo),
}

impl MoveDestination {
    pub fn target(&self) -> &MenuBarItemInfo {
        match self {
            Self::LeftOfItem(item) | Self::RightOfItem(item) => item,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePlan {
    pub item: MenuBarItemInfo,
    pub destination: MoveDestination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovePlanError {
    ItemNotFound(MenuBarItemInfo),
    TargetNotFound(MenuBarItemInfo),
    ItemNotMovable(MenuBarItemInfo),
    ItemCannotBeHidden(MenuBarItemInfo),
}

pub fn plan_item_move(
    cache: &SectionItemCache,
    item: MenuBarItemInfo,
    destination: MoveDestination,
) -> Result<MovePlan, MovePlanError> {
    let source_section = cache
        .section_for_info(&item)
        .ok_or_else(|| MovePlanError::ItemNotFound(item.clone()))?;
    let destination_section = cache.destination_section(&destination)?;
    let moving_item = cache
        .items(source_section)
        .iter()
        .find(|candidate| candidate.info == item)
        .ok_or_else(|| MovePlanError::ItemNotFound(item.clone()))?;

    if !moving_item.is_movable() {
        return Err(MovePlanError::ItemNotMovable(item));
    }

    if destination_section != MenuBarSectionName::Visible && !moving_item.can_be_hidden() {
        return Err(MovePlanError::ItemCannotBeHidden(item));
    }

    Ok(MovePlan { item, destination })
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MenuBarInventory {
    pub items: Vec<MenuBarItemSnapshot>,
}

impl MenuBarInventory {
    pub fn new(items: Vec<MenuBarItemSnapshot>) -> Self {
        Self { items }
    }

    pub fn filtered(&self, filter: MenuBarInventoryFilter) -> Vec<MenuBarItemSnapshot> {
        filter_inventory(self.items.clone(), filter)
    }

    pub fn partition(&self) -> SectionPartition {
        partition_sections(self.items.clone())
    }

    pub fn debug_lines(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|item| {
                format!(
                    "#{window_id} {identity} {name} x={x:.1} w={width:.1} pid={pid} display={display}",
                    window_id = item.window_id,
                    identity = item.info.encoded(),
                    name = item.display_name(),
                    x = item.frame.min_x,
                    width = item.frame.width,
                    pid = item.owner_pid,
                    display = item
                        .display_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "-".to_string())
                )
            })
            .collect()
    }
}

pub fn partition_sections(mut items: Vec<MenuBarItemSnapshot>) -> SectionPartition {
    items.sort_by(|left, right| {
        left.frame
            .min_x
            .partial_cmp(&right.frame.min_x)
            .unwrap_or(Ordering::Equal)
    });

    let hidden_control = items
        .iter()
        .find(|item| item.is_hidden_control_item())
        .map(|item| item.frame);
    let always_hidden_control = items
        .iter()
        .find(|item| item.is_always_hidden_control_item())
        .map(|item| item.frame);

    let mut visible = Vec::new();
    let mut hidden = Vec::new();
    let mut always_hidden = Vec::new();

    for item in items
        .into_iter()
        .filter(|item| !item.is_hidden_control_item() && !item.is_always_hidden_control_item())
    {
        match (hidden_control, always_hidden_control) {
            (Some(hidden_control), _) if item.frame.min_x >= hidden_control.max_x() => {
                visible.push(item);
            }
            (Some(hidden_control), Some(always_hidden_control))
                if item.frame.max_x() <= hidden_control.min_x
                    && item.frame.min_x >= always_hidden_control.max_x() =>
            {
                hidden.push(item);
            }
            (_, Some(always_hidden_control))
                if item.frame.max_x() <= always_hidden_control.min_x =>
            {
                always_hidden.push(item);
            }
            (Some(hidden_control), None) if item.frame.max_x() <= hidden_control.min_x => {
                hidden.push(item);
            }
            _ => visible.push(item),
        }
    }

    SectionPartition {
        visible,
        hidden,
        always_hidden,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuBarInventoryFilter {
    pub display_id: Option<u32>,
    pub on_screen_only: bool,
    pub active_space_only: bool,
    pub include_ice_control_items: bool,
}

impl Default for MenuBarInventoryFilter {
    fn default() -> Self {
        Self {
            display_id: None,
            on_screen_only: false,
            active_space_only: true,
            include_ice_control_items: true,
        }
    }
}

pub fn filter_inventory(
    items: impl IntoIterator<Item = MenuBarItemSnapshot>,
    filter: MenuBarInventoryFilter,
) -> Vec<MenuBarItemSnapshot> {
    let mut filtered = items
        .into_iter()
        .filter(|item| {
            filter
                .display_id
                .is_none_or(|display_id| item.display_id == Some(display_id))
                && (!filter.on_screen_only || item.is_on_screen)
                && (!filter.active_space_only || item.is_on_active_space)
                && (filter.include_ice_control_items || !item.is_ice_control_item())
        })
        .collect::<Vec<_>>();

    filtered.sort_by(|left, right| {
        left.frame
            .min_x
            .partial_cmp(&right.frame.min_x)
            .unwrap_or(Ordering::Equal)
    });
    filtered
}

pub mod namespaces {
    pub const ICE: &str = "dev.undivisible.rs_ice";
    pub const CONTROL_CENTER: &str = "com.apple.controlcenter";
    pub const SYSTEM_UI_SERVER: &str = "com.apple.systemuiserver";
    pub const SPECIAL: &str = "Special";
}

pub mod control_items {
    pub const ICE_ICON: &str = "SItem";
    pub const HIDDEN: &str = "HItem";
    pub const ALWAYS_HIDDEN: &str = "AHItem";
    pub const NEW_ITEMS: &str = "NewItems";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, x: f64, width: f64, namespace: &str, title: &str) -> MenuBarItemSnapshot {
        MenuBarItemSnapshot::new(
            id,
            Rect {
                min_x: x,
                min_y: 0.0,
                width,
                height: 24.0,
            },
            MenuBarItemInfo::new(namespace, title),
        )
    }

    #[test]
    fn identifies_upstream_immovable_and_non_hideable_items() {
        assert!(MenuBarItemInfo::clock().is_immovable());
        assert!(MenuBarItemInfo::siri().is_immovable());
        assert!(MenuBarItemInfo::control_center().is_immovable());
        assert!(!MenuBarItemInfo::music_recognition().can_be_hidden());
        assert!(MenuBarItemInfo::new("com.example.App", "Item").can_be_hidden());
    }

    #[test]
    fn partitions_visible_and_hidden_items_around_hidden_divider() {
        let partition = partition_sections(vec![
            item(1, 90.0, 20.0, "com.example.visible", "Visible"),
            MenuBarItemSnapshot::new(
                2,
                Rect {
                    min_x: 60.0,
                    min_y: 0.0,
                    width: 1.0,
                    height: 24.0,
                },
                MenuBarItemInfo::hidden_control_item(),
            ),
            item(3, 10.0, 20.0, "com.example.hidden", "Hidden"),
        ]);

        assert_eq!(partition.visible.len(), 1);
        assert_eq!(partition.hidden.len(), 1);
        assert_eq!(partition.always_hidden.len(), 0);
        assert_eq!(partition.visible[0].window_id, 1);
        assert_eq!(partition.hidden[0].window_id, 3);
    }

    #[test]
    fn partitions_always_hidden_items_after_always_hidden_divider() {
        let partition = partition_sections(vec![
            item(1, 110.0, 20.0, "com.example.visible", "Visible"),
            MenuBarItemSnapshot::new(
                2,
                Rect {
                    min_x: 80.0,
                    min_y: 0.0,
                    width: 1.0,
                    height: 24.0,
                },
                MenuBarItemInfo::hidden_control_item(),
            ),
            item(3, 50.0, 20.0, "com.example.hidden", "Hidden"),
            MenuBarItemSnapshot::new(
                4,
                Rect {
                    min_x: 30.0,
                    min_y: 0.0,
                    width: 1.0,
                    height: 24.0,
                },
                MenuBarItemInfo::always_hidden_control_item(),
            ),
            item(5, 0.0, 20.0, "com.example.always-hidden", "AlwaysHidden"),
        ]);

        assert_eq!(partition.visible.len(), 1);
        assert_eq!(partition.hidden.len(), 1);
        assert_eq!(partition.always_hidden.len(), 1);
        assert_eq!(partition.always_hidden[0].window_id, 5);
    }

    #[test]
    fn display_names_match_upstream_special_cases() {
        assert_eq!(
            MenuBarItemInfo::new(namespaces::CONTROL_CENTER, "WiFi")
                .display_name(Some("Control Center"), Some(namespaces::CONTROL_CENTER)),
            "Wi-Fi"
        );
        assert_eq!(
            MenuBarItemInfo::new(namespaces::SYSTEM_UI_SERVER, "TimeMachine.TMMenuExtraHost")
                .display_name(Some("SystemUIServer"), Some(namespaces::SYSTEM_UI_SERVER)),
            "Time Machine"
        );
        assert_eq!(
            MenuBarItemInfo::new("com.apple.Passwords.MenuBarExtra", "")
                .display_name(Some("Passwords"), Some("com.apple.Passwords.MenuBarExtra")),
            "Passwords"
        );
    }

    #[test]
    fn filters_inventory_by_display_screen_space_and_control_items() {
        let mut first = item(1, 40.0, 10.0, "com.example.one", "One");
        first.display_id = Some(1);
        first.is_on_screen = true;
        first.is_on_active_space = true;

        let mut hidden_control = MenuBarItemSnapshot::new(
            2,
            Rect {
                min_x: 60.0,
                min_y: 0.0,
                width: 1.0,
                height: 24.0,
            },
            MenuBarItemInfo::hidden_control_item(),
        );
        hidden_control.display_id = Some(1);

        let mut offscreen = item(3, 20.0, 10.0, "com.example.two", "Two");
        offscreen.display_id = Some(1);
        offscreen.is_on_screen = false;

        let mut other_display = item(4, 10.0, 10.0, "com.example.three", "Three");
        other_display.display_id = Some(2);

        let filtered = filter_inventory(
            vec![first.clone(), hidden_control, offscreen, other_display],
            MenuBarInventoryFilter {
                display_id: Some(1),
                on_screen_only: true,
                active_space_only: true,
                include_ice_control_items: false,
            },
        );

        assert_eq!(filtered, vec![first]);
    }

    #[test]
    fn encoded_identity_uses_upstream_namespace_title_shape() {
        assert_eq!(
            MenuBarItemInfo::new("com.example.App", "Clock:Extra").encoded(),
            "com.example.App:Clock:Extra"
        );
    }

    #[test]
    fn inventory_debug_lines_include_identity_position_and_owner() {
        let mut snapshot = item(42, 12.0, 24.0, "com.example.App", "Item");
        snapshot.owner_pid = 123;
        snapshot.owner_name = Some("Example".to_string());
        snapshot.bundle_identifier = Some("com.example.App".to_string());
        snapshot.display_id = Some(7);

        let lines = MenuBarInventory::new(vec![snapshot]).debug_lines();

        assert_eq!(
            lines,
            vec!["#42 com.example.App:Item Example x=12.0 w=24.0 pid=123 display=7"]
        );
    }

    #[test]
    fn section_cache_exposes_only_managed_items() {
        let partition = SectionPartition {
            visible: vec![
                item(1, 100.0, 20.0, "com.example.visible", "Visible"),
                item(2, 120.0, 20.0, namespaces::CONTROL_CENTER, "Clock"),
                item(
                    3,
                    140.0,
                    20.0,
                    namespaces::CONTROL_CENTER,
                    "AudioVideoModule",
                ),
                MenuBarItemSnapshot::new(
                    4,
                    Rect {
                        min_x: 160.0,
                        min_y: 0.0,
                        width: 20.0,
                        height: 24.0,
                    },
                    MenuBarItemInfo::ice_icon(),
                ),
                MenuBarItemSnapshot::new(
                    5,
                    Rect {
                        min_x: 180.0,
                        min_y: 0.0,
                        width: 1.0,
                        height: 24.0,
                    },
                    MenuBarItemInfo::hidden_control_item(),
                ),
            ],
            hidden: vec![item(6, 60.0, 20.0, "com.example.hidden", "Hidden")],
            always_hidden: vec![item(7, 20.0, 20.0, "com.example.always", "Always")],
        };

        let cache = SectionItemCache::new(partition);
        let visible = cache.managed_items(MenuBarSectionName::Visible);

        assert_eq!(
            visible
                .iter()
                .map(|item| item.info.clone())
                .collect::<Vec<_>>(),
            vec![
                MenuBarItemInfo::new("com.example.visible", "Visible"),
                MenuBarItemInfo::ice_icon(),
            ]
        );
    }

    #[test]
    fn section_cache_finds_section_by_item_identity() {
        let cache = SectionItemCache::new(SectionPartition {
            visible: vec![item(1, 100.0, 20.0, "com.example.visible", "Visible")],
            hidden: vec![item(2, 60.0, 20.0, "com.example.hidden", "Hidden")],
            always_hidden: vec![item(3, 20.0, 20.0, "com.example.always", "Always")],
        });

        assert_eq!(
            cache.section_for_info(&MenuBarItemInfo::new("com.example.hidden", "Hidden")),
            Some(MenuBarSectionName::Hidden)
        );
        assert_eq!(
            cache.section_for_info(&MenuBarItemInfo::new("com.example.missing", "Missing")),
            None
        );
    }

    #[test]
    fn move_plan_moves_visible_item_left_of_hidden_divider_into_hidden_section() {
        let cache = SectionItemCache::new(SectionPartition {
            visible: vec![item(1, 100.0, 20.0, "com.example.visible", "Visible")],
            hidden: vec![item(2, 60.0, 20.0, "com.example.hidden", "Hidden")],
            always_hidden: Vec::new(),
        });
        let moving = MenuBarItemInfo::new("com.example.visible", "Visible");

        let plan = plan_item_move(
            &cache,
            moving.clone(),
            MoveDestination::LeftOfItem(MenuBarItemInfo::hidden_control_item()),
        )
        .unwrap();
        let next = cache.apply_move_plan(&plan).unwrap();

        assert_eq!(next.visible, Vec::new());
        assert_eq!(
            next.hidden
                .iter()
                .map(|item| item.info.clone())
                .collect::<Vec<_>>(),
            vec![MenuBarItemInfo::new("com.example.hidden", "Hidden"), moving]
        );
    }

    #[test]
    fn move_plan_preserves_relative_order_around_regular_targets() {
        let cache = SectionItemCache::new(SectionPartition {
            visible: vec![
                item(1, 100.0, 20.0, "com.example.one", "One"),
                item(2, 120.0, 20.0, "com.example.two", "Two"),
                item(3, 140.0, 20.0, "com.example.three", "Three"),
            ],
            hidden: Vec::new(),
            always_hidden: Vec::new(),
        });

        let plan = plan_item_move(
            &cache,
            MenuBarItemInfo::new("com.example.three", "Three"),
            MoveDestination::LeftOfItem(MenuBarItemInfo::new("com.example.one", "One")),
        )
        .unwrap();
        let next = cache.apply_move_plan(&plan).unwrap();

        assert_eq!(
            next.visible
                .iter()
                .map(|item| item.info.title.clone())
                .collect::<Vec<_>>(),
            vec!["Three", "One", "Two"]
        );
    }

    #[test]
    fn move_plan_rejects_immovable_items() {
        let cache = SectionItemCache::new(SectionPartition {
            visible: vec![item(1, 100.0, 20.0, namespaces::CONTROL_CENTER, "Clock")],
            hidden: Vec::new(),
            always_hidden: Vec::new(),
        });

        let error = plan_item_move(
            &cache,
            MenuBarItemInfo::clock(),
            MoveDestination::LeftOfItem(MenuBarItemInfo::hidden_control_item()),
        )
        .unwrap_err();

        assert_eq!(
            error,
            MovePlanError::ItemNotMovable(MenuBarItemInfo::clock())
        );
    }

    #[test]
    fn move_plan_rejects_non_hideable_items_moved_to_hidden_sections() {
        let cache = SectionItemCache::new(SectionPartition {
            visible: vec![item(
                1,
                100.0,
                20.0,
                namespaces::CONTROL_CENTER,
                "AudioVideoModule",
            )],
            hidden: Vec::new(),
            always_hidden: Vec::new(),
        });

        let error = plan_item_move(
            &cache,
            MenuBarItemInfo::audio_video_module(),
            MoveDestination::LeftOfItem(MenuBarItemInfo::hidden_control_item()),
        )
        .unwrap_err();

        assert_eq!(
            error,
            MovePlanError::ItemCannotBeHidden(MenuBarItemInfo::audio_video_module())
        );
    }
}

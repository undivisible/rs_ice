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
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuBarItemSnapshot {
    pub window_id: u32,
    pub frame: Rect,
    pub info: MenuBarItemInfo,
    pub owner_name: Option<String>,
}

impl MenuBarItemSnapshot {
    pub fn new(window_id: u32, frame: Rect, info: MenuBarItemInfo) -> Self {
        Self {
            window_id,
            frame,
            info,
            owner_name: None,
        }
    }

    pub fn is_hidden_control_item(&self) -> bool {
        self.info == MenuBarItemInfo::hidden_control_item()
    }

    pub fn is_always_hidden_control_item(&self) -> bool {
        self.info == MenuBarItemInfo::always_hidden_control_item()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionPartition {
    pub visible: Vec<MenuBarItemSnapshot>,
    pub hidden: Vec<MenuBarItemSnapshot>,
    pub always_hidden: Vec<MenuBarItemSnapshot>,
}

pub fn partition_sections(mut items: Vec<MenuBarItemSnapshot>) -> SectionPartition {
    items.sort_by(|left, right| {
        left.frame
            .min_x
            .partial_cmp(&right.frame.min_x)
            .unwrap_or(Ordering::Equal)
    });

    let hidden_divider_x = items
        .iter()
        .find(|item| item.is_hidden_control_item())
        .map(|item| item.frame.min_x);
    let always_hidden_divider_x = items
        .iter()
        .find(|item| item.is_always_hidden_control_item())
        .map(|item| item.frame.min_x);

    let mut visible = Vec::new();
    let mut hidden = Vec::new();
    let mut always_hidden = Vec::new();

    for item in items
        .into_iter()
        .filter(|item| !item.is_hidden_control_item() && !item.is_always_hidden_control_item())
    {
        match (hidden_divider_x, always_hidden_divider_x) {
            (Some(_), Some(always_hidden_x)) if item.frame.max_x() > always_hidden_x => {
                always_hidden.push(item);
            }
            (Some(hidden_x), _) if item.frame.max_x() > hidden_x => {
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

pub mod namespaces {
    pub const ICE: &str = "dev.undivisible.rs_ice";
    pub const CONTROL_CENTER: &str = "com.apple.controlcenter";
    pub const SYSTEM_UI_SERVER: &str = "com.apple.systemuiserver";
}

pub mod control_items {
    pub const ICE_ICON: &str = "SItem";
    pub const HIDDEN: &str = "HItem";
    pub const ALWAYS_HIDDEN: &str = "AHItem";
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
            item(1, 10.0, 20.0, "com.example.visible", "Visible"),
            MenuBarItemSnapshot::new(
                2,
                Rect {
                    min_x: 40.0,
                    min_y: 0.0,
                    width: 1.0,
                    height: 24.0,
                },
                MenuBarItemInfo::hidden_control_item(),
            ),
            item(3, 50.0, 20.0, "com.example.hidden", "Hidden"),
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
            item(1, 10.0, 20.0, "com.example.visible", "Visible"),
            MenuBarItemSnapshot::new(
                2,
                Rect {
                    min_x: 40.0,
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
                    min_x: 80.0,
                    min_y: 0.0,
                    width: 1.0,
                    height: 24.0,
                },
                MenuBarItemInfo::always_hidden_control_item(),
            ),
            item(5, 90.0, 20.0, "com.example.always-hidden", "AlwaysHidden"),
        ]);

        assert_eq!(partition.visible.len(), 1);
        assert_eq!(partition.hidden.len(), 1);
        assert_eq!(partition.always_hidden.len(), 1);
        assert_eq!(partition.always_hidden[0].window_id, 5);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKind {
    Accessibility,
    ScreenRecording,
}

impl PermissionKind {
    pub const ALL: [Self; 2] = [Self::Accessibility, Self::ScreenRecording];

    pub fn title(self) -> &'static str {
        match self {
            Self::Accessibility => "Accessibility",
            Self::ScreenRecording => "Screen Recording",
        }
    }

    pub fn is_required(self) -> bool {
        match self {
            Self::Accessibility => true,
            Self::ScreenRecording => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionStatus {
    pub kind: PermissionKind,
    pub granted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionSnapshot {
    pub accessibility: PermissionStatus,
    pub screen_recording: PermissionStatus,
}

impl PermissionSnapshot {
    pub fn from_checker(checker: &impl PermissionChecker) -> Self {
        Self {
            accessibility: PermissionStatus {
                kind: PermissionKind::Accessibility,
                granted: checker.has_accessibility_permission(),
            },
            screen_recording: PermissionStatus {
                kind: PermissionKind::ScreenRecording,
                granted: checker.has_screen_recording_permission(),
            },
        }
    }

    pub fn state(self) -> PermissionsState {
        if self.accessibility.granted && self.screen_recording.granted {
            PermissionsState::HasAllPermissions
        } else if self.accessibility.granted {
            PermissionsState::HasRequiredPermissions
        } else {
            PermissionsState::MissingPermissions
        }
    }
}

impl Default for PermissionSnapshot {
    fn default() -> Self {
        Self {
            accessibility: PermissionStatus {
                kind: PermissionKind::Accessibility,
                granted: false,
            },
            screen_recording: PermissionStatus {
                kind: PermissionKind::ScreenRecording,
                granted: false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionsState {
    MissingPermissions,
    HasRequiredPermissions,
    HasAllPermissions,
}

pub trait PermissionChecker {
    fn has_accessibility_permission(&self) -> bool;
    fn has_screen_recording_permission(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakePermissionChecker {
        accessibility: bool,
        screen_recording: bool,
    }

    impl PermissionChecker for FakePermissionChecker {
        fn has_accessibility_permission(&self) -> bool {
            self.accessibility
        }

        fn has_screen_recording_permission(&self) -> bool {
            self.screen_recording
        }
    }

    #[test]
    fn missing_accessibility_means_missing_required_permissions() {
        let snapshot = PermissionSnapshot::from_checker(&FakePermissionChecker {
            accessibility: false,
            screen_recording: true,
        });

        assert_eq!(snapshot.state(), PermissionsState::MissingPermissions);
    }

    #[test]
    fn accessibility_without_screen_recording_is_enough_to_run() {
        let snapshot = PermissionSnapshot::from_checker(&FakePermissionChecker {
            accessibility: true,
            screen_recording: false,
        });

        assert_eq!(snapshot.state(), PermissionsState::HasRequiredPermissions);
    }

    #[test]
    fn all_granted_means_full_permissions() {
        let snapshot = PermissionSnapshot::from_checker(&FakePermissionChecker {
            accessibility: true,
            screen_recording: true,
        });

        assert_eq!(snapshot.state(), PermissionsState::HasAllPermissions);
    }
}

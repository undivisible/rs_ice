import SwiftUI
import Aurorality

@main
struct RsIceSettingsHostApp: App {
    @State private var state = AurorState()
    @State private var bridge = AurorBridge()

    var body: some Scene {
        WindowGroup {
            AurorRootView(state: state)
                .environment(bridge)
                .task { loadSettingsView() }
        }
        .aurorBridge(bridge)
    }

    private func loadSettingsView() {
        let template = Bundle.module.url(forResource: "settings", withExtension: "crepus")
            .flatMap { try? String(contentsOf: $0, encoding: .utf8) }
            ?? ""
        try? state.load(template: template, context: [
            "showIceIcon": .string(boolLabel("ShowIceIcon", defaultValue: true)),
            "showOnClick": .string(boolLabel("ShowOnClick", defaultValue: true)),
            "autoRehide": .string(boolLabel("AutoRehide", defaultValue: true)),
            "rehideStrategy": .string(rehideStrategyLabel()),
            "rehideInterval": .string(rehideIntervalLabel()),
            "contextMenuOnRightClick": .string(boolLabel("ShowContextMenuOnRightClick", defaultValue: true)),
        ])
    }

    private func boolLabel(_ key: String, defaultValue: Bool) -> String {
        let defaults = UserDefaults.standard
        let value = defaults.object(forKey: key) as? Bool ?? defaultValue
        return value ? "On" : "Off"
    }

    private func rehideStrategyLabel() -> String {
        switch UserDefaults.standard.object(forKey: "RehideStrategy") as? Int ?? 0 {
        case 1:
            return "Timed"
        case 2:
            return "Focused App"
        default:
            return "Smart"
        }
    }

    private func rehideIntervalLabel() -> String {
        let value = UserDefaults.standard.object(forKey: "RehideInterval") as? Double ?? 15.0
        return "\(Int(value)) seconds"
    }
}

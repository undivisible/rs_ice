import SwiftUI
import Aurorality

#if SWIFT_PACKAGE
private func resourceURL(_ name: String, _ ext: String) -> URL? {
    Bundle.module.url(forResource: name, withExtension: ext)
}
#else
private func resourceURL(_ name: String, _ ext: String) -> URL? {
    Bundle.main.url(forResource: name, withExtension: ext)
}
#endif

@main
struct RsIceSettingsHostApp: App {
    @State private var state = AurorState()
    @State private var bridge = {
        let bridge = AurorBridge()
        bridge.register(RsIceSettingsPlugin())
        return bridge
    }()

    var body: some Scene {
        WindowGroup("Ice Settings") {
            AurorRootView(state: state)
                .environment(bridge)
                .frame(minWidth: 420, minHeight: 520)
                .task { load() }
        }
    }

    private func load() {
        let snapshot = (try? bridge.invokeData(
            pluginId: "rsIceSettings",
            method: "snapshot",
            as: RsIceSettingsSnapshot.self
        )) ?? .defaults

        let template = resourceURL("settings", "crepus")
            .flatMap { try? String(contentsOf: $0, encoding: .utf8) }
            ?? "span\n  \"Ice Settings\""

        try? state.load(template: template, context: snapshot.context)
    }
}

struct RsIceSettingsSnapshot: Decodable {
    let showIceIcon: Bool
    let showOnClick: Bool
    let autoRehide: Bool
    let rehideStrategy: String
    let rehideInterval: String
    let contextMenuOnRightClick: Bool

    static let defaults = RsIceSettingsSnapshot(
        showIceIcon: true,
        showOnClick: true,
        autoRehide: true,
        rehideStrategy: "Smart",
        rehideInterval: "15 seconds",
        contextMenuOnRightClick: true
    )

    var context: [String: ContextValue] {
        [
            "showIceIcon": .string(showIceIcon ? "On" : "Off"),
            "showOnClick": .string(showOnClick ? "On" : "Off"),
            "autoRehide": .string(autoRehide ? "On" : "Off"),
            "rehideStrategy": .string(rehideStrategy),
            "rehideInterval": .string(rehideInterval),
            "contextMenuOnRightClick": .string(contextMenuOnRightClick ? "On" : "Off"),
        ]
    }
}

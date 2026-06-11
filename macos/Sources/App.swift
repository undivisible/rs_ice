import AppKit
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
    @NSApplicationDelegateAdaptor(RsIceMenuBarAppDelegate.self) private var appDelegate

    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}

@MainActor
final class RsIceMenuBarAppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem?
    private let popover = NSPopover()
    private let state = AurorState()
    private let bridge = {
        let bridge = AurorBridge()
        bridge.register(RsIceSettingsPlugin())
        return bridge
    }()

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        configureStatusItem()
        configurePopover()
        load()
    }

    private func configureStatusItem() {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        item.autosaveName = "rs_ice.settingsHost"

        if let button = item.button {
            button.image = NSImage(
                systemSymbolName: "snowflake",
                accessibilityDescription: "Ice"
            )
            button.target = self
            button.action = #selector(togglePopover(_:))
        }

        statusItem = item
    }

    private func configurePopover() {
        popover.behavior = .transient
        popover.contentSize = NSSize(width: 420, height: 560)
        popover.contentViewController = NSHostingController(
            rootView: AurorRootView(state: state)
                .environment(bridge)
                .frame(width: 420, height: 560)
        )
    }

    @objc private func togglePopover(_ sender: AnyObject?) {
        guard let button = statusItem?.button else {
            return
        }

        load()

        if popover.isShown {
            popover.performClose(sender)
        } else {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            popover.contentViewController?.view.window?.makeKey()
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

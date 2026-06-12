import AppKit
import ApplicationServices
import CoreGraphics
import SwiftUI

@main
struct RsIceSettingsHostApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate

    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}

@MainActor
private final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem?
    private let popover = NSPopover()
    private var defaultsObserver: NSObjectProtocol?
    private var eventMonitor: Any?
    private var hoverTimer: Timer?
    private var hoverStartedAt: Date?
    private var hiddenSectionShown = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        requestAccessibilityPermissionIfNeeded()
        requestScreenRecordingPermissionIfNeeded()
        configurePopover()
        configureStatusItem()
        configureEventMonitor()
        configureHoverTimer()
        defaultsObserver = NotificationCenter.default.addObserver(
            forName: UserDefaults.didChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor in
                self?.updateStatusButton()
            }
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
        if let defaultsObserver {
            NotificationCenter.default.removeObserver(defaultsObserver)
        }
        if let eventMonitor {
            NSEvent.removeMonitor(eventMonitor)
        }
        hoverTimer?.invalidate()
    }

    private func configureStatusItem() {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        item.autosaveName = "rs_ice.visibleStatusItem"

        if let button = item.button {
            button.target = self
            button.action = #selector(statusItemPressed(_:))
            button.sendAction(on: [.leftMouseDown])
        }

        statusItem = item
        updateStatusButton()
    }

    private func updateStatusButton() {
        let defaults = UserDefaults.standard
        let showIceIcon = defaults.object(forKey: "ShowIceIcon") as? Bool ?? true

        if let button = statusItem?.button {
            let symbolName = hiddenSectionShown ? "snowflake.circle.fill" : "snowflake"
            button.image = showIceIcon ? NSImage(systemSymbolName: symbolName, accessibilityDescription: "Ice") : nil
            button.imagePosition = .imageLeading
            button.title = showIceIcon ? "" : (hiddenSectionShown ? "Ice Shown" : "Ice")
            button.contentTintColor = hiddenSectionShown ? .controlAccentColor : nil
        }
    }

    private func configurePopover() {
        popover.behavior = .transient
        popover.contentSize = NSSize(width: 420, height: 480)
        popover.contentViewController = NSHostingController(
            rootView: IcePanel()
                .frame(width: 420, height: 480)
        )
    }

    private func configureHoverTimer() {
        hoverTimer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.handleHoverTimer()
            }
        }
    }

    private func configureEventMonitor() {
        eventMonitor = NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] event in
            guard let self else {
                return event
            }

            guard self.eventIsInsideStatusItem(event) else {
                return event
            }

            if event.type == .rightMouseDown || event.modifierFlags.contains(.control) {
                self.togglePopover(nil)
            } else {
                self.toggleHiddenSection()
            }
            return nil
        }
    }

    private func eventIsInsideStatusItem(_ event: NSEvent) -> Bool {
        guard let button = statusItem?.button, let window = button.window, event.window === window else {
            return false
        }

        return button.bounds.contains(button.convert(event.locationInWindow, from: nil))
    }

    private func handleHoverTimer() {
        let defaults = UserDefaults.standard
        let showOnHover = defaults.object(forKey: "ShowOnHover") as? Bool ?? false
        guard showOnHover, let button = statusItem?.button, let window = button.window else {
            hoverStartedAt = nil
            return
        }

        let mouse = NSEvent.mouseLocation
        guard window.frame.contains(mouse) else {
            hoverStartedAt = nil
            return
        }

        if hiddenSectionShown {
            hoverStartedAt = nil
            return
        }

        let startedAt = hoverStartedAt ?? Date()
        hoverStartedAt = startedAt
        let delay = defaults.object(forKey: "ShowOnHoverDelay") as? Double ?? 0.2
        if Date().timeIntervalSince(startedAt) >= delay {
            setHiddenSectionShown(true)
            hoverStartedAt = nil
        }
    }

    @objc private func statusItemPressed(_ sender: AnyObject?) {
        guard let event = NSApp.currentEvent else {
            toggleHiddenSection()
            return
        }

        if event.type == .rightMouseUp || event.modifierFlags.contains(.control) {
            togglePopover(sender)
        } else {
            toggleHiddenSection()
        }
    }

    private func togglePopover(_ sender: AnyObject?) {
        guard let button = statusItem?.button else {
            return
        }

        if popover.isShown {
            popover.performClose(sender)
        } else {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            popover.contentViewController?.view.window?.makeKey()
        }
    }

    private func toggleHiddenSection() {
        let defaults = UserDefaults.standard
        let showOnClick = defaults.object(forKey: "ShowOnClick") as? Bool ?? true
        guard showOnClick else {
            return
        }

        if !AXIsProcessTrusted() {
            requestAccessibilityPermissionIfNeeded()
        }

        setHiddenSectionShown(!hiddenSectionShown)
    }

    private func setHiddenSectionShown(_ shown: Bool) {
        hiddenSectionShown = shown
        moveDiscoveredMenuBarItems(hidden: shown)
        updateStatusButton()
    }

    private func requestAccessibilityPermissionIfNeeded() {
        guard !AXIsProcessTrusted() else {
            return
        }

        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
        AXIsProcessTrustedWithOptions(options)
    }

    private func requestScreenRecordingPermissionIfNeeded() {
        guard !CGPreflightScreenCaptureAccess() else {
            return
        }

        CGRequestScreenCaptureAccess()
    }

    private func moveDiscoveredMenuBarItems(hidden: Bool) {
        guard AXIsProcessTrusted(), CGPreflightScreenCaptureAccess() else {
            requestAccessibilityPermissionIfNeeded()
            requestScreenRecordingPermissionIfNeeded()
            return
        }

        guard let iceFrame = statusItem?.button?.window?.frame else {
            return
        }

        let candidates = discoveredMenuBarItemWindows(iceFrame: iceFrame)
        guard !candidates.isEmpty else {
            NSLog("rs_ice: no movable menu bar item windows discovered")
            return
        }

        for candidate in candidates {
            moveWindow(candidate, around: iceFrame, hidden: hidden)
        }
    }

    private func discoveredMenuBarItemWindows(iceFrame: CGRect) -> [MenuBarWindow] {
        guard let rawWindows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] else {
            return []
        }

        let ownPID = ProcessInfo.processInfo.processIdentifier
        let screenFrame = NSScreen.main?.frame ?? .zero
        let iceMidX = iceFrame.midX

        return rawWindows.compactMap { info in
            guard
                let number = info[kCGWindowNumber as String] as? CGWindowID,
                let ownerPID = info[kCGWindowOwnerPID as String] as? pid_t,
                ownerPID != ownPID,
                let boundsInfo = info[kCGWindowBounds as String] as? [String: CGFloat],
                let x = boundsInfo["X"],
                let y = boundsInfo["Y"],
                let width = boundsInfo["Width"],
                let height = boundsInfo["Height"]
            else {
                return nil
            }

            guard width > 1, width < 220, height > 1, height <= 80, y <= 80 else {
                return nil
            }

            let frame = CGRect(x: x, y: screenFrame.height - y - height, width: width, height: height)
            guard abs(frame.midX - iceMidX) > 2 else {
                return nil
            }

            return MenuBarWindow(windowID: number, ownerPID: ownerPID, frame: frame)
        }
        .sorted { $0.frame.minX < $1.frame.minX }
    }

    private func moveWindow(_ window: MenuBarWindow, around iceFrame: CGRect, hidden: Bool) {
        guard let source = CGEventSource(stateID: .hidSystemState) else {
            return
        }

        let start = CGPoint(x: window.frame.midX, y: window.frame.midY)
        let targetX = hidden ? iceFrame.minX - 8 : iceFrame.maxX + 8
        let end = CGPoint(x: targetX, y: iceFrame.midY)

        guard
            let down = CGEvent(mouseEventSource: source, mouseType: .leftMouseDown, mouseCursorPosition: start, mouseButton: .left),
            let up = CGEvent(mouseEventSource: source, mouseType: .leftMouseUp, mouseCursorPosition: end, mouseButton: .left)
        else {
            return
        }

        down.flags = .maskCommand
        down.setIntegerValueField(.mouseEventWindowUnderMousePointer, value: Int64(window.windowID))
        down.setIntegerValueField(.mouseEventWindowUnderMousePointerThatCanHandleThisEvent, value: Int64(window.windowID))
        down.setIntegerValueField(CGEventField(rawValue: 0x33)!, value: Int64(window.windowID))
        up.setIntegerValueField(.mouseEventWindowUnderMousePointer, value: Int64(window.windowID))
        up.setIntegerValueField(.mouseEventWindowUnderMousePointerThatCanHandleThisEvent, value: Int64(window.windowID))
        up.setIntegerValueField(CGEventField(rawValue: 0x33)!, value: Int64(window.windowID))

        down.post(tap: .cgSessionEventTap)
        up.post(tap: .cgSessionEventTap)
    }
}

private struct MenuBarWindow {
    let windowID: CGWindowID
    let ownerPID: pid_t
    let frame: CGRect
}

private enum SettingsPane: String, CaseIterable, Identifiable {
    case general = "General"
    case iceBar = "Ice Bar"
    case permissions = "Permissions"
    case advanced = "Advanced"

    var id: String { rawValue }
}

private enum RehideStrategy: Int, CaseIterable, Identifiable {
    case smart = 0
    case timed = 1
    case focusedApp = 2

    var id: Int { rawValue }

    var title: String {
        switch self {
        case .smart: "Smart"
        case .timed: "Timed"
        case .focusedApp: "Focused App"
        }
    }
}

private enum IceBarLocation: Int, CaseIterable, Identifiable {
    case dynamic = 0
    case mousePointer = 1
    case iceIcon = 2

    var id: Int { rawValue }

    var title: String {
        switch self {
        case .dynamic: "Dynamic"
        case .mousePointer: "Mouse pointer"
        case .iceIcon: "Ice icon"
        }
    }
}

private struct IcePanel: View {
    @AppStorage("ShowIceIcon") private var showIceIcon = true
    @AppStorage("CustomIceIconIsTemplate") private var customIceIconIsTemplate = false
    @AppStorage("UseIceBar") private var useIceBar = false
    @AppStorage("IceBarLocation") private var iceBarLocationRaw = IceBarLocation.dynamic.rawValue
    @AppStorage("ShowOnClick") private var showOnClick = true
    @AppStorage("ShowOnHover") private var showOnHover = false
    @AppStorage("ShowOnScroll") private var showOnScroll = true
    @AppStorage("ItemSpacingOffset") private var itemSpacingOffset = 0.0
    @AppStorage("AutoRehide") private var autoRehide = true
    @AppStorage("RehideStrategy") private var rehideStrategyRaw = RehideStrategy.smart.rawValue
    @AppStorage("RehideInterval") private var rehideInterval = 15.0
    @AppStorage("HideApplicationMenus") private var hideApplicationMenus = true
    @AppStorage("ShowSectionDividers") private var showSectionDividers = false
    @AppStorage("EnableAlwaysHiddenSection") private var enableAlwaysHiddenSection = false
    @AppStorage("CanToggleAlwaysHiddenSection") private var canToggleAlwaysHiddenSection = true
    @AppStorage("ShowOnHoverDelay") private var showOnHoverDelay = 0.2
    @AppStorage("TempShowInterval") private var tempShowInterval = 15.0
    @AppStorage("ShowAllSectionsOnUserDrag") private var showAllSectionsOnUserDrag = true
    @AppStorage("ShowContextMenuOnRightClick") private var contextMenuOnRightClick = true
    @State private var selectedPane = SettingsPane.general
    @State private var accessibilityGranted = AXIsProcessTrusted()
    @State private var screenRecordingGranted = CGPreflightScreenCaptureAccess()

    private var rehideStrategy: Binding<RehideStrategy> {
        Binding {
            RehideStrategy(rawValue: rehideStrategyRaw) ?? .smart
        } set: { strategy in
            rehideStrategyRaw = strategy.rawValue
        }
    }

    private var iceBarLocation: Binding<IceBarLocation> {
        Binding {
            IceBarLocation(rawValue: iceBarLocationRaw) ?? .dynamic
        } set: { location in
            iceBarLocationRaw = location.rawValue
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            Picker("", selection: $selectedPane) {
                ForEach(SettingsPane.allCases) { pane in
                    Text(pane.rawValue).tag(pane)
                }
            }
            .pickerStyle(.segmented)
            .labelsHidden()
            .padding(12)
            Divider()
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    switch selectedPane {
                    case .general:
                        generalPane
                    case .iceBar:
                        iceBarPane
                    case .permissions:
                        permissionsPane
                    case .advanced:
                        advancedPane
                    }
                }
                .padding(14)
            }
            Divider()
            footer
        }
        .background(.regularMaterial)
        .onAppear(perform: refreshPermissions)
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: "snowflake")
                .font(.system(size: 18, weight: .semibold))
                .frame(width: 28, height: 28)
                .background(.quaternary, in: RoundedRectangle(cornerRadius: 6))

            VStack(alignment: .leading, spacing: 1) {
                Text("Ice")
                    .font(.headline)
                Text("Menu Bar Settings")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack {
                Spacer()
                Button {
                    NSApplication.shared.terminate(nil)
                } label: {
                    Image(systemName: "power")
                }
                .buttonStyle(.borderless)
                .help("Quit Ice")
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 12)
    }

    private var generalPane: some View {
        VStack(alignment: .leading, spacing: 12) {
            GroupBox("Visibility") {
                VStack(alignment: .leading, spacing: 10) {
                    Toggle("Show Ice icon", isOn: $showIceIcon)
                    Toggle("Custom Ice icons render as templates", isOn: $customIceIconIsTemplate)
                    Toggle("Show hidden section on empty menu bar click", isOn: $showOnClick)
                    Toggle("Show hidden section on hover", isOn: $showOnHover)
                    Toggle("Show or hide with menu bar scroll", isOn: $showOnScroll)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }

            GroupBox("Rehide") {
                VStack(alignment: .leading, spacing: 10) {
                    Toggle("Auto rehide", isOn: $autoRehide)
                    Picker("Strategy", selection: rehideStrategy) {
                        ForEach(RehideStrategy.allCases) { strategy in
                            Text(strategy.title).tag(strategy)
                        }
                    }
                    Picker("Interval", selection: $rehideInterval) {
                        Text("5 seconds").tag(5.0)
                        Text("10 seconds").tag(10.0)
                        Text("15 seconds").tag(15.0)
                        Text("30 seconds").tag(30.0)
                        Text("60 seconds").tag(60.0)
                    }
                    .disabled(!autoRehide)
                    Picker("Temporary show", selection: $tempShowInterval) {
                        Text("5 seconds").tag(5.0)
                        Text("10 seconds").tag(10.0)
                        Text("15 seconds").tag(15.0)
                        Text("30 seconds").tag(30.0)
                        Text("60 seconds").tag(60.0)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }

            GroupBox("Spacing") {
                Picker("Item spacing offset", selection: $itemSpacingOffset) {
                    Text("-2 pt").tag(-2.0)
                    Text("-1 pt").tag(-1.0)
                    Text("0 pt").tag(0.0)
                    Text("1 pt").tag(1.0)
                    Text("2 pt").tag(2.0)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private var iceBarPane: some View {
        VStack(alignment: .leading, spacing: 12) {
            GroupBox("Ice Bar") {
                VStack(alignment: .leading, spacing: 10) {
                    Toggle("Use Ice Bar", isOn: $useIceBar)
                    Picker("Location", selection: iceBarLocation) {
                        ForEach(IceBarLocation.allCases) { location in
                            Text(location.title).tag(location)
                        }
                    }
                    .disabled(!useIceBar)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private var permissionsPane: some View {
        VStack(alignment: .leading, spacing: 12) {
            GroupBox("Required") {
                permissionRow(
                    title: "Accessibility",
                    granted: accessibilityGranted,
                    actionTitle: "Open Settings"
                ) {
                    openSystemSettings("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                    refreshPermissions()
                }
            }

            GroupBox("Optional") {
                permissionRow(
                    title: "Screen Recording",
                    granted: screenRecordingGranted,
                    actionTitle: "Request Access"
                ) {
                    CGRequestScreenCaptureAccess()
                    openSystemSettings("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                    refreshPermissions()
                }
            }

            Button("Refresh") {
                refreshPermissions()
            }
            .frame(maxWidth: .infinity, alignment: .trailing)
        }
    }

    private var advancedPane: some View {
        VStack(alignment: .leading, spacing: 12) {
            GroupBox("Application Menus") {
                Toggle("Hide application menus when needed", isOn: $hideApplicationMenus)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            GroupBox("Sections") {
                VStack(alignment: .leading, spacing: 10) {
                    Toggle("Show section dividers", isOn: $showSectionDividers)
                    Toggle("Enable always-hidden section", isOn: $enableAlwaysHiddenSection)
                    Toggle("Always-hidden section can be shown", isOn: $canToggleAlwaysHiddenSection)
                        .disabled(!enableAlwaysHiddenSection)
                    Toggle("Show all sections while dragging", isOn: $showAllSectionsOnUserDrag)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }

            GroupBox("Input") {
                VStack(alignment: .leading, spacing: 10) {
                    Picker("Hover delay", selection: $showOnHoverDelay) {
                        Text("0.0 seconds").tag(0.0)
                        Text("0.2 seconds").tag(0.2)
                        Text("0.5 seconds").tag(0.5)
                        Text("1.0 seconds").tag(1.0)
                    }
                    .disabled(!showOnHover)
                    Toggle("Show context menu on right click", isOn: $contextMenuOnRightClick)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private var footer: some View {
        HStack {
            Text("v0.1.0")
                .font(.caption)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 9)
    }

    private func permissionRow(
        title: String,
        granted: Bool,
        actionTitle: String,
        action: @escaping () -> Void
    ) -> some View {
        HStack {
            Label(title, systemImage: granted ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                .foregroundStyle(granted ? .green : .orange)
            Spacer()
            Button(actionTitle, action: action)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func refreshPermissions() {
        accessibilityGranted = AXIsProcessTrusted()
        screenRecordingGranted = CGPreflightScreenCaptureAccess()
    }

    private func openSystemSettings(_ value: String) {
        guard let url = URL(string: value) else {
            return
        }
        NSWorkspace.shared.open(url)
    }
}

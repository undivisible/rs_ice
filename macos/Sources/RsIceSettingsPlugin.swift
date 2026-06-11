import Foundation
import Aurorality

final class RsIceSettingsPlugin: AurorPlugin {
    let id = "rsIceSettings"

    func invoke(method: String, payload: String) throws -> String {
        switch method {
        case "snapshot":
            return envelope(data: snapshot())
        default:
            throw AurorPluginError("unknown method: \(method)")
        }
    }

    private func snapshot() -> [String: Any] {
        let defaults = UserDefaults.standard
        let strategy = defaults.object(forKey: "RehideStrategy") as? Int ?? 0
        let interval = defaults.object(forKey: "RehideInterval") as? Double ?? 15

        return [
            "showIceIcon": bool(defaults, "ShowIceIcon", fallback: true),
            "showOnClick": bool(defaults, "ShowOnClick", fallback: true),
            "autoRehide": bool(defaults, "AutoRehide", fallback: true),
            "rehideStrategy": strategyTitle(strategy),
            "rehideInterval": "\(Int(interval)) seconds",
            "contextMenuOnRightClick": bool(defaults, "ShowContextMenuOnRightClick", fallback: true),
        ]
    }

    private func bool(_ defaults: UserDefaults, _ key: String, fallback: Bool) -> Bool {
        if defaults.object(forKey: key) == nil {
            return fallback
        }
        return defaults.bool(forKey: key)
    }

    private func strategyTitle(_ rawValue: Int) -> String {
        switch rawValue {
        case 1: return "Timed"
        case 2: return "Focused App"
        default: return "Smart"
        }
    }

    private func envelope(data: Any) -> String {
        encode([
            "ok": true,
            "data": data,
        ])
    }

    private func encode(_ value: Any) -> String {
        guard let data = try? JSONSerialization.data(withJSONObject: value),
              let json = String(data: data, encoding: .utf8)
        else { return "{\"ok\":false,\"error\":\"encoding failed\"}" }
        return json
    }
}

import Foundation

enum Fmt {
    static func uptime(_ hours: Double) -> String {
        let totalMinutes = Int(hours * 60)
        let h = totalMinutes / 60
        let m = totalMinutes % 60
        return h > 0 ? "\(h)h \(m)m" : "\(m)m"
    }

    static func count(_ n: Int) -> String {
        if n >= 1_000_000 { return String(format: "%.1fM", Double(n) / 1_000_000) }
        if n >= 1_000 { return String(format: "%.1fK", Double(n) / 1_000) }
        return "\(n)"
    }

    static func pct(_ value: Double) -> String {
        String(format: "%.1f%%", value * 100)
    }

    static func pctRaw(_ value: Double) -> String {
        String(format: "%.1f%%", value)
    }

    static func decimal(_ value: Double, places: Int = 2) -> String {
        String(format: "%.\(places)f", value)
    }

    static func duration(_ seconds: Double) -> String {
        let s = Int(seconds)
        if s >= 3600 { return "\(s / 3600)h \((s % 3600) / 60)m" }
        if s >= 60 { return "\(s / 60)m \(s % 60)s" }
        return "\(s)s"
    }

    static func scientific(_ value: Double) -> String {
        String(format: "%.1e", value)
    }
}

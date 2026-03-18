import SwiftUI

extension Color {
    static let stsBg = Color(hex: 0x0d1117)
    static let stsCard = Color(hex: 0x161b22)
    static let stsBorder = Color(hex: 0x30363d)
    static let stsBorderDim = Color(hex: 0x21262d)
    static let stsText = Color(hex: 0xc9d1d9)
    static let stsTextDim = Color(hex: 0x8b949e)
    static let stsTextMuted = Color(hex: 0x484f58)
    static let stsAccent = Color(hex: 0x00ff41)
    static let stsRed = Color(hex: 0xf85149)
    static let stsOrange = Color(hex: 0xf0883e)
    static let stsYellow = Color(hex: 0xd29922)
    static let stsBlue = Color(hex: 0x58a6ff)
    static let stsGold = Color(hex: 0xd2a038)
    static let stsGreen = Color(hex: 0x3fb950)

    static let stanceCalm = Color(hex: 0x58a6ff)
    static let stanceWrath = Color(hex: 0xf85149)
    static let stanceDivinity = Color(hex: 0xd2a038)
    static let stanceNeutral = Color(hex: 0x484f58)

    init(hex: UInt, opacity: Double = 1.0) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255,
            opacity: opacity
        )
    }
}

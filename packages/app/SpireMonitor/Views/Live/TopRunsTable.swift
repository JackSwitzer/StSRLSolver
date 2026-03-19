import SwiftUI

struct TopRunsTable: View {
    let episodes: [Episode]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Top Runs")

            if episodes.isEmpty {
                Text("No episodes yet")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 100)
            } else {
                Grid(alignment: .leading, horizontalSpacing: 16, verticalSpacing: 6) {
                    // Header
                    GridRow {
                        Text("#").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Floor").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Config").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Seed").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Killer").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("HP").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Time").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("When").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                    }

                    Divider().gridCellColumns(8).background(Color.stsBorderDim)

                    ForEach(Array(episodes.enumerated()), id: \.element.id) { rank, ep in
                        GridRow {
                            Text("\(rank + 1)")
                                .font(.stsBody)
                                .foregroundStyle(Color.stsTextDim)

                            Text("\(ep.effectiveFloor)")
                                .font(.stsValue)
                                .foregroundStyle(floorColor(ep.effectiveFloor))

                            Text(ep.configName ?? "-")
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsBlue)
                                .lineLimit(1)

                            Text(ep.seed.prefix(10))
                                .font(.stsBody)
                                .foregroundStyle(Color.stsTextDim)

                            Text(ep.won ? "WIN" : (ep.deathEnemy ?? "-"))
                                .font(.stsBody)
                                .foregroundStyle(ep.won ? Color.stsAccent : Color.stsRed)
                                .lineLimit(1)

                            Text(ep.won ? "\(ep.effectiveHp)" : "\(ep.effectiveHp)/\(ep.effectiveMaxHp)")
                                .font(.stsBody)
                                .foregroundStyle(Color.stsText)

                            Text(Fmt.duration(ep.effectiveDuration))
                                .font(.stsBody)
                                .foregroundStyle(Color.stsTextDim)

                            Text(shortTimestamp(ep.timestamp))
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsTextMuted)
                        }
                    }
                }
            }
        }
    }

    private func floorColor(_ floor: Int) -> Color {
        if floor >= 51 { return Color.stsAccent }
        if floor >= 34 { return Color.stsBlue }
        if floor >= 17 { return Color.stsYellow }
        return Color.stsText
    }

    /// Parse ISO timestamp and show compact relative or HH:mm format
    private func shortTimestamp(_ ts: String?) -> String {
        guard let ts = ts else { return "-" }
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        // Try with fractional seconds first, then without
        guard let date = formatter.date(from: ts) ?? {
            formatter.formatOptions = [.withInternetDateTime]
            return formatter.date(from: ts)
        }() else { return ts.prefix(5).description }

        let elapsed = Date().timeIntervalSince(date)
        if elapsed < 60 { return "now" }
        if elapsed < 3600 { return "\(Int(elapsed / 60))m ago" }
        if elapsed < 86400 { return "\(Int(elapsed / 3600))h ago" }

        let df = DateFormatter()
        df.dateFormat = "MM/dd HH:mm"
        return df.string(from: date)
    }
}

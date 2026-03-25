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
                        Text("Seed").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Killer").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("HP").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Time").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                    }

                    Divider().gridCellColumns(6).background(Color.stsBorderDim)

                    ForEach(Array(episodes.enumerated()), id: \.element.id) { rank, ep in
                        GridRow {
                            Text("\(rank + 1)")
                                .font(.stsBody)
                                .foregroundStyle(Color.stsTextDim)

                            Text("\(ep.effectiveFloor)")
                                .font(.stsValue)
                                .foregroundStyle(floorColor(ep.effectiveFloor))

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
}

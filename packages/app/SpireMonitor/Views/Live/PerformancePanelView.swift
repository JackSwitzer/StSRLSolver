import SwiftUI

struct PerformancePanelView: View {
    let performance: [RoomCategory: RoomPerformance]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Combat Performance")

            if performance.isEmpty {
                Text("No combat data yet")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 100)
            } else {
                Grid(alignment: .leading, horizontalSpacing: 16, verticalSpacing: 8) {
                    // Header
                    GridRow {
                        Text("Type").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Count").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Avg Turns").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Avg HP Lost").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                        Text("Potion %").font(.stsLabel).foregroundStyle(Color.stsTextDim)
                    }

                    Divider().gridCellColumns(5).background(Color.stsBorderDim)

                    ForEach(RoomCategory.allCases, id: \.self) { cat in
                        if let perf = performance[cat] {
                            GridRow {
                                Text(cat.rawValue)
                                    .font(.stsBody)
                                    .foregroundStyle(roomColor(cat))

                                Text("\(perf.count)")
                                    .font(.stsBody)
                                    .foregroundStyle(Color.stsText)

                                Text(Fmt.decimal(perf.avgTurns, places: 1))
                                    .font(.stsBody)
                                    .foregroundStyle(Color.stsText)

                                Text(Fmt.decimal(perf.avgHpLost, places: 1))
                                    .font(.stsBody)
                                    .foregroundStyle(perf.avgHpLost > 20 ? Color.stsRed : Color.stsText)

                                Text(Fmt.pct(perf.potionRate))
                                    .font(.stsBody)
                                    .foregroundStyle(Color.stsText)
                            }
                        }
                    }
                }
            }
        }
    }

    private func roomColor(_ cat: RoomCategory) -> Color {
        switch cat {
        case .monster: return Color.stsText
        case .elite: return Color.stsOrange
        case .boss: return Color.stsRed
        }
    }
}

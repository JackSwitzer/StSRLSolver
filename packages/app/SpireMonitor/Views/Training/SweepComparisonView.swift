import SwiftUI

struct SweepComparisonView: View {
    let configs: [String: ConfigStats]

    private var sorted: [ConfigStats] {
        configs.values.sorted { $0.avgFloor > $1.avgFloor }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Sweep Comparison")

            if configs.isEmpty {
                Text("No config data yet...")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
            } else {
                ForEach(sorted) { config in
                    HStack {
                        Circle()
                            .fill(config.isActive ? Color.stsAccent : Color.stsTextMuted)
                            .frame(width: 8, height: 8)

                        Text(config.name)
                            .font(.system(size: 12, weight: config.isActive ? .bold : .regular, design: .monospaced))
                            .foregroundStyle(config.isActive ? Color.stsAccent : Color.stsText)
                            .frame(width: 120, alignment: .leading)

                        metricLabel("Floor", value: String(format: "%.1f", config.avgFloor))
                        metricLabel("Peak", value: "\(config.peakFloor)")
                        metricLabel("Games", value: "\(config.games)")

                        if let loss = config.lastLoss {
                            metricLabel("Loss", value: String(format: "%.3f", loss))
                        }

                        if let phase = config.phase {
                            Text(phase)
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsTextDim)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.stsBorderDim)
                                .clipShape(RoundedRectangle(cornerRadius: 3))
                        }

                        Spacer()
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private func metricLabel(_ label: String, value: String) -> some View {
        VStack(spacing: 1) {
            Text(value)
                .font(.system(size: 12, weight: .medium, design: .monospaced))
                .foregroundStyle(Color.stsText)
            Text(label)
                .font(.system(size: 9))
                .foregroundStyle(Color.stsTextDim)
        }
        .frame(width: 60)
    }
}

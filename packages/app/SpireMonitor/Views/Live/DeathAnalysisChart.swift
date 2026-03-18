import SwiftUI
import Charts

struct DeathAnalysisChart: View {
    let deaths: [(enemy: String, count: Int)]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Deaths")

            if deaths.isEmpty {
                Text("No death data yet")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 100)
            } else {
                // Simple list format: enemy name + count
                ForEach(Array(deaths.enumerated()), id: \.offset) { index, item in
                    HStack(spacing: 8) {
                        Text(item.enemy)
                            .font(.stsBody)
                            .foregroundStyle(index == 0 ? Color.stsRed : Color.stsText)
                            .lineLimit(1)

                        Spacer()

                        // Bar
                        GeometryReader { geo in
                            let maxCount = Double(deaths.first?.count ?? 1)
                            let width = geo.size.width * (Double(item.count) / maxCount)
                            RoundedRectangle(cornerRadius: 2)
                                .fill(index == 0 ? Color.stsRed.opacity(0.6) : Color.stsAccent.opacity(0.4))
                                .frame(width: max(width, 2))
                        }
                        .frame(width: 80, height: 8)

                        Text("\(item.count)")
                            .font(.stsValue)
                            .foregroundStyle(Color.stsTextDim)
                            .frame(width: 30, alignment: .trailing)
                    }
                }
            }
        }
    }
}

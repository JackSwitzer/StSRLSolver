import SwiftUI

struct StatusBarView: View {
    let status: TrainingStatus?
    let isLive: Bool

    var body: some View {
        HStack(spacing: 0) {
            metric("Games", value: Fmt.count(status?.totalGames ?? 0))
            divider
            metric("Avg Floor", value: Fmt.decimal(status?.avgFloor100 ?? 0, places: 1))
            divider
            metric("Win Rate", value: Fmt.pctRaw(status?.winRate100 ?? 0))
            divider
            metric("G/min", value: Fmt.decimal(status?.gamesPerMin ?? 0, places: 1))
            divider
            metric("Steps", value: Fmt.count(status?.trainSteps ?? 0))
            divider
            metric("Entropy", value: Fmt.decimal(status?.entropy ?? 0, places: 3))
            divider
            metric("Loss", value: Fmt.scientific(status?.totalLoss ?? 0))
            divider
            metric("Uptime", value: Fmt.uptime(status?.elapsedHours ?? 0))
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color.stsCard)
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.stsBorder, lineWidth: 1))
    }

    private func metric(_ label: String, value: String) -> some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
            Text(value)
                .font(.stsValue)
                .foregroundStyle(Color.stsText)
        }
        .frame(maxWidth: .infinity)
    }

    private var divider: some View {
        Rectangle()
            .fill(Color.stsBorderDim)
            .frame(width: 1, height: 30)
    }
}

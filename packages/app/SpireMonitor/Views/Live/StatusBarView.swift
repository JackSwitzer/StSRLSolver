import SwiftUI

struct StatusBarView: View {
    let status: TrainingStatus?
    let isLive: Bool
    let peakFloor: Int
    let mode: StatsMode

    private var avgFloor: String {
        if mode == .allTime, let s = status, let total = s.totalGames, total > 0 {
            // Approximate from recent_floors when all-time isn't available
            return Fmt.decimal(s.avgFloor100 ?? 0, places: 1)
        }
        return Fmt.decimal(status?.avgFloor100 ?? 0, places: 1)
    }

    private var winRate: String {
        if mode == .allTime, let s = status, let games = s.totalGames, games > 0 {
            let rate = Double(s.totalWins ?? 0) / Double(games) * 100
            return Fmt.decimal(rate, places: 1) + "%"
        }
        return Fmt.pctRaw(status?.winRate100 ?? 0)
    }

    var body: some View {
        HStack(spacing: 0) {
            metric("Games", value: Fmt.count(status?.totalGames ?? 0))
            divider
            metric("Avg Floor", value: avgFloor)
            divider
            metric("Peak", value: "\(peakFloor)", highlight: peakFloor >= 15)
            divider
            metric("Win Rate", value: winRate)
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

    private func metric(_ label: String, value: String, highlight: Bool = false) -> some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
            Text(value)
                .font(.stsValue)
                .foregroundStyle(highlight ? Color.stsAccent : Color.stsText)
        }
        .frame(maxWidth: .infinity)
    }

    private var divider: some View {
        Rectangle()
            .fill(Color.stsBorderDim)
            .frame(width: 1, height: 30)
    }
}

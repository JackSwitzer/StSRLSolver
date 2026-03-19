import SwiftUI

struct CardPickSummaryView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Card Pick Summary")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)

            Text("Card pick data appears after games are played with the new logging.")
                .font(.stsBody)
                .foregroundStyle(Color.stsTextDim)

            // Placeholder: will populate from card_picks data in episodes
            if appState.store.recentEpisodes.isEmpty {
                Text("No episodes loaded yet.")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .padding(.vertical, 20)
            } else {
                Text("\(appState.store.recentEpisodes.count) recent episodes")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextDim)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color.stsCard)
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}

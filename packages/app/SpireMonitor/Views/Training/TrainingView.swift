import SwiftUI

struct TrainingView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                // Sweep comparison
                SweepComparisonView(configs: appState.store.configHistory)
                    .sectionCard()
                    .padding(.horizontal, 16)

                // Top row: key metrics
                if let status = appState.store.status {
                    HStack(spacing: 12) {
                        MetricCard(title: "Train Steps", value: "\(status.trainSteps ?? 0)")
                        MetricCard(title: "EV", value: String(format: "%.3f", status.explainedVariance ?? 0))
                        MetricCard(title: "KL Div", value: String(format: "%.4f", status.klDivergence ?? 0))
                        MetricCard(title: "Mean Value", value: String(format: "%.2f", status.meanValue ?? 0))
                    }
                    .padding(.horizontal, 16)
                }

                // Charts
                DiagnosticsChartsView()
                    .padding(.horizontal, 16)

                // Card picks summary
                CardPickSummaryView()
                    .padding(.horizontal, 16)
            }
            .padding(.vertical, 16)
        }
        .background(Color.stsBg)
    }
}

struct MetricCard: View {
    let title: String
    let value: String

    var body: some View {
        VStack(spacing: 4) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.stsTitle)
                .foregroundStyle(Color.stsText)
        }
        .frame(maxWidth: .infinity)
        .padding(12)
        .background(Color.stsCard)
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}

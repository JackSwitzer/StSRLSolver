import SwiftUI

struct HyperparamGridView: View {
    let status: TrainingStatus?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Hyperparameters")

            let columns = Array(repeating: GridItem(.flexible(), spacing: 12), count: 3)
            LazyVGrid(columns: columns, spacing: 8) {
                paramCell("Config", value: status?.configName ?? "-")
                paramCell("Phase", value: status?.sweepPhase ?? "-")
                paramCell("Entropy", value: Fmt.decimal(status?.entropy ?? 0, places: 4))
                paramCell("Ent Coeff", value: Fmt.decimal(status?.entropyCoeff ?? 0, places: 4))
                paramCell("Buffer", value: Fmt.count(status?.bufferSize ?? 0))
                paramCell("Steps", value: Fmt.count(status?.trainSteps ?? 0))
                paramCell("Clip Frac", value: Fmt.decimal(status?.clipFraction ?? 0, places: 3))
                paramCell("Peak Floor", value: "\(status?.peakFloor ?? 0)")
                paramCell("Games", value: Fmt.count(status?.totalGames ?? 0))
            }
        }
    }

    private func paramCell(_ label: String, value: String) -> some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
            Text(value)
                .font(.stsBody)
                .fontWeight(.medium)
                .foregroundStyle(Color.stsText)
                .lineLimit(1)
        }
        .padding(6)
        .frame(maxWidth: .infinity)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

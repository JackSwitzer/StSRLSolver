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
                paramCell("Entropy", value: fmtOpt(status?.entropy, places: 3))
                paramCell("Ent Coeff", value: fmtOpt(status?.entropyCoeff, places: 4))
                paramCell("Buffer", value: fmtOptInt(status?.bufferSize))
                paramCell("Steps", value: fmtOptInt(status?.trainSteps))
                paramCell("Loss", value: fmtOpt(status?.totalLoss, places: 4))
                paramCell("Peak Floor", value: fmtOptInt(status?.peakFloor ?? status?.replayBestFloor))
                paramCell("Games", value: fmtOptInt(status?.totalGames))
            }
        }
    }

    private func fmtOpt(_ val: Double?, places: Int) -> String {
        guard let v = val else { return "-" }
        return Fmt.decimal(v, places: places)
    }

    private func fmtOptInt(_ val: Int?) -> String {
        guard let v = val else { return "-" }
        return Fmt.count(v)
    }

    private func paramCell(_ label: String, value: String) -> some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
            Text(value)
                .font(.stsBody)
                .fontWeight(.medium)
                .foregroundStyle(value == "-" ? Color.stsTextMuted : Color.stsText)
                .lineLimit(1)
        }
        .padding(6)
        .frame(maxWidth: .infinity)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

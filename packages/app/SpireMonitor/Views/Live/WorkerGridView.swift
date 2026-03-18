import SwiftUI

private let allSlots = [
    "Vengeance", "Fury", "Zen", "Vigilante", "Serenity", "Tempest",
    "Oracle", "Ascendant", "Sentinel", "Harmony", "Specter", "Eclipse",
]

struct WorkerGridView: View {
    let workers: [WorkerStatus]

    private let columns = Array(repeating: GridItem(.flexible(), spacing: 8), count: 4)

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            SectionHeader(title: "Workers (\(workers.count)/12)")

            LazyVGrid(columns: columns, spacing: 8) {
                ForEach(allSlots, id: \.self) { name in
                    if let worker = workers.first(where: { $0.name == name }) {
                        WorkerCard(worker: worker)
                    } else {
                        EmptyWorkerSlot(name: name)
                    }
                }
            }
        }
    }
}

private struct EmptyWorkerSlot: View {
    let name: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(name)
                .font(.stsBody)
                .foregroundStyle(Color.stsTextMuted)
            RoundedRectangle(cornerRadius: 2)
                .fill(Color.stsBorderDim)
                .frame(height: 4)
            Text("idle")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
        }
        .padding(8)
        .background(Color.stsBg.opacity(0.5))
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.stsBorderDim.opacity(0.5), lineWidth: 1))
    }
}

private struct WorkerCard: View {
    let worker: WorkerStatus

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(worker.name)
                    .font(.stsBody)
                    .fontWeight(.medium)
                    .foregroundStyle(Color.stsText)

                Spacer()

                Circle()
                    .fill(stanceColor(worker.phase))
                    .frame(width: 6, height: 6)
            }

            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.stsBorderDim)
                    RoundedRectangle(cornerRadius: 2)
                        .fill(hpColor(worker.hpFraction))
                        .frame(width: geo.size.width * worker.hpFraction)
                }
            }
            .frame(height: 4)

            HStack {
                Text("F\(worker.floor ?? 0)")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextDim)

                Spacer()

                Text(worker.enemy ?? "-")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextDim)
                    .lineLimit(1)
            }
        }
        .padding(8)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.stsBorderDim, lineWidth: 1))
    }

    private func stanceColor(_ phase: String?) -> Color {
        switch phase?.lowercased() {
        case "wrath": return .stanceWrath
        case "calm": return .stanceCalm
        case "divinity": return .stanceDivinity
        default: return .stanceNeutral
        }
    }

    private func hpColor(_ fraction: Double) -> Color {
        if fraction > 0.6 { return Color.stsGreen }
        if fraction > 0.3 { return Color.stsYellow }
        return Color.stsRed
    }
}

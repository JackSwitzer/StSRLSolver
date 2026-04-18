import SwiftUI

struct RecordedRunReplayView: View {
    @Environment(AppState.self) private var appState

    private var report: RecordedRunReplayReportArtifact? {
        appState.store.recordedRunReplay
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let report {
                    headerCard(report)
                    perCombatTable(report)
                } else {
                    emptyState
                }
            }
            .padding(16)
        }
        .background(Color.stsBg)
    }

    private func headerCard(_ report: RecordedRunReplayReportArtifact) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("RECORDED RUN REPLAY")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            HStack(spacing: 24) {
                summaryStat("Play ID", String(report.playID.prefix(12)))
                summaryStat("Seed", report.seedPlayed)
                summaryStat("Combats", "\(report.totalCombats)")
                summaryStat("Solved", "\(report.solved)", color: .stsAccent)
                summaryStat("Failed", "\(report.failed)", color: report.failed > 0 ? .stsRed : .stsTextMuted)
                summaryStat("Unsupported", "\(report.unsupported)", color: .stsYellow)
                summaryStat("Errors", "\(report.error)", color: report.error > 0 ? .stsRed : .stsTextMuted)
            }
        }
        .padding(12)
        .background(Color.stsCard)
        .cornerRadius(8)
    }

    private func perCombatTable(_ report: RecordedRunReplayReportArtifact) -> some View {
        VStack(spacing: 0) {
            tableHeader
            Divider().background(Color.stsBorder)
            ForEach(report.results) { row in
                combatRow(row)
                Divider().background(Color.stsBorder)
            }
        }
        .background(Color.stsCard)
        .cornerRadius(8)
    }

    private var tableHeader: some View {
        HStack(alignment: .top, spacing: 0) {
            Text("FLOOR")
                .frame(width: 56, alignment: .leading)
            Text("ENCOUNTER")
                .frame(width: 200, alignment: .leading)
            Text("ROOM")
                .frame(width: 70, alignment: .leading)
            VStack(alignment: .leading, spacing: 2) {
                Text("HUMAN")
                    .foregroundStyle(Color.stsAccent)
                Text("hp loss · turns · entry hp")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            VStack(alignment: .leading, spacing: 2) {
                Text("SOLVER")
                    .foregroundStyle(Color.stsYellow)
                Text("hp loss · status · visits · stop")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Text("DECK / RELICS / POTIONS")
                .frame(width: 220, alignment: .leading)
        }
        .font(.stsLabel)
        .foregroundStyle(Color.stsTextMuted)
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
    }

    private func combatRow(_ row: RecordedRunCombatResultArtifact) -> some View {
        HStack(alignment: .top, spacing: 0) {
            Text("F\(row.floor)")
                .frame(width: 56, alignment: .leading)
                .foregroundStyle(Color.stsText)
            Text(row.encounter)
                .frame(width: 200, alignment: .leading)
                .foregroundStyle(Color.stsText)
            Text(row.roomKind)
                .frame(width: 70, alignment: .leading)
                .foregroundStyle(Color.stsTextMuted)
            humanCell(row)
                .frame(maxWidth: .infinity, alignment: .leading)
            solverCell(row)
                .frame(maxWidth: .infinity, alignment: .leading)
            deckCell(row)
                .frame(width: 220, alignment: .leading)
        }
        .font(.stsBody)
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    private func humanCell(_ row: RecordedRunCombatResultArtifact) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 8) {
                Text("\(row.recordedHPLoss) HP")
                    .foregroundStyle(row.recordedHPLoss == 0 ? Color.stsAccent : Color.stsText)
                if let turns = row.recordedTurns {
                    Text("\(turns)t")
                        .foregroundStyle(Color.stsTextMuted)
                }
                Text("\(row.entryHP)/\(row.maxHP)")
                    .foregroundStyle(Color.stsTextMuted)
                    .font(.stsLabel)
            }
        }
    }

    private func solverCell(_ row: RecordedRunCombatResultArtifact) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 8) {
                if let loss = row.solverHPLoss {
                    Text(String(format: "%.1f HP", loss))
                        .foregroundStyle(loss <= Double(row.recordedHPLoss) ? Color.stsAccent : Color.stsYellow)
                } else {
                    Text("—")
                        .foregroundStyle(Color.stsTextMuted)
                }
                statusBadge(row.status)
                if let visits = row.searchVisits {
                    Text("\(visits) v")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsTextMuted)
                }
                if let stop = row.stopReason {
                    Text(stop)
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsTextMuted)
                }
            }
            if let err = row.error {
                Text(err)
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)
                    .lineLimit(1)
            }
        }
    }

    private func deckCell(_ row: RecordedRunCombatResultArtifact) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("\(row.entryDeckSize) cards")
                .foregroundStyle(Color.stsText)
            Text("\(row.entryRelics.count) relics")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            if !row.entryPotions.isEmpty {
                Text("\(row.entryPotions.count) potions: \(row.entryPotions.joined(separator: ", "))")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)
                    .lineLimit(1)
            }
        }
    }

    private func statusBadge(_ status: String) -> some View {
        let color: Color = {
            switch status {
            case "solved": return .stsAccent
            case "failed": return .stsRed
            case "unsupported": return .stsYellow
            case "error": return .stsRed
            default: return .stsTextMuted
            }
        }()
        return Text(status.uppercased())
            .font(.stsLabel)
            .foregroundStyle(color)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .overlay(
                RoundedRectangle(cornerRadius: 3)
                    .stroke(color, lineWidth: 1)
            )
    }

    private func summaryStat(_ label: String, _ value: String, color: Color = .stsText) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.stsBody)
                .foregroundStyle(color)
        }
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Text("No recorded_run_replay_report.json yet")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text("Run `./scripts/training.sh validate-recorded-run --run-file <path> --output-dir logs/active` to populate this view.")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
                .multilineTextAlignment(.center)
        }
        .padding(40)
        .frame(maxWidth: .infinity)
        .background(Color.stsCard)
        .cornerRadius(8)
    }
}

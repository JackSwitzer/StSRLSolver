import SwiftUI

struct FrontierInspectorView: View {
    @Environment(AppState.self) private var appState

    private var episodes: [LocatedEpisodeLog] { appState.store.artifactEpisodes }
    private var selectedEpisode: LocatedEpisodeLog? { appState.selectedArtifactEpisode ?? episodes.first }

    private var selectedStep: ArtifactEpisodeStep? {
        guard let selectedEpisode else { return nil }
        let steps = selectedEpisode.episode.frontierSteps
        return steps.first(where: { $0.stepIndex == appState.selectedArtifactStepIndex }) ?? steps.first
    }

    var body: some View {
        HSplitView {
            episodeList
            frontierStepList
            chosenAlternativesPanel
        }
        .background(Color.stsBg)
        .onAppear {
            if appState.selectedArtifactEpisode == nil, let first = episodes.first {
                appState.selectedArtifactEpisode = first
                appState.selectedArtifactStepIndex = first.episode.frontierSteps.first?.stepIndex ?? 0
            }
        }
    }

    private var episodeList: some View {
        VStack(alignment: .leading, spacing: 0) {
            SectionHeader(title: "Frontier Episodes")
                .padding(12)

            if episodes.isEmpty {
                emptyColumn(
                    title: "No artifact episodes",
                    subtitle: "Waiting for rebuilt episodes.jsonl entries with search frontier lines."
                )
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(episodes) { located in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(located.episode.displayName)
                                    .font(.stsBody)
                                    .foregroundStyle(Color.stsText)
                                Text("\(located.episode.frontierSteps.count) frontier steps")
                                    .font(.stsLabel)
                                    .foregroundStyle(Color.stsTextDim)
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(10)
                            .background(
                                appState.selectedArtifactEpisode?.id == located.id
                                    ? Color.stsAccent.opacity(0.12)
                                    : Color.clear
                            )
                            .contentShape(Rectangle())
                            .onTapGesture {
                                appState.selectedArtifactEpisode = located
                                appState.selectedArtifactStepIndex = located.episode.frontierSteps.first?.stepIndex ?? 0
                            }

                            Divider().background(Color.stsBorderDim)
                        }
                    }
                }
            }
        }
        .frame(minWidth: 240, idealWidth: 280, maxWidth: 320)
    }

    private var frontierStepList: some View {
        VStack(alignment: .leading, spacing: 0) {
            SectionHeader(title: "Chosen vs Alternatives")
                .padding(12)

            if let selectedEpisode {
                let steps = selectedEpisode.episode.frontierSteps
                if steps.isEmpty {
                    emptyColumn(
                        title: "No frontier-bearing steps",
                        subtitle: "This episode log exists, but no step captured a search frontier yet."
                    )
                } else {
                    ScrollView {
                        LazyVStack(spacing: 8) {
                            ForEach(steps) { step in
                                FrontierStepRow(
                                    step: step,
                                    isSelected: step.stepIndex == selectedStep?.stepIndex
                                )
                                .onTapGesture {
                                    appState.selectedArtifactStepIndex = step.stepIndex
                                }
                            }
                        }
                        .padding(12)
                    }
                }
            } else {
                emptyColumn(
                    title: "Select an episode",
                    subtitle: "Choose an artifact episode to inspect its frontier lines."
                )
            }
        }
        .frame(minWidth: 260, idealWidth: 320)
    }

    private var chosenAlternativesPanel: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                SectionHeader(title: "Frontier Inspector")

                if let step = selectedStep, let frontier = step.searchFrontier {
                    chosenHeader(step: step)
                    if let value = step.value {
                        rolloutValue(value)
                    }
                    frontierLines(frontier: frontier, chosenActionID: step.actionID)
                } else if let frontier = appState.store.frontierReport {
                    frontierRankingFallback(frontier)
                } else {
                    emptyColumn(
                        title: "No frontier data selected",
                        subtitle: "Pick a frontier-bearing episode step, or wait for frontier_report.json."
                    )
                }
            }
            .padding(16)
        }
        .frame(minWidth: 420)
    }

    private func chosenHeader(step: ArtifactEpisodeStep) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Step \(step.stepIndex)")
                .font(.stsTitle)
                .foregroundStyle(Color.stsText)
            HStack(spacing: 12) {
                pill("Chosen action", "\(step.actionID)", color: .stsAccent)
                pill("Reward Δ", Fmt.decimal(step.rewardDelta, places: 3), color: .stsBlue)
                if step.done {
                    pill("Terminal", "done", color: .stsYellow)
                }
            }
        }
        .sectionCard()
    }

    private func rolloutValue(_ outcome: CombatOutcomeArtifact) -> some View {
        HStack(spacing: 12) {
            valueMetric("Solve", Fmt.pct(outcome.solveProbability))
            valueMetric("HP Loss", Fmt.decimal(outcome.expectedHPLoss))
            valueMetric("Turns", Fmt.decimal(outcome.expectedTurns))
            valueMetric("Potion", Fmt.decimal(outcome.potionCost))
        }
        .sectionCard()
    }

    private func frontierLines(frontier: CombatFrontierSummaryArtifact, chosenActionID: Int) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Alternative lines")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)

            ForEach(sortedLines(frontier.lines, chosenActionID: chosenActionID)) { line in
                let isChosen = line.actionPrefix.first == chosenActionID
                VStack(alignment: .leading, spacing: 6) {
                    HStack {
                        Text(isChosen ? "Chosen" : "Alternative")
                            .font(.system(size: 10, weight: .bold, design: .monospaced))
                            .foregroundStyle(isChosen ? Color.stsAccent : Color.stsTextMuted)
                        Text("line \(line.lineIndex)")
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                        Spacer()
                        Text("prefix \(line.actionPrefix.map(String.init).joined(separator: ","))")
                            .font(.system(size: 11, weight: .regular, design: .monospaced))
                            .foregroundStyle(Color.stsTextDim)
                    }

                    HStack(spacing: 12) {
                        valueMetric("Solve", Fmt.pct(line.outcome.solveProbability))
                        valueMetric("HP", Fmt.decimal(line.outcome.expectedHPLoss))
                        valueMetric("Turns", Fmt.decimal(line.outcome.expectedTurns))
                        valueMetric("Potion", Fmt.decimal(line.outcome.potionCost))
                        valueMetric("Visits", "\(line.visits)")
                        valueMetric("ms", "\(line.elapsedMS)")
                    }
                }
                .padding(10)
                .background(isChosen ? Color.stsAccent.opacity(0.10) : Color.stsBg)
                .clipShape(RoundedRectangle(cornerRadius: 6))
            }
        }
        .sectionCard()
    }

    private func frontierRankingFallback(_ frontier: FrontierReportArtifact) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("No per-step frontier line is selected yet")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text("Showing the run-level frontier ranking from frontier_report.json instead.")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)

            ForEach(Array(frontier.ranking.prefix(6).enumerated()), id: \.offset) { index, label in
                HStack {
                    Text("#\(index + 1)")
                        .font(.system(size: 11, weight: .bold, design: .monospaced))
                        .foregroundStyle(Color.stsAccent)
                    Text(label)
                        .font(.stsBody)
                        .foregroundStyle(Color.stsText)
                    Spacer()
                    if frontier.frontier.contains(label) {
                        Text("FRONTIER")
                            .font(.system(size: 10, weight: .bold, design: .monospaced))
                            .foregroundStyle(Color.stsYellow)
                    }
                }
            }
        }
        .sectionCard()
    }

    private func sortedLines(_ lines: [CombatFrontierLineArtifact], chosenActionID: Int) -> [CombatFrontierLineArtifact] {
        lines.sorted { lhs, rhs in
            let lhsChosen = lhs.actionPrefix.first == chosenActionID
            let rhsChosen = rhs.actionPrefix.first == chosenActionID
            if lhsChosen != rhsChosen { return lhsChosen }
            if lhs.outcome.solveProbability != rhs.outcome.solveProbability {
                return lhs.outcome.solveProbability > rhs.outcome.solveProbability
            }
            if lhs.outcome.expectedHPLoss != rhs.outcome.expectedHPLoss {
                return lhs.outcome.expectedHPLoss < rhs.outcome.expectedHPLoss
            }
            return lhs.lineIndex < rhs.lineIndex
        }
    }

    private func valueMetric(_ title: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.system(size: 12, weight: .semibold, design: .monospaced))
                .foregroundStyle(Color.stsText)
        }
    }

    private func pill(_ title: String, _ value: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.system(size: 12, weight: .bold, design: .monospaced))
                .foregroundStyle(color)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 6)
        .background(color.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func emptyColumn(title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text(subtitle)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(12)
    }
}

private struct FrontierStepRow: View {
    let step: ArtifactEpisodeStep
    let isSelected: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            HStack {
                Text("Step \(step.stepIndex)")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsText)
                Spacer()
                Text("action \(step.actionID)")
                    .font(.system(size: 11, weight: .medium, design: .monospaced))
                    .foregroundStyle(Color.stsAccent)
            }

            HStack(spacing: 10) {
                tinyMetric("lines", "\(step.searchFrontier?.lines.count ?? 0)")
                if let value = step.value {
                    tinyMetric("solve", Fmt.pct(value.solveProbability))
                    tinyMetric("hp", Fmt.decimal(value.expectedHPLoss))
                }
                tinyMetric("Δr", Fmt.decimal(step.rewardDelta, places: 3))
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(isSelected ? Color.stsAccent.opacity(0.10) : Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func tinyMetric(_ title: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                .foregroundStyle(Color.stsText)
        }
    }
}

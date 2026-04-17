import SwiftUI

struct ArtifactStreamsView: View {
    let events: [TrainingEventRecord]
    let metrics: [TrainingMetricRecord]

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            eventPanel
            metricPanel
        }
    }

    private var eventPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Event Stream")

            if events.isEmpty {
                emptyPanel("events.jsonl not found yet")
            } else {
                ForEach(Array(events.suffix(8).reversed()), id: \.id) { event in
                    VStack(alignment: .leading, spacing: 3) {
                        Text(event.eventType)
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                        Text(event.timestamp ?? "")
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                        if !event.payload.isEmpty {
                            Text(
                                event.payload
                                    .sorted { $0.key < $1.key }
                                    .prefix(3)
                                    .map { "\($0.key)=\($0.value.displayString)" }
                                    .joined(separator: " · ")
                            )
                            .font(.system(size: 11, weight: .regular, design: .monospaced))
                            .foregroundStyle(Color.stsTextMuted)
                            .lineLimit(2)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(8)
                    .background(Color.stsBg)
                    .clipShape(RoundedRectangle(cornerRadius: 6))
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .sectionCard()
    }

    private var metricPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Metric Stream")

            if metrics.isEmpty {
                emptyPanel("metrics.jsonl not found yet")
            } else {
                ForEach(Array(metrics.suffix(8).reversed()), id: \.id) { metric in
                    HStack {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(metric.name)
                                .font(.stsBody)
                                .foregroundStyle(Color.stsText)
                            Text("step \(metric.step) · \(metric.config)")
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsTextDim)
                        }
                        Spacer()
                        Text(Fmt.decimal(metric.value, places: 4))
                            .font(.system(size: 12, weight: .semibold, design: .monospaced))
                            .foregroundStyle(Color.stsAccent)
                    }
                    .padding(8)
                    .background(Color.stsBg)
                    .clipShape(RoundedRectangle(cornerRadius: 6))
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .sectionCard()
    }

    private func emptyPanel(_ text: String) -> some View {
        Text(text)
            .font(.stsLabel)
            .foregroundStyle(Color.stsTextDim)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(10)
            .background(Color.stsBg)
            .clipShape(RoundedRectangle(cornerRadius: 6))
    }
}

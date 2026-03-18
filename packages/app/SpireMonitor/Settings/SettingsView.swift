import SwiftUI

struct SettingsView: View {
    @Bindable var config: AppConfig

    var body: some View {
        Form {
            Section("Data Directory") {
                LabeledContent("Logs Path") {
                    HStack {
                        Text(config.logsPath.path())
                            .font(.stsBody)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)

                        Button("Choose...") {
                            let panel = NSOpenPanel()
                            panel.canChooseDirectories = true
                            panel.canChooseFiles = false
                            panel.allowsMultipleSelection = false
                            panel.message = "Select the training logs directory"
                            if panel.runModal() == .OK, let url = panel.url {
                                config.setLogsPath(url)
                            }
                        }
                    }
                }

                Button("Reset to Default") {
                    config.resetToDefault()
                }
                .foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
        .frame(width: 500, height: 200)
    }
}

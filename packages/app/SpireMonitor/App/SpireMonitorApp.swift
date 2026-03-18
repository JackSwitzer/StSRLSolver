import SwiftUI

@main
struct SpireMonitorApp: App {
    @State private var appState = AppState()

    var body: some Scene {
        Window("Spire Monitor", id: "main") {
            ContentView()
                .environment(appState)
                .onAppear { appState.startPolling() }
                .onDisappear { appState.stopPolling() }
                .frame(minWidth: 1000, minHeight: 600)
        }
        .defaultSize(width: 1400, height: 900)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }

        Settings {
            SettingsView(config: appState.config)
        }
    }
}

struct ContentView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        VStack(spacing: 0) {
            // Toolbar with segmented control
            HStack {
                Picker("View", selection: Bindable(appState).selectedView) {
                    ForEach(AppView.allCases) { view in
                        Label(view.rawValue, systemImage: view.icon)
                            .tag(view)
                    }
                }
                .pickerStyle(.segmented)
                .frame(maxWidth: 400)

                Spacer()

                // Live indicator
                HStack(spacing: 6) {
                    Circle()
                        .fill(appState.store.isLive ? Color.stsAccent : Color.stsTextMuted)
                        .frame(width: 8, height: 8)
                    Text(appState.store.isLive ? "LIVE" : "STALE")
                        .font(.stsLabel)
                        .foregroundStyle(appState.store.isLive ? Color.stsAccent : Color.stsTextMuted)
                }

                Button(action: { appState.refresh() }) {
                    Image(systemName: "arrow.clockwise")
                }
                .keyboardShortcut("r", modifiers: .command)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .background(Color.stsCard)

            Divider().background(Color.stsBorder)

            // View router
            Group {
                switch appState.selectedView {
                case .live:
                    LiveView()
                case .analysis:
                    PlaceholderView(name: "Analysis")
                case .detail:
                    PlaceholderView(name: "Detail")
                case .replay:
                    PlaceholderView(name: "Replay")
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Color.stsBg)
        .foregroundStyle(Color.stsText)
    }
}

struct PlaceholderView: View {
    let name: String

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "hammer.fill")
                .font(.system(size: 40))
                .foregroundStyle(Color.stsTextMuted)
            Text("\(name) view")
                .font(.stsTitle)
                .foregroundStyle(Color.stsTextDim)
            Text("Coming in a future session")
                .font(.stsBody)
                .foregroundStyle(Color.stsTextMuted)
        }
    }
}

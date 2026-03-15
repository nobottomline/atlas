// Atlas Platform
import SwiftUI
import Core

/// App settings — registry URLs, preferences, diagnostics.
public struct SettingsView: View {

    @AppStorage("registryURL") private var registryURL = "https://raw.githubusercontent.com/atlas-platform/atlas-registry/main"
    @AppStorage("nsfwEnabled") private var nsfwEnabled = false
    @State private var showLogs = false

    public init() {}

    public var body: some View {
        NavigationStack {
            Form {
                Section("Registry") {
                    TextField("Registry URL", text: $registryURL)
                        .textContentType(.URL)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                }

                Section("Content") {
                    Toggle("Allow NSFW Sources", isOn: $nsfwEnabled)
                }

                Section("Diagnostics") {
                    Button("View Source Logs") {
                        showLogs = true
                    }
                }

                Section("About") {
                    LabeledContent("Version", value: "0.1.0")
                    LabeledContent("Runtime", value: "WasmKit (pure Swift)")
                    LabeledContent("SDK", value: "Atlas SDK 0.1.0")
                    Link("Source Code", destination: URL(string: "https://github.com/atlas-platform/atlas")!)
                }
            }
            .navigationTitle("Settings")
            .sheet(isPresented: $showLogs) {
                SourceLogView()
            }
        }
    }
}

// MARK: - Source Log View

struct SourceLogView: View {
    @State private var logs: [String] = []
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Group {
                if logs.isEmpty {
                    ContentUnavailableView(
                        "No Logs",
                        systemImage: "text.alignleft",
                        description: Text("Source logs will appear here during execution.")
                    )
                } else {
                    List(logs, id: \.self) { log in
                        Text(log)
                            .font(.system(.caption, design: .monospaced))
                    }
                }
            }
            .navigationTitle("Source Logs")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }
}

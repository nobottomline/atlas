// Atlas Platform
import SwiftUI
import Core

/// App settings — registry URLs, preferences, diagnostics.
public struct SettingsView: View {

    @AppStorage("registryURL")
    private var registryURL = "https://raw.githubusercontent.com/nobottomline/atlas/main/registry"

    @AppStorage("nsfwEnabled") private var nsfwEnabled = false

    public init() {}

    public var body: some View {
        NavigationStack {
            Form {
                Section("Source Repository") {
                    TextField("Registry URL", text: $registryURL)
                        .textContentType(.URL)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .font(.system(.caption, design: .monospaced))

                    Text("Sources are downloaded from this URL. Change it to use a custom registry.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Section("Content") {
                    Toggle("Allow NSFW Sources", isOn: $nsfwEnabled)
                }

                Section("About") {
                    LabeledContent("Version", value: "0.1.0")
                    LabeledContent("Runtime", value: "WasmKit (pure Swift)")
                    LabeledContent("SDK", value: "Atlas SDK 0.1.0")
                }
            }
            .navigationTitle("Settings")
        }
    }
}

// Atlas Platform
import SwiftUI
import Core
import SourceRegistry
import SourceRuntime

/// Browse, install, and manage source modules.
public struct SourcesView: View {

    @State private var sourceManager: SourceManager
    @State private var searchText = ""
    @State private var showInstalled = false

    public init(sourceManager: SourceManager) {
        self._sourceManager = State(initialValue: sourceManager)
    }

    public var body: some View {
        NavigationStack {
            Group {
                if sourceManager.isLoadingRegistry && sourceManager.registryEntries.isEmpty {
                    ProgressView("Loading sources...")
                } else if filteredEntries.isEmpty {
                    ContentUnavailableView(
                        "No Sources",
                        systemImage: "puzzlepiece.extension",
                        description: Text("Pull to refresh or check your registry URL.")
                    )
                } else {
                    sourceList
                }
            }
            .navigationTitle("Sources")
            .searchable(text: $searchText, prompt: "Search sources")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Picker("Filter", selection: $showInstalled) {
                        Text("All").tag(false)
                        Text("Installed").tag(true)
                    }
                    .pickerStyle(.segmented)
                    .frame(width: 150)
                }
            }
            .refreshable {
                await sourceManager.refreshRegistry()
            }
            .task {
                if sourceManager.registryEntries.isEmpty {
                    await sourceManager.refreshRegistry()
                }
            }
        }
    }

    private var filteredEntries: [RegistryEntry] {
        var entries = sourceManager.registryEntries
        if showInstalled {
            let installedIDs = Set(sourceManager.installedSources.map(\.id))
            entries = entries.filter { installedIDs.contains($0.id) }
        }
        if !searchText.isEmpty {
            entries = entries.filter {
                $0.name.localizedCaseInsensitiveContains(searchText) ||
                $0.id.localizedCaseInsensitiveContains(searchText)
            }
        }
        return entries
    }

    private var sourceList: some View {
        List(filteredEntries) { entry in
            SourceRow(entry: entry, state: sourceManager.state(for: entry)) {
                Task {
                    try? await sourceManager.install(entry: entry)
                }
            } onRemove: {
                Task {
                    try? await sourceManager.remove(sourceId: entry.id)
                }
            }
        }
    }
}

// MARK: - Source Row

private struct SourceRow: View {
    let entry: RegistryEntry
    let state: SourceManager.SourceState
    let onInstall: () -> Void
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            // Icon placeholder
            RoundedRectangle(cornerRadius: 8)
                .fill(.secondary.opacity(0.2))
                .frame(width: 44, height: 44)
                .overlay {
                    Image(systemName: "puzzlepiece.extension")
                        .foregroundStyle(.secondary)
                }

            VStack(alignment: .leading, spacing: 2) {
                HStack {
                    Text(entry.name)
                        .font(.headline)
                    Text("v\(entry.latestVersion)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                if let desc = entry.description {
                    Text(desc)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                HStack(spacing: 4) {
                    Text(entry.lang.uppercased())
                        .font(.caption2)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(.quaternary)
                        .clipShape(RoundedRectangle(cornerRadius: 3))
                    ForEach(entry.tags, id: \.self) { tag in
                        Text(tag)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Spacer()

            actionButton
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private var actionButton: some View {
        switch state {
        case .notInstalled:
            Button("Install", action: onInstall)
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
        case .installed:
            Menu {
                Button("Remove", role: .destructive, action: onRemove)
            } label: {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            }
        case .updateAvailable(let version):
            Button("Update to \(version)", action: onInstall)
                .buttonStyle(.bordered)
                .controlSize(.small)
        }
    }
}

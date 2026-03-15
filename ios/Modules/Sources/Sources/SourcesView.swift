// Atlas Platform
import SwiftUI
import Core
import SourceRegistry
import SourceRuntime
import Reader

/// Browse, install, and manage source modules.
public struct SourcesView: View {

    @State private var sourceManager: SourceManager
    @State private var searchText = ""

    public init(sourceManager: SourceManager) {
        self._sourceManager = State(initialValue: sourceManager)
    }

    public var body: some View {
        NavigationStack {
            Group {
                if sourceManager.isLoadingRegistry
                    && sourceManager.registryEntries.isEmpty
                    && sourceManager.installedSources.isEmpty {
                    ProgressView("Loading sources...")
                } else {
                    mainList
                }
            }
            .navigationTitle("Sources")
            .searchable(text: $searchText, prompt: "Search sources")
            .refreshable {
                await sourceManager.refreshRegistry()
            }
            .task {
                if sourceManager.registryEntries.isEmpty {
                    await sourceManager.refreshRegistry()
                }
            }
            .navigationDestination(for: SourceManager.InstalledSource.self) { source in
                SourceBrowseView(sourceInstance: source.instance, sourceName: source.manifest.name)
                    .navigationDestination(for: MangaNavItem.self) { item in
                        MangaDetailView(mangaId: item.mangaId, sourceInstance: item.sourceInstance)
                    }
            }
        }
    }

    private var mainList: some View {
        List {
            // Installed sources — tap to browse
            if !filteredInstalled.isEmpty {
                Section("Installed") {
                    ForEach(filteredInstalled) { source in
                        NavigationLink(value: source) {
                            InstalledSourceRow(source: source)
                        }
                    }
                }
            }

            // Available sources — install button
            if !filteredAvailable.isEmpty {
                Section("Available") {
                    ForEach(filteredAvailable) { entry in
                        AvailableSourceRow(
                            entry: entry,
                            state: sourceManager.state(for: entry),
                            isInstalling: sourceManager.isInstalling == entry.id
                        ) {
                            Task { try? await sourceManager.install(entry: entry) }
                        } onRemove: {
                            Task { try? await sourceManager.remove(sourceId: entry.id) }
                        }
                    }
                }
            }
        }
    }

    private var filteredInstalled: [SourceManager.InstalledSource] {
        guard !searchText.isEmpty else { return sourceManager.installedSources }
        return sourceManager.installedSources.filter {
            $0.manifest.name.localizedCaseInsensitiveContains(searchText)
        }
    }

    private var filteredAvailable: [RegistryEntry] {
        var entries = sourceManager.registryEntries
        if !searchText.isEmpty {
            entries = entries.filter {
                $0.name.localizedCaseInsensitiveContains(searchText)
            }
        }
        return entries
    }
}

// MARK: - Navigation item for manga detail

public struct MangaNavItem: Hashable {
    public let mangaId: String
    public let sourceInstance: SourceInstance

    public static func == (lhs: MangaNavItem, rhs: MangaNavItem) -> Bool {
        lhs.mangaId == rhs.mangaId
    }
    public func hash(into hasher: inout Hasher) {
        hasher.combine(mangaId)
    }
}

// MARK: - Installed Source Row

private struct InstalledSourceRow: View {
    let source: SourceManager.InstalledSource

    var body: some View {
        HStack(spacing: 12) {
            RoundedRectangle(cornerRadius: 10)
                .fill(.blue.opacity(0.15))
                .frame(width: 44, height: 44)
                .overlay {
                    Image(systemName: "book.pages")
                        .foregroundStyle(.blue)
                }

            VStack(alignment: .leading, spacing: 2) {
                Text(source.manifest.name)
                    .font(.headline)
                HStack(spacing: 4) {
                    Text(source.manifest.lang.uppercased())
                        .font(.caption2)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(.quaternary)
                        .clipShape(RoundedRectangle(cornerRadius: 3))
                    Text("v\(source.manifest.version)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()

            Image(systemName: "chevron.right")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Available Source Row

private struct AvailableSourceRow: View {
    let entry: RegistryEntry
    let state: SourceManager.SourceState
    let isInstalling: Bool
    let onInstall: () -> Void
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            RoundedRectangle(cornerRadius: 10)
                .fill(.secondary.opacity(0.15))
                .frame(width: 44, height: 44)
                .overlay {
                    Image(systemName: "puzzlepiece.extension")
                        .foregroundStyle(.secondary)
                }

            VStack(alignment: .leading, spacing: 2) {
                Text(entry.name)
                    .font(.headline)
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
                    Text("v\(entry.latestVersion)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()

            actionButton
        }
        .padding(.vertical, 2)
    }

    @ViewBuilder
    private var actionButton: some View {
        if isInstalling {
            ProgressView()
                .controlSize(.small)
        } else {
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
                Button("Update \(version)", action: onInstall)
                    .buttonStyle(.bordered)
                    .controlSize(.small)
            }
        }
    }
}

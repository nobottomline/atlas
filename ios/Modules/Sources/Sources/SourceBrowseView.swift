// Atlas Platform
import SwiftUI
import Core
import SourceRuntime

/// Browse manga from an installed source — search, latest, popular.
public struct SourceBrowseView: View {

    let sourceInstance: SourceInstance
    let sourceName: String

    @State private var searchText = ""
    @State private var results: [MangaEntry] = []
    @State private var isLoading = false
    @State private var error: String?
    @State private var selectedTab = 0

    public init(sourceInstance: SourceInstance, sourceName: String) {
        self.sourceInstance = sourceInstance
        self.sourceName = sourceName
    }

    public var body: some View {
        VStack(spacing: 0) {
            // Tab selector: Popular / Latest
            Picker("", selection: $selectedTab) {
                Text("Popular").tag(0)
                Text("Latest").tag(1)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)
            .padding(.vertical, 8)

            Group {
                if isLoading && results.isEmpty {
                    Spacer()
                    ProgressView("Loading manga...")
                    Spacer()
                } else if results.isEmpty && error == nil {
                    Spacer()
                    ContentUnavailableView("No Manga Found", systemImage: "magnifyingglass")
                    Spacer()
                } else if let error {
                    Spacer()
                    ContentUnavailableView("Error", systemImage: "exclamationmark.triangle",
                                           description: Text(error))
                    Spacer()
                } else {
                    mangaGrid
                }
            }
        }
        .navigationTitle(sourceName)
        .searchable(text: $searchText, prompt: "Search \(sourceName)")
        .onSubmit(of: .search) { performSearch() }
        .onChange(of: selectedTab) { loadContent() }
        .task { loadContent() }
    }

    private var mangaGrid: some View {
        ScrollView {
            LazyVGrid(columns: [
                GridItem(.adaptive(minimum: 110, maximum: 160))
            ], spacing: 12) {
                ForEach(results) { entry in
                    NavigationLink(value: MangaNavItem(
                        mangaId: entry.id,
                        sourceInstance: sourceInstance
                    )) {
                        MangaCoverCell(entry: entry)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding()
        }
    }

    private func loadContent() {
        isLoading = true
        error = nil
        let tab = selectedTab
        let source = sourceInstance
        Task.detached {
            do {
                print("[atlas-ui] Loading \(tab == 0 ? "popular" : "latest")...")
                let response = tab == 0
                    ? try source.getPopular()
                    : try source.getLatest()
                print("[atlas-ui] Got \(response.entries.count) entries")
                await MainActor.run {
                    results = response.entries
                    isLoading = false
                }
            } catch {
                print("[atlas-ui] Load error: \(error)")
                await MainActor.run {
                    self.error = "\(error)"
                    isLoading = false
                }
            }
        }
    }

    private func performSearch() {
        guard !searchText.isEmpty else {
            loadContent()
            return
        }
        let text = searchText
        let source = sourceInstance
        isLoading = true
        error = nil
        Task.detached {
            do {
                let query = SearchQuery(title: text)
                let response = try source.search(query: query)
                await MainActor.run {
                    results = response.entries
                    isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    isLoading = false
                }
            }
        }
    }
}

// MARK: - Manga Cover Cell

struct MangaCoverCell: View {
    let entry: MangaEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            AsyncImage(url: entry.coverURL.flatMap(URL.init)) { phase in
                switch phase {
                case .success(let image):
                    image.resizable().aspectRatio(2.0/3.0, contentMode: .fill)
                case .failure:
                    coverPlaceholder
                default:
                    coverPlaceholder.overlay(ProgressView())
                }
            }
            .frame(height: 160)
            .clipShape(RoundedRectangle(cornerRadius: 8))

            Text(entry.title)
                .font(.caption)
                .lineLimit(2)
                .foregroundStyle(.primary)
        }
    }

    private var coverPlaceholder: some View {
        RoundedRectangle(cornerRadius: 8)
            .fill(.secondary.opacity(0.15))
            .frame(height: 160)
            .overlay {
                Image(systemName: "book.closed")
                    .foregroundStyle(.secondary)
            }
    }
}

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

    public init(sourceInstance: SourceInstance, sourceName: String) {
        self.sourceInstance = sourceInstance
        self.sourceName = sourceName
    }

    public var body: some View {
        Group {
            if isLoading && results.isEmpty {
                ProgressView()
            } else if results.isEmpty {
                ContentUnavailableView("No Results", systemImage: "magnifyingglass")
            } else {
                mangaGrid
            }
        }
        .navigationTitle(sourceName)
        .searchable(text: $searchText, prompt: "Search \(sourceName)")
        .onSubmit(of: .search) { performSearch() }
        .task { await loadPopular() }
        .alert("Error", isPresented: .init(
            get: { error != nil },
            set: { if !$0 { error = nil } }
        )) {} message: {
            Text(error ?? "")
        }
    }

    private var mangaGrid: some View {
        ScrollView {
            LazyVGrid(columns: [
                GridItem(.adaptive(minimum: 110, maximum: 160))
            ], spacing: 12) {
                ForEach(results) { entry in
                    NavigationLink(value: entry) {
                        MangaCoverCell(entry: entry)
                    }
                }
            }
            .padding()
        }
    }

    private func performSearch() {
        guard !searchText.isEmpty else { return }
        let text = searchText
        isLoading = true
        Task.detached {
            do {
                let query = SearchQuery(title: text)
                let response = try sourceInstance.search(query: query)
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

    private func loadPopular() async {
        isLoading = true
        Task.detached {
            do {
                let response = try sourceInstance.getPopular()
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
            // Cover image
            AsyncImage(url: entry.coverURL.flatMap(URL.init)) { phase in
                switch phase {
                case .success(let image):
                    image.resizable().aspectRatio(2/3, contentMode: .fill)
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
    }
}

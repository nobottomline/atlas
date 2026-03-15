// Atlas Platform
import SwiftUI
import Core
import SourceRuntime

/// Displays the user's manga library — manga added from installed sources.
public struct LibraryView: View {

    @State private var library: [LibraryItem] = []
    @State private var searchText = ""

    public init() {}

    public var body: some View {
        NavigationStack {
            Group {
                if library.isEmpty {
                    ContentUnavailableView(
                        "Your Library is Empty",
                        systemImage: "books.vertical",
                        description: Text("Browse sources to add manga to your library.")
                    )
                } else {
                    libraryGrid
                }
            }
            .navigationTitle("Library")
            .searchable(text: $searchText, prompt: "Search library")
        }
    }

    private var filteredItems: [LibraryItem] {
        guard !searchText.isEmpty else { return library }
        return library.filter {
            $0.title.localizedCaseInsensitiveContains(searchText)
        }
    }

    private var libraryGrid: some View {
        ScrollView {
            LazyVGrid(columns: [
                GridItem(.adaptive(minimum: 110, maximum: 160))
            ], spacing: 12) {
                ForEach(filteredItems) { item in
                    VStack(alignment: .leading, spacing: 4) {
                        AsyncImage(url: item.coverURL) { phase in
                            switch phase {
                            case .success(let image):
                                image.resizable().aspectRatio(2/3, contentMode: .fill)
                            default:
                                RoundedRectangle(cornerRadius: 8)
                                    .fill(.secondary.opacity(0.15))
                            }
                        }
                        .frame(height: 160)
                        .clipShape(RoundedRectangle(cornerRadius: 8))

                        Text(item.title)
                            .font(.caption)
                            .lineLimit(2)

                        if let unread = item.unreadCount, unread > 0 {
                            Text("\(unread) new")
                                .font(.caption2)
                                .foregroundStyle(.blue)
                        }
                    }
                }
            }
            .padding()
        }
    }
}

// MARK: - Library Item

public struct LibraryItem: Identifiable, Sendable {
    public let id: String
    public let sourceId: String
    public let title: String
    public let coverURL: URL?
    public let unreadCount: Int?

    public init(id: String, sourceId: String, title: String, coverURL: URL?, unreadCount: Int? = nil) {
        self.id = id
        self.sourceId = sourceId
        self.title = title
        self.coverURL = coverURL
        self.unreadCount = unreadCount
    }
}

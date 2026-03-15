// Atlas Platform
import SwiftUI
import Core
import SourceRuntime

/// Displays manga details and chapter list, with navigation to the reader.
public struct MangaDetailView: View {

    let mangaId: String
    let sourceInstance: SourceInstance

    @State private var manga: Manga?
    @State private var chapters: [Chapter] = []
    @State private var isLoading = true
    @State private var error: String?
    @State private var selectedChapter: Chapter?

    public init(mangaId: String, sourceInstance: SourceInstance) {
        self.mangaId = mangaId
        self.sourceInstance = sourceInstance
    }

    public var body: some View {
        Group {
            if isLoading {
                ProgressView()
            } else if let manga {
                detailContent(manga)
            } else {
                ContentUnavailableView("Manga Not Found", systemImage: "book.closed")
            }
        }
        .navigationTitle(manga?.title ?? "Loading...")
        .navigationBarTitleDisplayMode(.inline)
        .task { await loadDetails() }
        .fullScreenCover(item: $selectedChapter) { chapter in
            ChapterReaderWrapper(
                chapter: chapter,
                sourceInstance: sourceInstance
            )
        }
    }

    private func detailContent(_ manga: Manga) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Header
                HStack(alignment: .top, spacing: 12) {
                    AsyncImage(url: manga.coverURL.flatMap(URL.init)) { phase in
                        if case .success(let image) = phase {
                            image.resizable().aspectRatio(2/3, contentMode: .fill)
                        } else {
                            RoundedRectangle(cornerRadius: 8).fill(.secondary.opacity(0.15))
                        }
                    }
                    .frame(width: 120, height: 180)
                    .clipShape(RoundedRectangle(cornerRadius: 8))

                    VStack(alignment: .leading, spacing: 6) {
                        Text(manga.title)
                            .font(.title3.bold())

                        if let author = manga.author {
                            Label(author, systemImage: "person")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }

                        HStack(spacing: 4) {
                            StatusBadge(status: manga.status)
                            Text(manga.contentType.rawValue.capitalized)
                                .font(.caption)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(.quaternary)
                                .clipShape(Capsule())
                        }

                        if !manga.tags.isEmpty {
                            FlowLayout(spacing: 4) {
                                ForEach(manga.tags, id: \.self) { tag in
                                    Text(tag)
                                        .font(.caption2)
                                        .padding(.horizontal, 6)
                                        .padding(.vertical, 2)
                                        .background(.blue.opacity(0.1))
                                        .clipShape(Capsule())
                                }
                            }
                        }
                    }
                }
                .padding(.horizontal)

                // Description
                if let desc = manga.description {
                    Text(desc)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal)
                }

                Divider()

                // Chapter list
                Text("Chapters (\(chapters.count))")
                    .font(.headline)
                    .padding(.horizontal)

                LazyVStack(spacing: 0) {
                    ForEach(chapters) { chapter in
                        Button {
                            selectedChapter = chapter
                        } label: {
                            ChapterRow(chapter: chapter)
                        }
                        Divider().padding(.leading)
                    }
                }
            }
            .padding(.vertical)
        }
    }

    private func loadDetails() async {
        isLoading = true
        Task.detached {
            do {
                let manga = try sourceInstance.getMangaDetails(id: mangaId)
                let chapters = try sourceInstance.getChapters(mangaId: mangaId)
                await MainActor.run {
                    self.manga = manga
                    self.chapters = chapters
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
}

// MARK: - Chapter Row

private struct ChapterRow: View {
    let chapter: Chapter

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 4) {
                    if let num = chapter.number {
                        Text("Ch. \(String(format: "%.1f", num))")
                            .font(.subheadline.bold())
                    }
                    if let title = chapter.title {
                        Text(title)
                            .font(.subheadline)
                            .lineLimit(1)
                    }
                }

                HStack(spacing: 8) {
                    if let scanlator = chapter.scanlator {
                        Text(scanlator)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    if let date = chapter.dateUpdated {
                        Text(date, style: .date)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Spacer()

            Image(systemName: "chevron.right")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
        .padding(.horizontal)
        .padding(.vertical, 10)
        .contentShape(Rectangle())
    }
}

// MARK: - Chapter Reader Wrapper

private struct ChapterReaderWrapper: View {
    let chapter: Chapter
    let sourceInstance: SourceInstance

    @State private var pages: [Page] = []
    @State private var isLoading = true
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        Group {
            if isLoading {
                ZStack {
                    Color.black.ignoresSafeArea()
                    ProgressView().tint(.white)
                }
            } else {
                ReaderView(
                    pages: pages,
                    chapterTitle: chapter.title ?? "Ch. \(String(format: "%.0f", chapter.number ?? 0))"
                )
            }
        }
        .task {
            Task.detached {
                do {
                    let pages = try sourceInstance.getPages(chapterId: chapter.id)
                    await MainActor.run {
                        self.pages = pages
                        self.isLoading = false
                    }
                } catch {
                    await MainActor.run {
                        dismiss()
                    }
                }
            }
        }
    }
}

// MARK: - Status Badge

private struct StatusBadge: View {
    let status: MangaStatus

    var body: some View {
        Text(status.rawValue.capitalized)
            .font(.caption)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.15))
            .foregroundStyle(color)
            .clipShape(Capsule())
    }

    private var color: Color {
        switch status {
        case .ongoing:   return .green
        case .completed: return .blue
        case .hiatus:    return .orange
        case .cancelled: return .red
        case .unknown:   return .secondary
        }
    }
}

// MARK: - Flow Layout

private struct FlowLayout: Layout {
    var spacing: CGFloat = 4

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = arrangeSubviews(proposal: proposal, subviews: subviews)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = arrangeSubviews(proposal: proposal, subviews: subviews)
        for (index, offset) in result.offsets.enumerated() {
            subviews[index].place(
                at: CGPoint(x: bounds.minX + offset.x, y: bounds.minY + offset.y),
                proposal: .unspecified
            )
        }
    }

    private func arrangeSubviews(proposal: ProposedViewSize, subviews: Subviews) -> (offsets: [CGPoint], size: CGSize) {
        let maxWidth = proposal.width ?? .infinity
        var offsets: [CGPoint] = []
        var x: CGFloat = 0
        var y: CGFloat = 0
        var rowHeight: CGFloat = 0
        var maxX: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if x + size.width > maxWidth, x > 0 {
                x = 0
                y += rowHeight + spacing
                rowHeight = 0
            }
            offsets.append(CGPoint(x: x, y: y))
            rowHeight = max(rowHeight, size.height)
            x += size.width + spacing
            maxX = max(maxX, x)
        }

        return (offsets, CGSize(width: maxX, height: y + rowHeight))
    }
}

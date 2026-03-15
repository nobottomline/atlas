// Atlas Platform
import SwiftUI
import Core

/// Manages chapter downloads for offline reading.
public struct DownloadsView: View {

    @State private var downloads: [DownloadItem] = []

    public init() {}

    public var body: some View {
        NavigationStack {
            Group {
                if downloads.isEmpty {
                    ContentUnavailableView(
                        "No Downloads",
                        systemImage: "arrow.down.circle",
                        description: Text("Downloaded chapters will appear here.")
                    )
                } else {
                    downloadList
                }
            }
            .navigationTitle("Downloads")
        }
    }

    private var downloadList: some View {
        List {
            ForEach(downloads) { item in
                HStack {
                    VStack(alignment: .leading) {
                        Text(item.mangaTitle)
                            .font(.headline)
                        Text(item.chapterTitle)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    switch item.status {
                    case .downloading(let progress):
                        ProgressView(value: progress)
                            .frame(width: 60)
                    case .completed:
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    case .failed:
                        Image(systemName: "exclamationmark.circle.fill")
                            .foregroundStyle(.red)
                    case .queued:
                        Image(systemName: "clock")
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .onDelete { offsets in
                downloads.remove(atOffsets: offsets)
            }
        }
    }
}

// MARK: - Download Item

public struct DownloadItem: Identifiable, Sendable {
    public let id: String
    public let mangaTitle: String
    public let chapterTitle: String
    public var status: DownloadStatus

    public enum DownloadStatus: Sendable {
        case queued
        case downloading(progress: Double)
        case completed
        case failed
    }
}

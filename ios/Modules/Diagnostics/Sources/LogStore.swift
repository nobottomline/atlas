// Atlas Platform
import Foundation
import Core

/// Collects and stores log messages from source modules.
@Observable
public final class LogStore: @unchecked Sendable {

    public struct LogEntry: Identifiable, Sendable {
        public let id = UUID()
        public let timestamp: Date
        public let sourceId: String
        public let message: String
    }

    public private(set) var entries: [LogEntry] = []
    private let maxEntries: Int

    public init(maxEntries: Int = 500) {
        self.maxEntries = maxEntries
    }

    /// Append log messages from a source.
    public func append(sourceId: String, messages: [String]) {
        let now = Date()
        let new = messages.map { LogEntry(timestamp: now, sourceId: sourceId, message: $0) }
        entries.append(contentsOf: new)
        if entries.count > maxEntries {
            entries.removeFirst(entries.count - maxEntries)
        }
    }

    /// Clear all logs.
    public func clear() {
        entries.removeAll()
    }
}

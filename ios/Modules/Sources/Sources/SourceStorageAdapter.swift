// Atlas Platform
import Foundation
import Core

/// Adapts the actor-based `SourceStorage` from Persistence to the
/// `SourceStorageProtocol` expected by `SourceManager`.
///
/// This lives in the Sources module because it bridges Persistence → Sources.
public final class SourceStorageAdapter: SourceStorageProtocol, Sendable {

    private let rootURL: URL

    public init() {
        let appSupport = FileManager.default.urls(
            for: .applicationSupportDirectory, in: .userDomainMask
        ).first!
        self.rootURL = appSupport.appendingPathComponent("atlas/sources", isDirectory: true)
    }

    public func installedSourceIDs() async throws -> [String] {
        try ensureDirectory()
        let contents = try FileManager.default.contentsOfDirectory(
            at: rootURL, includingPropertiesForKeys: nil
        )
        return contents.filter(\.hasDirectoryPath).map(\.lastPathComponent)
    }

    public func loadManifest(sourceId: String) async throws -> SourceManifest {
        let url = sourceDir(sourceId).appendingPathComponent("manifest.json")
        let data = try Data(contentsOf: url)
        return try JSONDecoder().decode(SourceManifest.self, from: data)
    }

    public func loadWASM(sourceId: String, filename: String) async throws -> Data {
        try Data(contentsOf: sourceDir(sourceId).appendingPathComponent(filename))
    }

    public func install(manifest: SourceManifest, wasm: Data) async throws {
        let dir = sourceDir(manifest.id)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let manifestData = try JSONEncoder().encode(manifest)
        try manifestData.write(to: dir.appendingPathComponent("manifest.json"))
        try wasm.write(to: dir.appendingPathComponent(manifest.moduleFilename))
    }

    public func remove(sourceId: String) async throws {
        let dir = sourceDir(sourceId)
        if FileManager.default.fileExists(atPath: dir.path) {
            try FileManager.default.removeItem(at: dir)
        }
    }

    public func isInstalled(sourceId: String) async -> Bool {
        FileManager.default.fileExists(
            atPath: sourceDir(sourceId).appendingPathComponent("manifest.json").path
        )
    }

    private func sourceDir(_ sourceId: String) -> URL {
        rootURL.appendingPathComponent(sourceId, isDirectory: true)
    }

    private func ensureDirectory() throws {
        try FileManager.default.createDirectory(at: rootURL, withIntermediateDirectories: true)
    }
}

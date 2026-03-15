// Atlas Platform
import Foundation
import Core

/// Manages on-disk storage of installed source modules.
///
/// Sources are stored under `<appSupport>/atlas/sources/<sourceId>/`.
/// Each installed source has its `.wasm` module and `manifest.json`.
public actor SourceStorage {

    private let rootURL: URL

    public init() {
        let appSupport = FileManager.default.urls(
            for: .applicationSupportDirectory, in: .userDomainMask
        ).first!
        self.rootURL = appSupport.appendingPathComponent("atlas/sources", isDirectory: true)
    }

    public func installedSourceIDs() throws -> [String] {
        try ensureDirectory()
        let contents = try FileManager.default.contentsOfDirectory(
            at: rootURL, includingPropertiesForKeys: nil
        )
        return contents.filter(\.hasDirectoryPath).map(\.lastPathComponent)
    }

    public func loadManifest(sourceId: String) throws -> SourceManifest {
        let manifestURL = sourceDir(sourceId).appendingPathComponent("manifest.json")
        let data = try Data(contentsOf: manifestURL)
        return try JSONDecoder().decode(SourceManifest.self, from: data)
    }

    public func loadWASM(sourceId: String, filename: String) throws -> Data {
        let wasmURL = sourceDir(sourceId).appendingPathComponent(filename)
        return try Data(contentsOf: wasmURL)
    }

    public func install(manifest: SourceManifest, wasm: Data) throws {
        let dir = sourceDir(manifest.id)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let manifestData = try JSONEncoder().encode(manifest)
        try manifestData.write(to: dir.appendingPathComponent("manifest.json"))
        try wasm.write(to: dir.appendingPathComponent(manifest.moduleFilename))
    }

    public func remove(sourceId: String) throws {
        let dir = sourceDir(sourceId)
        if FileManager.default.fileExists(atPath: dir.path) {
            try FileManager.default.removeItem(at: dir)
        }
    }

    public func isInstalled(sourceId: String) -> Bool {
        FileManager.default.fileExists(
            atPath: sourceDir(sourceId).appendingPathComponent("manifest.json").path
        )
    }

    // MARK: - Private

    private func sourceDir(_ sourceId: String) -> URL {
        rootURL.appendingPathComponent(sourceId, isDirectory: true)
    }

    private func ensureDirectory() throws {
        try FileManager.default.createDirectory(at: rootURL, withIntermediateDirectories: true)
    }
}

// Atlas Platform
import Foundation
import Core
import SourceRuntime
import SourceRegistry

/// Central service for managing source lifecycle: discovery, install, update, removal.
@Observable
public final class SourceManager: @unchecked Sendable {

    public enum SourceState: Sendable {
        case notInstalled
        case installed
        case updateAvailable(newVersion: String)
    }

    public struct InstalledSource: Identifiable, Hashable, Sendable {
        public let id: String
        public let manifest: SourceManifest
        public let instance: SourceInstance

        public static func == (lhs: InstalledSource, rhs: InstalledSource) -> Bool {
            lhs.id == rhs.id
        }
        public func hash(into hasher: inout Hasher) {
            hasher.combine(id)
        }
    }

    // MARK: - State

    public private(set) var registryEntries: [RegistryEntry] = []
    public private(set) var installedSources: [InstalledSource] = []
    public private(set) var isLoadingRegistry = false
    public private(set) var isInstalling: String?
    public private(set) var error: String?

    // MARK: - Dependencies

    private let runtime: AtlasRuntime
    private let registryClient: RegistryClient
    private let storage: SourceStorageProtocol

    /// Base URL of the registry (e.g. raw.githubusercontent.com/user/repo/main/registry)
    public var registryBaseURL: URL

    public init(
        runtime: AtlasRuntime = AtlasRuntime(),
        registryClient: RegistryClient = RegistryClient(),
        storage: SourceStorageProtocol,
        registryBaseURL: URL
    ) {
        self.runtime = runtime
        self.registryClient = registryClient
        self.storage = storage
        self.registryBaseURL = registryBaseURL
    }

    // MARK: - Registry

    public func refreshRegistry() async {
        isLoadingRegistry = true
        error = nil
        do {
            let indexURL = registryBaseURL.appendingPathComponent("index.json")
            let index = try await registryClient.fetchIndex(from: indexURL)
            registryEntries = index.sources
        } catch {
            self.error = error.localizedDescription
        }
        isLoadingRegistry = false
    }

    public func state(for entry: RegistryEntry) -> SourceState {
        if let installed = installedSources.first(where: { $0.id == entry.id }) {
            if installed.manifest.version != entry.latestVersion {
                return .updateAvailable(newVersion: entry.latestVersion)
            }
            return .installed
        }
        return .notInstalled
    }

    // MARK: - Install / Update

    public func install(entry: RegistryEntry) async throws {
        isInstalling = entry.id
        defer { isInstalling = nil }

        // 1. Fetch manifest
        let manifestURL = registryBaseURL.appendingPathComponent(entry.manifestURL)
        let manifest = try await registryClient.fetchManifest(from: manifestURL)

        // 2. Download WASM module — path: sources/{id}/{version}/{filename}
        let wasmURL = registryBaseURL
            .appendingPathComponent("sources/\(entry.id)/\(manifest.version)")
            .appendingPathComponent(manifest.moduleFilename)
        let wasm = try await registryClient.downloadModule(from: wasmURL)

        // 3. Store on disk
        try await storage.install(manifest: manifest, wasm: wasm)

        // 4. Load into runtime
        let instance = try runtime.loadSource(wasm: wasm, manifest: manifest)
        let source = InstalledSource(id: manifest.id, manifest: manifest, instance: instance)
        installedSources.removeAll { $0.id == manifest.id }
        installedSources.append(source)
    }

    public func remove(sourceId: String) async throws {
        try await storage.remove(sourceId: sourceId)
        installedSources.removeAll { $0.id == sourceId }
    }

    // MARK: - Load from disk

    public func loadInstalledSources() async {
        do {
            let ids = try await storage.installedSourceIDs()
            for sourceId in ids {
                do {
                    let manifest = try await storage.loadManifest(sourceId: sourceId)
                    let wasm = try await storage.loadWASM(
                        sourceId: sourceId,
                        filename: manifest.moduleFilename
                    )
                    let instance = try runtime.loadSource(wasm: wasm, manifest: manifest)
                    installedSources.append(
                        InstalledSource(id: sourceId, manifest: manifest, instance: instance)
                    )
                } catch {
                    print("[atlas] Failed to load source \(sourceId): \(error)")
                }
            }
        } catch {
            print("[atlas] Failed to enumerate installed sources: \(error)")
        }
    }

    public func source(id: String) -> SourceInstance? {
        installedSources.first(where: { $0.id == id })?.instance
    }
}

/// Protocol so we can use SourceStorage from Persistence module.
public protocol SourceStorageProtocol: Sendable {
    func installedSourceIDs() async throws -> [String]
    func loadManifest(sourceId: String) async throws -> SourceManifest
    func loadWASM(sourceId: String, filename: String) async throws -> Data
    func install(manifest: SourceManifest, wasm: Data) async throws
    func remove(sourceId: String) async throws
    func isInstalled(sourceId: String) async -> Bool
}

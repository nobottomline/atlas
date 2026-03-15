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

    public struct InstalledSource: Identifiable, Sendable {
        public let id: String
        public let manifest: SourceManifest
        public let instance: SourceInstance
    }

    // MARK: - State

    public private(set) var registryEntries: [RegistryEntry] = []
    public private(set) var installedSources: [InstalledSource] = []
    public private(set) var isLoadingRegistry = false
    public private(set) var error: String?

    // MARK: - Dependencies

    private let runtime: AtlasRuntime
    private let registryClient: RegistryClient
    private let storage: SourceStorageProtocol
    private let registryBaseURL: URL

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

    /// Fetch the registry index and update available sources.
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

    /// Check the state of a source.
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

    /// Install a source from the registry.
    public func install(entry: RegistryEntry) async throws {
        // 1. Fetch manifest
        let manifestURL = registryBaseURL.appendingPathComponent(entry.manifestURL)
        let manifest = try await registryClient.fetchManifest(from: manifestURL)

        // 2. Download WASM module
        let wasmURL = registryBaseURL
            .appendingPathComponent("sources/\(entry.id)/0.1.0")
            .appendingPathComponent(manifest.moduleFilename)
        let wasm = try await registryClient.downloadModule(from: wasmURL)

        // 3. Store on disk
        try await storage.install(manifest: manifest, wasm: wasm)

        // 4. Load into runtime
        let instance = try runtime.loadSource(wasm: wasm, manifest: manifest)
        let source = InstalledSource(id: manifest.id, manifest: manifest, instance: instance)
        installedSources.append(source)
    }

    /// Remove an installed source.
    public func remove(sourceId: String) async throws {
        try await storage.remove(sourceId: sourceId)
        installedSources.removeAll { $0.id == sourceId }
    }

    // MARK: - Load from disk

    /// Load all previously installed sources from disk.
    public func loadInstalledSources() async {
        do {
            let ids = try await storage.installedSourceIDs()
            for id in ids {
                do {
                    let manifest = try await storage.loadManifest(sourceId: id)
                    let wasm = try await storage.loadWASM(
                        sourceId: id,
                        filename: manifest.moduleFilename
                    )
                    let instance = try runtime.loadSource(wasm: wasm, manifest: manifest)
                    installedSources.append(
                        InstalledSource(id: id, manifest: manifest, instance: instance)
                    )
                } catch {
                    print("[atlas] Failed to load source \(id): \(error)")
                }
            }
        } catch {
            print("[atlas] Failed to enumerate installed sources: \(error)")
        }
    }

    /// Get an installed source instance by ID.
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

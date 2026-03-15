import Foundation
import Core

/// Fetches and caches registry metadata from a remote or local URL.
public actor RegistryClient {

    private let session: URLSession
    private let decoder: JSONDecoder

    public init(session: URLSession = .shared) {
        self.session = session
        self.decoder = JSONDecoder()
        self.decoder.dateDecodingStrategy = .iso8601
    }

    // MARK: - Index

    /// Fetch and decode the registry index.
    public func fetchIndex(from url: URL) async throws -> RegistryIndex {
        let (data, response) = try await session.data(from: url)
        try validate(response: response, url: url)
        return try decoder.decode(RegistryIndex.self, from: data)
    }

    // MARK: - Manifest

    /// Fetch a full source manifest by URL.
    public func fetchManifest(from url: URL) async throws -> SourceManifest {
        let (data, response) = try await session.data(from: url)
        try validate(response: response, url: url)
        return try decoder.decode(SourceManifest.self, from: data)
    }

    // MARK: - Module download

    /// Download a WASM module and return the raw bytes.
    public func downloadModule(from url: URL) async throws -> Data {
        let (data, response) = try await session.data(from: url)
        try validate(response: response, url: url)
        return data
    }

    // MARK: - Revocations

    /// Fetch the revocation list.
    public func fetchRevocations(from url: URL) async throws -> [RevocationRecord] {
        let (data, response) = try await session.data(from: url)
        try validate(response: response, url: url)
        return try decoder.decode([RevocationRecord].self, from: data)
    }

    // MARK: - Private

    private func validate(response: URLResponse, url: URL) throws {
        guard let http = response as? HTTPURLResponse else { return }
        guard (200..<300).contains(http.statusCode) else {
            throw RegistryError.httpError(statusCode: http.statusCode, url: url)
        }
    }
}

// MARK: - Errors

public enum RegistryError: LocalizedError {
    case httpError(statusCode: Int, url: URL)
    case integrityFailed(expected: String, actual: String)
    case manifestMissing(sourceId: String)
    case revoked(sourceId: String, reason: String)

    public var errorDescription: String? {
        switch self {
        case .httpError(let code, let url):
            return "HTTP \(code) fetching \(url)"
        case .integrityFailed(let expected, let actual):
            return "Integrity check failed: expected \(expected), got \(actual)"
        case .manifestMissing(let id):
            return "Manifest not found for source: \(id)"
        case .revoked(let id, let reason):
            return "Source \(id) has been revoked: \(reason)"
        }
    }
}

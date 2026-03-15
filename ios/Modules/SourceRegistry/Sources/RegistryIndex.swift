import Foundation

/// Top-level registry catalog, fetched from `index.json`.
public struct RegistryIndex: Codable, Sendable {
    public var version: String
    public var sources: [RegistryEntry]
}

/// Lightweight catalog entry for a single source.
public struct RegistryEntry: Identifiable, Codable, Sendable {
    public let id: String
    public var name: String
    public var latestVersion: String
    public var lang: String
    public var manifestURL: String
    public var iconURL: String?
    public var description: String?
    public var tags: [String]
    public var deprecated: Bool

    enum CodingKeys: String, CodingKey {
        case id, name, lang, description, tags, deprecated
        case latestVersion  = "latest_version"
        case manifestURL    = "manifest_url"
        case iconURL        = "icon_url"
    }
}

/// A single revocation record from `revocations.json`.
public struct RevocationRecord: Codable, Sendable {
    public var sourceId: String
    public var version: String?
    public var reason: String
    public var revokedAt: Date

    enum CodingKeys: String, CodingKey {
        case reason
        case sourceId   = "source_id"
        case version
        case revokedAt  = "revoked_at"
    }
}

import Foundation

/// Swift mirror of `atlas_spec::manifest::SourceManifest`.
public struct SourceManifest: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var name: String
    public var version: String
    public var lang: String
    public var baseURLs: [String]
    public var contentType: String
    public var supportsNSFW: Bool
    public var moduleFilename: String
    public var moduleSHA256: String
    public var signature: String?
    public var minRuntimeVersion: String
    public var minAppVersion: String?
    public var capabilities: [String]
    public var allowedDomains: [String]
    public var author: String?
    public var license: String?
    public var tags: [String]
    public var deprecated: Bool
    public var replacedBy: String?
    public var description: String?

    enum CodingKeys: String, CodingKey {
        case id, name, version, lang
        case baseURLs           = "base_urls"
        case contentType        = "content_type"
        case supportsNSFW       = "supports_nsfw"
        case moduleFilename     = "module_filename"
        case moduleSHA256       = "module_sha256"
        case signature
        case minRuntimeVersion  = "min_runtime_version"
        case minAppVersion      = "min_app_version"
        case capabilities
        case allowedDomains     = "allowed_domains"
        case author, license, tags, deprecated
        case replacedBy         = "replaced_by"
        case description
    }
}

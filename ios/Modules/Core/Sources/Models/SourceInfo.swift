// Atlas Platform
import Foundation

/// Swift mirror of `atlas_spec::types::source_info::SourceInfo`.
public struct SourceInfo: Codable, Sendable, Identifiable {
    public let id: String
    public var name: String
    public var version: String
    public var lang: String
    public var baseURLs: [String]
    public var contentType: ContentType
    public var supportsNSFW: Bool
    public var capabilities: [String]
    public var iconURL: String?
    public var description: String?

    enum CodingKeys: String, CodingKey {
        case id, name, version, lang
        case baseURLs       = "base_urls"
        case contentType    = "content_type"
        case supportsNSFW   = "supports_nsfw"
        case capabilities
        case iconURL        = "icon_url"
        case description
    }
}

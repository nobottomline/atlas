import Foundation

/// Swift mirror of `atlas_spec::types::manga::Manga`.
public struct Manga: Identifiable, Codable, Sendable {
    public let id: String
    public var title: String
    public var url: String
    public var coverURL: String?
    public var author: String?
    public var artist: String?
    public var description: String?
    public var tags: [String]
    public var status: MangaStatus
    public var contentRating: ContentRating
    public var contentType: ContentType
    public var lang: String
    public var altTitles: [String]

    enum CodingKeys: String, CodingKey {
        case id, title, url
        case coverURL      = "cover_url"
        case author, artist, description, tags, status
        case contentRating = "content_rating"
        case contentType   = "content_type"
        case lang
        case altTitles     = "alt_titles"
    }
}

public enum MangaStatus: String, Codable, Sendable {
    case ongoing, completed, hiatus, cancelled, unknown
}

public enum ContentRating: String, Codable, Sendable {
    case safe, suggestive, nsfw
}

public enum ContentType: String, Codable, Sendable {
    case manga, manhwa, manhua, comic, novel, webtoon, unknown
}

/// Lightweight entry for listing/search results.
public struct MangaEntry: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var title: String
    public var url: String
    public var coverURL: String?
    public var contentRating: ContentRating

    enum CodingKeys: String, CodingKey {
        case id, title, url
        case coverURL      = "cover_url"
        case contentRating = "content_rating"
    }
}

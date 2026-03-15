import Foundation

/// Swift mirror of `atlas_spec::types::chapter::Chapter`.
public struct Chapter: Identifiable, Codable, Sendable, Hashable {
    public let id: String
    public var mangaId: String
    public var title: String?
    public var number: Double?
    public var volume: Double?
    public var lang: String
    public var dateUpdated: Date?
    public var scanlator: String?
    public var url: String

    enum CodingKeys: String, CodingKey {
        case id
        case mangaId      = "manga_id"
        case title, number, volume, lang
        case dateUpdated  = "date_updated"
        case scanlator, url
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id          = try container.decode(String.self, forKey: .id)
        mangaId     = try container.decode(String.self, forKey: .mangaId)
        title       = try container.decodeIfPresent(String.self, forKey: .title)
        number      = try container.decodeIfPresent(Double.self, forKey: .number)
        volume      = try container.decodeIfPresent(Double.self, forKey: .volume)
        lang        = try container.decode(String.self, forKey: .lang)
        scanlator   = try container.decodeIfPresent(String.self, forKey: .scanlator)
        url         = try container.decode(String.self, forKey: .url)

        if let ts = try container.decodeIfPresent(Int64.self, forKey: .dateUpdated) {
            dateUpdated = Date(timeIntervalSince1970: Double(ts))
        } else {
            dateUpdated = nil
        }
    }
}

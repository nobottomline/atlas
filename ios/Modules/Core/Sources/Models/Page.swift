// Atlas Platform
import Foundation

/// Swift mirror of `atlas_spec::types::page::Page`.
public struct Page: Codable, Sendable, Identifiable {
    public var id: UInt32 { index }
    public var index: UInt32
    public var data: PageData
}

/// Swift mirror of `atlas_spec::types::page::PageData`.
public enum PageData: Codable, Sendable {
    case url(String)
    case base64(String)
    case text(String)

    enum CodingKeys: String, CodingKey {
        case url = "Url"
        case base64 = "Base64"
        case text = "Text"
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if let url = try container.decodeIfPresent(String.self, forKey: .url) {
            self = .url(url)
        } else if let b64 = try container.decodeIfPresent(String.self, forKey: .base64) {
            self = .base64(b64)
        } else if let text = try container.decodeIfPresent(String.self, forKey: .text) {
            self = .text(text)
        } else {
            throw DecodingError.dataCorrupted(.init(
                codingPath: decoder.codingPath,
                debugDescription: "Unknown PageData variant"
            ))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .url(let url): try container.encode(url, forKey: .url)
        case .base64(let b64): try container.encode(b64, forKey: .base64)
        case .text(let text): try container.encode(text, forKey: .text)
        }
    }

    /// Returns the image URL if this is a URL page.
    public var imageURL: URL? {
        if case .url(let str) = self { return URL(string: str) }
        return nil
    }
}

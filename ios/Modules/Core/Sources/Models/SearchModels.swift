// Atlas Platform
import Foundation

/// Swift mirror of `atlas_spec::types::search::SearchQuery`.
public struct SearchQuery: Codable, Sendable {
    public var title: String?
    public var filters: [AppliedFilter]
    public var page: UInt32

    public init(title: String? = nil, filters: [AppliedFilter] = [], page: UInt32 = 1) {
        self.title = title
        self.filters = filters
        self.page = page
    }
}

/// Swift mirror of `atlas_spec::types::search::SearchResponse`.
public struct SearchResponse: Codable, Sendable {
    public var entries: [MangaEntry]
    public var hasNextPage: Bool
    public var total: UInt32?

    enum CodingKeys: String, CodingKey {
        case entries
        case hasNextPage = "has_next_page"
        case total
    }
}

/// Swift mirror of `atlas_spec::types::search::Filter`.
public struct Filter: Identifiable, Codable, Sendable {
    public let id: String
    public var label: String
    public var kind: FilterKind
}

public enum FilterKind: String, Codable, Sendable {
    case select
    case multiSelect = "multi_select"
    case toggle
    case sort
    case text
}

/// Swift mirror of `atlas_spec::types::search::AppliedFilter`.
public struct AppliedFilter: Codable, Sendable {
    public var filterId: String
    public var value: FilterValue

    enum CodingKeys: String, CodingKey {
        case filterId = "filter_id"
        case value
    }

    public init(filterId: String, value: FilterValue) {
        self.filterId = filterId
        self.value = value
    }
}

public enum FilterValue: Codable, Sendable {
    case str(String)
    case strList([String])
    case bool(Bool)

    enum CodingKeys: String, CodingKey {
        case str, strList = "str_list", bool
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let s = try? container.decode(String.self) {
            self = .str(s)
        } else if let b = try? container.decode(Bool.self) {
            self = .bool(b)
        } else if let arr = try? container.decode([String].self) {
            self = .strList(arr)
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown FilterValue")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .str(let s): try container.encode(s)
        case .strList(let arr): try container.encode(arr)
        case .bool(let b): try container.encode(b)
        }
    }
}

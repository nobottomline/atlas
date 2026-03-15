// Atlas Platform
import Foundation

/// Swift mirror of `atlas_spec::preferences::PreferenceSchema`.
public struct PreferenceSchema: Codable, Sendable {
    public var fields: [PreferenceField]

    public init(fields: [PreferenceField] = []) {
        self.fields = fields
    }
}

/// Swift mirror of `atlas_spec::preferences::PreferenceField`.
public struct PreferenceField: Identifiable, Codable, Sendable {
    public var id: String { key }
    public var key: String
    public var label: String
    public var description: String?
    public var kind: PreferenceFieldKind
    public var defaultValue: PreferenceValue?
    public var visible: Bool

    enum CodingKeys: String, CodingKey {
        case key, label, description, kind
        case defaultValue = "default_value"
        case visible
    }
}

public enum PreferenceFieldKind: String, Codable, Sendable {
    case text, select, multiSelect = "multi_select", toggle, number, password
}

public enum PreferenceValue: Codable, Sendable {
    case str(String)
    case bool(Bool)
    case num(Double)
    case strList([String])

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let b = try? container.decode(Bool.self) {
            self = .bool(b)
        } else if let n = try? container.decode(Double.self) {
            self = .num(n)
        } else if let s = try? container.decode(String.self) {
            self = .str(s)
        } else if let arr = try? container.decode([String].self) {
            self = .strList(arr)
        } else {
            throw DecodingError.dataCorruptedError(
                in: container, debugDescription: "Unknown PreferenceValue"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .str(let s): try container.encode(s)
        case .bool(let b): try container.encode(b)
        case .num(let n): try container.encode(n)
        case .strList(let arr): try container.encode(arr)
        }
    }
}

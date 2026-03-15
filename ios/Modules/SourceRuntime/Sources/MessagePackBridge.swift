// Atlas Platform
import Foundation

/// Bridges between Swift Codable types and JSON wire encoding.
///
/// All host-guest communication uses JSON for cross-language compatibility
/// between the Swift iOS host and Rust WASM modules (serde_json).
enum MessagePackBridge {

    nonisolated(unsafe) static let jsonEncoder = JSONEncoder()
    nonisolated(unsafe) static let jsonDecoder = JSONDecoder()

    // MARK: - Encode (Swift → WASM)

    static func encode<T: Encodable>(_ value: T) throws -> Data {
        try jsonEncoder.encode(value)
    }

    // MARK: - Decode (WASM → Swift)

    /// Decode a JSON-encoded `AtlasResultWire<T>` response.
    static func decodeResult<T: Decodable>(_ data: Data) throws -> T {
        let envelope: ResultEnvelope<T>
        do {
            envelope = try jsonDecoder.decode(ResultEnvelope<T>.self, from: data)
        } catch {
            let preview = String(data: data.prefix(300), encoding: .utf8) ?? "<binary>"
            throw SourceRuntimeError.sourceError("decode: \(error.localizedDescription) | \(preview)")
        }

        switch envelope.status {
        case "ok":
            guard let value = envelope.data else {
                throw SourceRuntimeError.sourceError("ok envelope has no data")
            }
            return value
        case "err":
            let extracted = envelope.errorMessage ?? "unknown error"
            throw SourceRuntimeError.sourceError(extracted)
        default:
            throw SourceRuntimeError.sourceError("unexpected status: \(envelope.status)")
        }
    }

    // MARK: - Wire types

    private struct ResultEnvelope<T: Decodable>: Decodable {
        let status: String
        let data: T?
        let errorMessage: String?

        enum CodingKeys: String, CodingKey {
            case status, data
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            status = try container.decode(String.self, forKey: .status)
            if status == "ok" {
                data = try container.decodeIfPresent(T.self, forKey: .data)
                errorMessage = nil
            } else {
                data = nil
                errorMessage = Self.extractErrorMessage(from: container)
            }
        }

        /// Rust SourceError: #[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
        /// Format: {"kind": "runtime_failure", "detail": {"message": "..."}}
        private static func extractErrorMessage(
            from container: KeyedDecodingContainer<CodingKeys>
        ) -> String? {
            // Adjacently tagged with kind/detail
            if let err = try? container.decode(SourceErrorJSON.self, forKey: .data) {
                let kind = err.kind
                if let detail = err.detail {
                    if let msg = detail["message"] { return "\(kind): \(msg)" }
                    if let cap = detail["capability"] { return "\(kind): \(cap)" }
                }
                return kind
            }
            // Unit variants without detail (e.g. "not_found")
            if let str = try? container.decode(String.self, forKey: .data) {
                return str
            }
            return nil
        }
    }

    /// Matches Rust SourceError serde format
    private struct SourceErrorJSON: Decodable {
        let kind: String
        let detail: [String: String]?
    }
}

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

    /// Encode a Swift `Encodable` value to JSON bytes.
    static func encode<T: Encodable>(_ value: T) throws -> Data {
        try jsonEncoder.encode(value)
    }

    // MARK: - Decode (WASM → Swift)

    /// Decode a JSON-encoded `AtlasResultWire<T>` response.
    ///
    /// Rust wire format (adjacently tagged enum):
    /// `{"status": "ok", "data": T}` or `{"status": "err", "data": SourceError}`
    static func decodeResult<T: Decodable>(_ data: Data) throws -> T {
        let envelope: ResultEnvelope<T>
        do {
            envelope = try jsonDecoder.decode(ResultEnvelope<T>.self, from: data)
        } catch {
            let preview = String(data: data.prefix(300), encoding: .utf8) ?? "<binary>"
            print("[atlas-bridge] Decode failed (\(data.count)B): \(error)")
            print("[atlas-bridge] JSON preview: \(preview)")
            throw SourceRuntimeError.sourceError("decode failed: \(error.localizedDescription)")
        }

        switch envelope.status {
        case "ok":
            guard let value = envelope.data else {
                throw SourceRuntimeError.sourceError("ok envelope has no data")
            }
            return value
        case "err":
            let msg = envelope.errorMessage ?? "source returned an error"
            throw SourceRuntimeError.sourceError(msg)
        default:
            throw SourceRuntimeError.sourceError("unexpected status: \(envelope.status)")
        }
    }

    // MARK: - Decode helpers

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

        /// Extract message from Rust SourceError variants.
        /// Externally tagged: {"VariantName": {"message": "..."}}
        private static func extractErrorMessage(
            from container: KeyedDecodingContainer<CodingKeys>
        ) -> String? {
            if let map = try? container.decode([String: [String: String]].self, forKey: .data) {
                for (variant, fields) in map {
                    if let msg = fields["message"] { return "\(variant): \(msg)" }
                    return variant
                }
            }
            if let str = try? container.decode(String.self, forKey: .data) {
                return str
            }
            return nil
        }
    }
}

// Atlas Platform
import Foundation
import MessagePack

/// Bridges between Swift Codable types and MessagePack wire encoding.
///
/// The Rust WASM modules use `rmp_serde::to_vec_named` / `rmp_serde::from_slice`,
/// which produces/consumes named (map-style) MessagePack. This bridge uses
/// MessagePack.swift's `MessagePackEncoder`/`MessagePackDecoder` which are
/// compatible with that format.
enum MessagePackBridge {

    private nonisolated(unsafe) static let encoder = MessagePackEncoder()
    private nonisolated(unsafe) static let decoder = MessagePackDecoder()

    /// Encode a Swift `Encodable` value to MessagePack bytes.
    static func encode<T: Encodable>(_ value: T) throws -> Data {
        try encoder.encode(value)
    }

    /// Decode a MessagePack-encoded `AtlasResultWire<T>` response envelope.
    ///
    /// The Rust side emits:
    /// ```
    /// #[serde(tag = "status", content = "data")]
    /// enum AtlasResultWire<T> { Ok(T), Err(SourceError) }
    /// ```
    static func decodeResult<T: Decodable>(_ data: Data) throws -> T {
        let envelope = try decoder.decode(ResultEnvelope<T>.self, from: data)
        switch envelope.status {
        case "ok":
            guard let value = envelope.data else {
                throw DecodingError.valueNotFound(T.self, .init(
                    codingPath: [],
                    debugDescription: "ok envelope has no data"
                ))
            }
            return value
        case "err":
            let message = envelope.errorMessage ?? "unknown source error"
            throw SourceRuntimeError.sourceError(message)
        default:
            throw SourceRuntimeError.sourceError("unexpected envelope status: \(envelope.status)")
        }
    }

    /// Encode an `AtlasResultWire::Ok(value)` envelope as MessagePack.
    static func encodeOk<T: Encodable>(_ value: T) throws -> Data {
        let envelope = ResultEnvelopeOut(status: "ok", data: value)
        return try encoder.encode(envelope)
    }

    /// Encode an `AtlasResultWire::Err` envelope as MessagePack.
    static func encodeError(message: String) throws -> Data {
        let error = SourceErrorWire(kind: "runtime_failure", message: message)
        let envelope = ErrorEnvelopeOut(status: "err", data: error)
        return try encoder.encode(envelope)
    }

    /// Encode an `AtlasResultWire::Ok(null)` envelope as MessagePack.
    static func encodeOkNil() throws -> Data {
        let envelope = NilEnvelopeOut(status: "ok", data: nil as String?)
        return try encoder.encode(envelope)
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
                // Try to decode SourceError from data field
                if let errData = try? container.decode(SourceErrorWire.self, forKey: .data) {
                    errorMessage = errData.message
                } else {
                    errorMessage = nil
                }
            }
        }
    }

    private struct ResultEnvelopeOut<T: Encodable>: Encodable {
        let status: String
        let data: T
    }

    private struct ErrorEnvelopeOut: Encodable {
        let status: String
        let data: SourceErrorWire
    }

    private struct NilEnvelopeOut: Encodable {
        let status: String
        let data: String?
    }

    struct SourceErrorWire: Codable {
        let kind: String?
        let message: String?

        enum CodingKeys: String, CodingKey {
            case message
            // Handle various error shapes from Rust
        }

        init(kind: String, message: String) {
            self.kind = kind
            self.message = message
        }

        init(from decoder: Decoder) throws {
            // SourceError is a tagged enum in Rust. Try multiple decode strategies.
            if let container = try? decoder.container(keyedBy: CodingKeys.self) {
                message = try container.decodeIfPresent(String.self, forKey: .message)
                kind = nil
            } else if let single = try? decoder.singleValueContainer(),
                      let str = try? single.decode(String.self) {
                message = str
                kind = nil
            } else {
                message = "unknown error"
                kind = nil
            }
        }
    }
}

// Atlas Platform
import Foundation
import MessagePack

/// Bridges between Swift Codable types and MessagePack wire encoding.
///
/// For ENCODING (Swift → WASM guest): manually constructs MessagePack bytes
/// to match exactly what `rmp_serde::from_slice` expects.
/// For DECODING (WASM guest → Swift): uses Flight-School's MessagePackDecoder.
enum MessagePackBridge {

    nonisolated(unsafe) static let mpEncoder = MessagePackEncoder()
    nonisolated(unsafe) static let mpDecoder = MessagePackDecoder()

    // MARK: - Encode (Swift → WASM)

    /// Encode a Swift `Encodable` value to MessagePack bytes.
    /// Used for encoding arguments (SearchQuery, String, UInt32, etc.)
    static func encode<T: Encodable>(_ value: T) throws -> Data {
        try mpEncoder.encode(value)
    }

    /// Encode `AtlasResultWire::Ok(value)` — manually constructs the envelope
    /// to ensure rmp_serde compatibility.
    ///
    /// Produces: `fixmap(2) { "status": "ok", "data": <msgpack(value)> }`
    static func encodeOk<T: Encodable>(_ value: T) throws -> Data {
        let innerData = try mpEncoder.encode(value)
        return buildEnvelope(status: "ok", innerData: innerData)
    }

    /// Encode `AtlasResultWire::Err(SourceError::RuntimeFailure { message })`.
    ///
    /// Rust SourceError uses default (externally tagged) serde:
    /// `{"RuntimeFailure": {"message": "..."}}`
    static func encodeError(message: String) throws -> Data {
        // Build inner error: {"RuntimeFailure": {"message": "..."}}
        let msgBytes = packStr(message)
        let innerMap = packMap(count: 1,
            key: packStr("message"),
            value: msgBytes
        )
        let errorMap = packMap(count: 1,
            key: packStr("RuntimeFailure"),
            value: innerMap
        )
        return buildEnvelope(status: "err", innerData: errorMap)
    }

    /// Encode `AtlasResultWire::Ok(None)`.
    static func encodeOkNil() throws -> Data {
        buildEnvelope(status: "ok", innerData: Data([0xc0])) // 0xc0 = msgpack nil
    }

    // MARK: - Decode (WASM → Swift)

    /// Decode a MessagePack-encoded `AtlasResultWire<T>` response.
    static func decodeResult<T: Decodable>(_ data: Data) throws -> T {
        let envelope: ResultEnvelope<T>
        do {
            envelope = try mpDecoder.decode(ResultEnvelope<T>.self, from: data)
        } catch {
            print("[atlas-bridge] Decode failed (\(data.count)B): \(error)")
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

    // MARK: - Manual MessagePack construction

    /// Build the AtlasResultWire::Ok envelope with pre-encoded inner data.
    static func buildOkEnvelope(innerData: Data) -> Data {
        buildEnvelope(status: "ok", innerData: innerData)
    }

    /// Build the AtlasResultWire envelope: `{"status": "<status>", "data": <raw>}`
    private static func buildEnvelope(status: String, innerData: Data) -> Data {
        var out = Data()
        // fixmap with 2 entries
        out.append(0x82)
        // key: "status"
        out.append(contentsOf: packStr("status"))
        // value: status string
        out.append(contentsOf: packStr(status))
        // key: "data"
        out.append(contentsOf: packStr("data"))
        // value: raw inner data
        out.append(innerData)
        return out
    }

    /// Encode a string as MessagePack fixstr/str8/str16.
    static func packStr(_ str: String) -> Data {
        let utf8 = Array(str.utf8)
        let len = utf8.count
        var out = Data()
        if len < 32 {
            out.append(UInt8(0xa0 | len)) // fixstr
        } else if len < 256 {
            out.append(0xd9) // str8
            out.append(UInt8(len))
        } else {
            out.append(0xda) // str16
            out.append(UInt8(len >> 8))
            out.append(UInt8(len & 0xff))
        }
        out.append(contentsOf: utf8)
        return out
    }

    /// Build a MessagePack map with 1 entry from pre-encoded key and value.
    private static func packMap(count: Int, key: Data, value: Data) -> Data {
        var out = Data()
        if count < 16 {
            out.append(UInt8(0x80 | count)) // fixmap
        } else {
            out.append(0xde) // map16
            out.append(UInt8(count >> 8))
            out.append(UInt8(count & 0xff))
        }
        out.append(key)
        out.append(value)
        return out
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

        private static func extractErrorMessage(
            from container: KeyedDecodingContainer<CodingKeys>
        ) -> String? {
            // Rust SourceError externally tagged: {"VariantName": {"message": "..."}}
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

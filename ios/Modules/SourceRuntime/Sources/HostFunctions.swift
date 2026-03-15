// Atlas Platform
import Foundation
@preconcurrency import WasmKit

/// Implements all Atlas host-import functions for the WasmKit runtime.
///
/// Host functions use JSON wire format for cross-language compatibility.
/// (Swift JSONEncoder/JSONDecoder ↔ Rust serde_json)
final class HostFunctions: @unchecked Sendable {

    let policy: CapabilityPolicy
    private let store: Store
    private var logBuffer: [String] = []
    private var cache: [String: (data: Data, expiry: Date?)] = [:]
    private let session: URLSession
    private let jsonEncoder = JSONEncoder()
    private let jsonDecoder = JSONDecoder()

    init(policy: CapabilityPolicy, store: Store, session: URLSession = .shared) {
        self.policy = policy
        self.store = store
        self.session = session
    }

    func register(into imports: inout Imports) {
        imports.define(
            module: "atlas", name: "host_network_fetch",
            Function(store: store, parameters: [.i32, .i32], results: [.i64]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i64(0)] }
                return [.i64(self.hostNetworkFetch(caller: caller, args: args))]
            }
        )
        imports.define(
            module: "atlas", name: "host_log_debug",
            Function(store: store, parameters: [.i32, .i32]) {
                [weak self] caller, args -> [Value] in
                self?.hostLogDebug(caller: caller, args: args)
                return []
            }
        )
        imports.define(
            module: "atlas", name: "host_time_now",
            Function(store: store, parameters: [], results: [.i64]) {
                [weak self] _, _ -> [Value] in
                guard let self, (try? self.policy.require("time.now")) != nil else { return [.i64(0)] }
                return [.i64(UInt64(Date().timeIntervalSince1970))]
            }
        )
        imports.define(
            module: "atlas", name: "host_cache_get",
            Function(store: store, parameters: [.i32, .i32], results: [.i64]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i64(0)] }
                return [.i64(self.hostCacheGet(caller: caller, args: args))]
            }
        )
        imports.define(
            module: "atlas", name: "host_cache_set",
            Function(store: store, parameters: [.i32, .i32], results: [.i32]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i32(1)] }
                return [.i32(self.hostCacheSet(caller: caller, args: args))]
            }
        )
        imports.define(
            module: "atlas", name: "host_preferences_get",
            Function(store: store, parameters: [.i32, .i32], results: [.i64]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i64(0)] }
                return [.i64(self.hostPreferencesGet(caller: caller, args: args))]
            }
        )
    }

    func drainLogs() -> [String] {
        defer { logBuffer.removeAll() }
        return logBuffer
    }

    // MARK: - Network fetch

    private struct FetchRequestWire: Decodable {
        let url: String
        let method: String
        let headers: [String: String]
        let body: [UInt8]?
    }

    private struct FetchResponseWire: Encodable {
        let status: UInt16
        let headers: [String: String]
        let body: [UInt8]
    }

    /// JSON envelope matching Rust AtlasResult<T>
    private struct OkEnvelope<T: Encodable>: Encodable {
        let status: String
        let data: T
    }

    private struct ErrEnvelope: Encodable {
        let status: String
        let data: ErrData
    }

    private struct ErrData: Encodable {
        let RuntimeFailure: MessageField // swiftlint:disable:this identifier_name
    }

    private struct MessageField: Encodable {
        let message: String
    }

    private struct NilEnvelope: Encodable {
        let status: String
        let data: String?
    }

    private func hostNetworkFetch(caller: Caller, args: [Value]) -> UInt64 {
        guard (try? policy.require("network.fetch")) != nil else {
            return writeJsonError(caller: caller, message: "capability denied: network.fetch")
        }

        let ptr = Int(args[0].i32)
        let len = Int(args[1].i32)

        guard let memory = caller.instance?.exports[memory: "memory"] else { return 0 }
        let bytes = Array(memory.data[ptr..<(ptr + len)])

        // Decode FetchRequest from JSON (SDK sends JSON for host functions)
        let request: FetchRequestWire
        do {
            request = try jsonDecoder.decode(FetchRequestWire.self, from: Data(bytes))
            print("[atlas-host] fetch: \(request.method) \(request.url)")
        } catch {
            print("[atlas-host] decode FetchRequest FAILED: \(error)")
            return writeJsonError(caller: caller, message: "decode FetchRequest: \(error)")
        }

        guard let url = URL(string: request.url) else {
            return writeJsonError(caller: caller, message: "invalid URL: \(request.url)")
        }

        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = request.method.uppercased()
        for (key, value) in request.headers {
            urlRequest.setValue(value, forHTTPHeaderField: key)
        }
        if let body = request.body {
            urlRequest.httpBody = Data(body)
        }
        urlRequest.timeoutInterval = 30

        // Synchronous network — WASM is single-threaded
        let semaphore = DispatchSemaphore(value: 0)
        var responseData: Data?
        var responseHTTP: HTTPURLResponse?
        var responseError: Error?

        let task = session.dataTask(with: urlRequest) { data, response, error in
            responseData = data
            responseHTTP = response as? HTTPURLResponse
            responseError = error
            semaphore.signal()
        }
        task.resume()
        semaphore.wait()

        if let error = responseError {
            return writeJsonError(caller: caller, message: "network: \(error.localizedDescription)")
        }

        let status = UInt16(responseHTTP?.statusCode ?? 0)
        var headers: [String: String] = [:]
        for (key, value) in responseHTTP?.allHeaderFields ?? [:] {
            headers[String(describing: key).lowercased()] = String(describing: value)
        }

        print("[atlas-host] fetch response: HTTP \(status), \(responseData?.count ?? 0) bytes")

        let response = FetchResponseWire(
            status: status,
            headers: headers,
            body: Array(responseData ?? Data())
        )
        return writeJsonOk(caller: caller, value: response)
    }

    // MARK: - Log

    private func hostLogDebug(caller: Caller, args: [Value]) {
        guard (try? policy.require("log.debug")) != nil else { return }
        let ptr = Int(args[0].i32)
        let len = Int(args[1].i32)
        guard let memory = caller.instance?.exports[memory: "memory"] else { return }
        let bytes = Array(memory.data[ptr..<(ptr + len)])
        if let msg = String(bytes: bytes, encoding: .utf8) {
            logBuffer.append(msg)
            print("[atlas-source] \(msg)")
        }
    }

    // MARK: - Cache

    private func hostCacheGet(caller: Caller, args: [Value]) -> UInt64 {
        guard (try? policy.require("cache.read")) != nil else { return 0 }
        return writeJsonNil(caller: caller)
    }

    private func hostCacheSet(caller: Caller, args: [Value]) -> UInt32 {
        guard (try? policy.require("cache.write")) != nil else { return 1 }
        return 0
    }

    // MARK: - Preferences

    private func hostPreferencesGet(caller: Caller, args: [Value]) -> UInt64 {
        guard (try? policy.require("preferences.read")) != nil else { return 0 }
        return writeJsonNil(caller: caller)
    }

    // MARK: - JSON response helpers

    private func writeJsonOk<T: Encodable>(caller: Caller, value: T) -> UInt64 {
        let envelope = OkEnvelope(status: "ok", data: value)
        guard let data = try? jsonEncoder.encode(envelope) else { return 0 }
        return writeToGuestMemory(caller: caller, data: data)
    }

    private func writeJsonError(caller: Caller, message: String) -> UInt64 {
        let envelope = ErrEnvelope(
            status: "err",
            data: ErrData(RuntimeFailure: MessageField(message: message))
        )
        guard let data = try? jsonEncoder.encode(envelope) else { return 0 }
        return writeToGuestMemory(caller: caller, data: data)
    }

    private func writeJsonNil(caller: Caller) -> UInt64 {
        let envelope = NilEnvelope(status: "ok", data: nil)
        guard let data = try? jsonEncoder.encode(envelope) else { return 0 }
        return writeToGuestMemory(caller: caller, data: data)
    }

    // MARK: - Guest memory

    private func writeToGuestMemory(caller: Caller, data: Data) -> UInt64 {
        let len = UInt32(data.count)
        guard len > 0 else { return 0 }

        guard let allocFn = caller.instance?.exports[function: "atlas_alloc"] else { return 0 }
        guard let results = try? allocFn.invoke([.i32(len)]) else { return 0 }
        let ptr = results[0].i32

        guard let memory = caller.instance?.exports[memory: "memory"] else { return 0 }
        let bytes = Array(data)
        memory.withUnsafeMutableBufferPointer(offset: UInt(ptr), count: Int(len)) { buffer in
            for idx in 0..<Int(len) {
                buffer[idx] = bytes[idx]
            }
        }

        return (UInt64(ptr) << 32) | UInt64(len)
    }
}

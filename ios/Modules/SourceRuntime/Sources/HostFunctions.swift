// Atlas Platform
import Foundation
@preconcurrency import WasmKit
import MessagePack

/// Implements all Atlas host-import functions for the WasmKit runtime.
final class HostFunctions: @unchecked Sendable {

    let policy: CapabilityPolicy
    private let store: Store
    private var logBuffer: [String] = []
    private var cache: [String: (data: Data, expiry: Date?)] = [:]
    private let session: URLSession

    init(policy: CapabilityPolicy, store: Store, session: URLSession = .shared) {
        self.policy = policy
        self.store = store
        self.session = session
    }

    /// Register all host functions into a WasmKit `Imports` table.
    func register(into imports: inout Imports) {
        // network.fetch(ptr: i32, len: i32) -> i64
        imports.define(
            module: "atlas", name: "host_network_fetch",
            Function(store: store, parameters: [.i32, .i32], results: [.i64]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i64(0)] }
                return [.i64(self.hostNetworkFetch(caller: caller, args: args))]
            }
        )

        // log.debug(ptr: i32, len: i32)
        imports.define(
            module: "atlas", name: "host_log_debug",
            Function(store: store, parameters: [.i32, .i32]) {
                [weak self] caller, args -> [Value] in
                self?.hostLogDebug(caller: caller, args: args)
                return []
            }
        )

        // time.now() -> i64
        imports.define(
            module: "atlas", name: "host_time_now",
            Function(store: store, parameters: [], results: [.i64]) {
                [weak self] _, _ -> [Value] in
                guard let self, (try? self.policy.require("time.now")) != nil else { return [.i64(0)] }
                return [.i64(UInt64(Date().timeIntervalSince1970))]
            }
        )

        // cache.get(ptr: i32, len: i32) -> i64
        imports.define(
            module: "atlas", name: "host_cache_get",
            Function(store: store, parameters: [.i32, .i32], results: [.i64]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i64(0)] }
                return [.i64(self.hostCacheGet(caller: caller, args: args))]
            }
        )

        // cache.set(ptr: i32, len: i32) -> i32
        imports.define(
            module: "atlas", name: "host_cache_set",
            Function(store: store, parameters: [.i32, .i32], results: [.i32]) {
                [weak self] caller, args -> [Value] in
                guard let self else { return [.i32(1)] }
                return [.i32(self.hostCacheSet(caller: caller, args: args))]
            }
        )

        // preferences.get(ptr: i32, len: i32) -> i64
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
        let body: Data?
    }

    private struct FetchResponseWire: Encodable {
        let status: UInt16
        let headers: [String: String]
        let body: Data
    }

    private func hostNetworkFetch(caller: Caller, args: [Value]) -> UInt64 {
        guard (try? policy.require("network.fetch")) != nil else {
            return writeError(caller: caller, message: "capability denied: network.fetch")
        }

        let ptr = Int(args[0].i32)
        let len = Int(args[1].i32)

        guard let memory = caller.instance?.exports[memory: "memory"] else { return 0 }
        let bytes = Array(memory.data[ptr..<(ptr + len)])

        let request: FetchRequestWire
        do {
            request = try MessagePackDecoder().decode(FetchRequestWire.self, from: Data(bytes))
        } catch {
            return writeError(caller: caller, message: "decode FetchRequest: \(error)")
        }

        guard let url = URL(string: request.url) else {
            return writeError(caller: caller, message: "invalid URL: \(request.url)")
        }

        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = request.method.uppercased()
        for (key, value) in request.headers {
            urlRequest.setValue(value, forHTTPHeaderField: key)
        }
        urlRequest.httpBody = request.body
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
            return writeError(caller: caller, message: "network: \(error.localizedDescription)")
        }

        let status = UInt16(responseHTTP?.statusCode ?? 0)
        var headers: [String: String] = [:]
        for (key, value) in responseHTTP?.allHeaderFields ?? [:] {
            headers[String(describing: key).lowercased()] = String(describing: value)
        }

        let response = FetchResponseWire(
            status: status,
            headers: headers,
            body: responseData ?? Data()
        )
        return writeOk(caller: caller, value: response)
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
        let ptr = Int(args[0].i32)
        let len = Int(args[1].i32)
        guard let memory = caller.instance?.exports[memory: "memory"] else {
            return writeOkNil(caller: caller)
        }
        let keyBytes = Array(memory.data[ptr..<(ptr + len)])

        guard let key = try? MessagePackDecoder().decode(String.self, from: Data(keyBytes)) else {
            return writeOkNil(caller: caller)
        }

        if let entry = cache[key] {
            if let expiry = entry.expiry, Date() > expiry {
                cache.removeValue(forKey: key)
                return writeOkNil(caller: caller)
            }
            let value: [UInt8]? = Array(entry.data)
            return writeOk(caller: caller, value: value)
        }

        return writeOkNil(caller: caller)
    }

    private struct CacheEntryWire: Decodable {
        let key: String
        let value: Data
        let ttlSeconds: UInt32?
        enum CodingKeys: String, CodingKey {
            case key, value
            case ttlSeconds = "ttl_seconds"
        }
    }

    private func hostCacheSet(caller: Caller, args: [Value]) -> UInt32 {
        guard (try? policy.require("cache.write")) != nil else { return 1 }
        let ptr = Int(args[0].i32)
        let len = Int(args[1].i32)
        guard let memory = caller.instance?.exports[memory: "memory"] else { return 1 }
        let bytes = Array(memory.data[ptr..<(ptr + len)])

        guard let entry = try? MessagePackDecoder().decode(CacheEntryWire.self, from: Data(bytes)) else {
            return 1
        }

        let expiry: Date? = entry.ttlSeconds.map { Date().addingTimeInterval(Double($0)) }
        cache[entry.key] = (data: entry.value, expiry: expiry)
        return 0
    }

    // MARK: - Preferences

    private func hostPreferencesGet(caller: Caller, args: [Value]) -> UInt64 {
        guard (try? policy.require("preferences.read")) != nil else { return 0 }
        return writeOkNil(caller: caller)
    }

    // MARK: - Memory helpers

    private func writeOk<T: Encodable>(caller: Caller, value: T) -> UInt64 {
        guard let data = try? MessagePackBridge.encodeOk(value) else { return 0 }
        return writeToGuestMemory(caller: caller, data: data)
    }

    private func writeOkNil(caller: Caller) -> UInt64 {
        guard let data = try? MessagePackBridge.encodeOkNil() else { return 0 }
        return writeToGuestMemory(caller: caller, data: data)
    }

    private func writeError(caller: Caller, message: String) -> UInt64 {
        guard let data = try? MessagePackBridge.encodeError(message: message) else { return 0 }
        return writeToGuestMemory(caller: caller, data: data)
    }

    private func writeToGuestMemory(caller: Caller, data: Data) -> UInt64 {
        let len = UInt32(data.count)
        guard len > 0 else { return 0 }

        guard let allocFn = caller.instance?.exports[function: "atlas_alloc"] else { return 0 }
        guard let results = try? allocFn.invoke([.i32(len)]) else { return 0 }
        let ptr = results[0].i32

        guard let memory = caller.instance?.exports[memory: "memory"] else { return 0 }
        let bytes = Array(data)
        // Write bytes to guest memory using withUnsafeMutableBufferPointer
        memory.withUnsafeMutableBufferPointer(offset: UInt(ptr), count: Int(len)) { buffer in
            for i in 0..<Int(len) {
                buffer[i] = bytes[i]
            }
        }

        return (UInt64(ptr) << 32) | UInt64(len)
    }
}

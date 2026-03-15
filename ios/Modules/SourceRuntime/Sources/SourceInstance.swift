// Atlas Platform
import Foundation
@preconcurrency import WasmKit
import MessagePack
import Core

/// A loaded, instantiated source module ready for invocation.
public final class SourceInstance: @unchecked Sendable {

    private let instance: WasmKit.Instance
    private let store: Store
    public let manifest: SourceManifest
    private let hostFunctions: HostFunctions

    init(
        instance: WasmKit.Instance,
        store: Store,
        manifest: SourceManifest,
        hostFunctions: HostFunctions
    ) {
        self.instance = instance
        self.store = store
        self.manifest = manifest
        self.hostFunctions = hostFunctions
    }

    // MARK: - Required contract

    public func getInfo() throws -> SourceInfo {
        try callNoArgs(export: "atlas_get_info")
    }

    public func search(query: SearchQuery) throws -> SearchResponse {
        try callWithArg(export: "atlas_search", arg: query)
    }

    public func getMangaDetails(id: String) throws -> Manga {
        try callWithArg(export: "atlas_get_manga_details", arg: id)
    }

    public func getChapters(mangaId: String) throws -> [Chapter] {
        try callWithArg(export: "atlas_get_chapters", arg: mangaId)
    }

    public func getPages(chapterId: String) throws -> [Page] {
        try callWithArg(export: "atlas_get_pages", arg: chapterId)
    }

    // MARK: - Optional contract

    public func getLatest(page: Int = 1) throws -> SearchResponse {
        try callWithArg(export: "atlas_get_latest", arg: UInt32(page))
    }

    public func getPopular(page: Int = 1) throws -> SearchResponse {
        try callWithArg(export: "atlas_get_popular", arg: UInt32(page))
    }

    public func getFilters() throws -> [Filter] {
        try callNoArgs(export: "atlas_get_filters")
    }

    public func getPreferencesSchema() throws -> PreferenceSchema {
        try callNoArgs(export: "atlas_get_preferences_schema")
    }

    // MARK: - Diagnostics

    public func drainLogs() -> [String] {
        hostFunctions.drainLogs()
    }

    // MARK: - Invocation helpers

    private func callNoArgs<T: Decodable>(export name: String) throws -> T {
        guard let fn = instance.exports[function: name] else {
            throw SourceRuntimeError.exportNotFound(name)
        }

        let results = try fn.invoke([])
        return try decodeResult(packed: results[0].i64, export: name)
    }

    private func callWithArg<A: Encodable, T: Decodable>(
        export name: String,
        arg: A
    ) throws -> T {
        guard let fn = instance.exports[function: name] else {
            throw SourceRuntimeError.exportNotFound(name)
        }

        let (argPtr, argLen) = try writeArg(arg)
        defer { freeGuestBuffer(ptr: argPtr, len: argLen) }

        let results = try fn.invoke([.i32(argPtr), .i32(argLen)])
        return try decodeResult(packed: results[0].i64, export: name)
    }

    private func writeArg<A: Encodable>(_ arg: A) throws -> (UInt32, UInt32) {
        let data = try MessagePackBridge.encode(arg)
        let len = UInt32(data.count)

        guard let allocFn = instance.exports[function: "atlas_alloc"] else {
            throw SourceRuntimeError.memoryError("atlas_alloc not found")
        }

        let allocResult = try allocFn.invoke([.i32(len)])
        let ptr = allocResult[0].i32

        guard let memory = instance.exports[memory: "memory"] else {
            throw SourceRuntimeError.memoryError("no exported memory")
        }

        let bytes = Array(data)
        memory.withUnsafeMutableBufferPointer(offset: UInt(ptr), count: Int(len)) { buffer in
            for i in 0..<Int(len) {
                buffer[i] = bytes[i]
            }
        }

        return (ptr, len)
    }

    private func freeGuestBuffer(ptr: UInt32, len: UInt32) {
        guard let fn = instance.exports[function: "atlas_dealloc"] else { return }
        _ = try? fn.invoke([.i32(ptr), .i32(len)])
    }

    private func decodeResult<T: Decodable>(packed: UInt64, export name: String) throws -> T {
        let ptr = UInt32(packed >> 32)
        let len = UInt32(packed & 0xFFFF_FFFF)

        guard ptr > 0, len > 0 else {
            throw SourceRuntimeError.invalidReturn(name)
        }

        guard let memory = instance.exports[memory: "memory"] else {
            throw SourceRuntimeError.memoryError("cannot read result buffer")
        }

        let bytes = Array(memory.data[Int(ptr)..<Int(ptr + len)])

        // Free result buffer
        freeGuestBuffer(ptr: ptr, len: len)

        return try MessagePackBridge.decodeResult(Data(bytes))
    }
}

// MARK: - Errors

public enum SourceRuntimeError: LocalizedError {
    case exportNotFound(String)
    case memoryError(String)
    case invalidReturn(String)
    case sourceError(String)

    public var errorDescription: String? {
        switch self {
        case .exportNotFound(let n): return "WASM export not found: \(n)"
        case .memoryError(let m):   return "Memory error: \(m)"
        case .invalidReturn(let n): return "Unexpected return type from \(n)"
        case .sourceError(let m):   return "Source error: \(m)"
        }
    }
}

// Atlas Platform
import Foundation
@preconcurrency import WasmKit
import Core

/// The main entry point for the Atlas WASM runtime on iOS.
///
/// Wraps WasmKit (pure Swift WASM interpreter) and the Atlas host-function
/// bridge. Create one per session or share as a singleton.
public final class AtlasRuntime: @unchecked Sendable {

    private let engine: Engine
    private let urlSession: URLSession

    public init(session: URLSession = .shared) {
        self.engine = Engine(configuration: EngineConfiguration(
            compilationMode: .lazy,
            stackSize: 1024 * 1024  // 1MB stack
        ))
        self.urlSession = session
    }

    /// Load a WASM source module and return an invocable `SourceInstance`.
    public func loadSource(
        wasm: Data,
        manifest: SourceManifest
    ) throws -> SourceInstance {
        let store = Store(engine: engine)
        let module = try parseWasm(bytes: Array(wasm))

        let policy = CapabilityPolicy(capabilities: manifest.capabilities)
        let hostFunctions = HostFunctions(policy: policy, store: store, session: urlSession)

        var imports = Imports()
        hostFunctions.register(into: &imports)

        let instance = try module.instantiate(store: store, imports: imports)

        return SourceInstance(
            instance: instance,
            store: store,
            manifest: manifest,
            hostFunctions: hostFunctions
        )
    }
}

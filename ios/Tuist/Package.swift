// swift-tools-version: 5.10
import PackageDescription

#if TUIST
import ProjectDescription

let packageSettings = PackageSettings(
    productTypes: [
        "WasmKit": .framework,
        "MessagePack": .framework,
    ]
)
#endif

let package = Package(
    name: "AtlasDependencies",
    dependencies: [
        // Pure-Swift WASM interpreter — no JIT, iOS W^X compliant.
        .package(url: "https://github.com/swiftwasm/WasmKit.git", from: "0.2.0"),

        // MessagePack Codable encoder/decoder for host-guest communication.
        .package(url: "https://github.com/Flight-School/MessagePack.git", from: "1.2.0"),
    ]
)

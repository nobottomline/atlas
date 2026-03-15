// Atlas Platform
import Foundation

/// Enforces that a source only calls host imports it has declared.
///
/// Capabilities in manifests use underscores (`network_fetch`),
/// while host functions use dots (`network.fetch`). Both are accepted.
public struct CapabilityPolicy: Sendable {

    private let granted: Set<String>

    public init(capabilities: [String]) {
        // Normalize: store both underscore and dot variants
        var all = Set<String>()
        for cap in capabilities {
            all.insert(cap)
            all.insert(cap.replacingOccurrences(of: "_", with: "."))
            all.insert(cap.replacingOccurrences(of: ".", with: "_"))
        }
        self.granted = all
    }

    public func isGranted(_ capability: String) -> Bool {
        granted.contains(capability)
    }

    public func require(_ capability: String) throws {
        guard isGranted(capability) else {
            throw CapabilityError.denied(capability)
        }
    }
}

public enum CapabilityError: LocalizedError {
    case denied(String)

    public var errorDescription: String? {
        if case .denied(let cap) = self {
            return "Capability denied: \(cap)"
        }
        return nil
    }
}

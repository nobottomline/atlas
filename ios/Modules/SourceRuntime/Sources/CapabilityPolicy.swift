import Foundation

/// Enforces that a source only calls host imports it has declared.
public struct CapabilityPolicy: Sendable {

    private let granted: Set<String>

    public init(capabilities: [String]) {
        self.granted = Set(capabilities)
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

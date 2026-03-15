// Atlas Platform
import Foundation
import Core

/// Lightweight URLSession wrapper for the app's own networking needs.
///
/// Sources use the WASM host function `host_network_fetch` for their HTTP.
/// This client is for the app's infrastructure (registry, image loading, etc.).
public actor NetworkClient {

    private let session: URLSession

    public init(session: URLSession = .shared) {
        self.session = session
    }

    public func fetch(_ url: URL) async throws -> (Data, HTTPURLResponse) {
        let (data, response) = try await session.data(from: url)
        guard let http = response as? HTTPURLResponse else {
            throw NetworkError.invalidResponse
        }
        guard (200..<300).contains(http.statusCode) else {
            throw NetworkError.httpError(statusCode: http.statusCode, url: url)
        }
        return (data, http)
    }

    public func fetchJSON<T: Decodable>(_ type: T.Type, from url: URL) async throws -> T {
        let (data, _) = try await fetch(url)
        return try JSONDecoder().decode(type, from: data)
    }

    public func fetchData(from url: URL) async throws -> Data {
        let (data, _) = try await fetch(url)
        return data
    }
}

public enum NetworkError: LocalizedError {
    case invalidResponse
    case httpError(statusCode: Int, url: URL)

    public var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "Invalid response"
        case .httpError(let code, let url):
            return "HTTP \(code) from \(url.host ?? url.absoluteString)"
        }
    }
}

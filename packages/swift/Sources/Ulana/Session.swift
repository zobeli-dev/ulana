import Foundation
import Observation

/// Something that runs a view and hands frames to the screen: a server
/// (`ServerSession`) or the device itself (`LocalSession` in UlanaLocal).
@MainActor
public protocol UlanaSession: AnyObject {
    /// The last frame that rendered. It stays while the script has an error.
    var frame: Frame? { get }
    /// What went wrong last, written to be read by a person or a model.
    var error: String? { get }
    /// Lines the script printed.
    var logs: [String] { get }
    /// Run a handler an `event` action named, then render again.
    func send(version: UInt64, id: String, value: JSONValue)
}

/// What `ulana serve` answers for one view.
public struct Snapshot: Codable, Sendable, Equatable {
    public var name: String
    public var revision: UInt64
    public var frame: Frame?
    public var error: String?
    public var logs: [String]
}

/// Draws views that run on a server speaking the `ulana serve` protocol:
///
/// - `GET  /views` lists the views and their revisions.
/// - `GET  /views/<name>?since=<rev>` answers when the view changes.
/// - `POST /views/<name>/events` with `{ version, id, value }`.
///
/// Set `name` to pick a view; the session follows it until `stop()`.
@MainActor
@Observable
public final class ServerSession: UlanaSession {
    public let baseURL: URL
    /// The views the server has, by name.
    public private(set) var views: [String] = []
    /// The view being followed.
    public var name: String? {
        didSet { if name != oldValue { follow() } }
    }
    public private(set) var snapshot: Snapshot?
    /// Set while the server cannot be reached.
    public private(set) var offline: String?

    public var frame: Frame? { snapshot?.frame }
    public var error: String? { snapshot?.error }
    public var logs: [String] { snapshot?.logs ?? [] }

    @ObservationIgnored private var listing: Task<Void, Never>?
    @ObservationIgnored private var following: Task<Void, Never>?
    @ObservationIgnored private let urlSession: URLSession

    public init(baseURL: URL) {
        self.baseURL = baseURL
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 40  // longer than the server's 25 s wait
        config.requestCachePolicy = .reloadIgnoringLocalCacheData
        urlSession = URLSession(configuration: config)
    }

    /// Start listing views and following `name`.
    public func start() {
        guard listing == nil else { return }
        listing = Task { [weak self] in
            while !Task.isCancelled {
                await self?.list()
                try? await Task.sleep(for: .seconds(2))
            }
        }
        follow()
    }

    public func stop() {
        listing?.cancel()
        following?.cancel()
        listing = nil
        following = nil
    }

    public func send(version: UInt64, id: String, value: JSONValue) {
        guard let name else { return }
        Task {
            do {
                var request = URLRequest(url: viewURL(name).appending(path: "events"))
                request.httpMethod = "POST"
                request.setValue("application/json", forHTTPHeaderField: "content-type")
                request.httpBody = try JSONEncoder().encode(EventBody(version: version, id: id, value: value))
                accept(try await fetch(Snapshot.self, request))
            } catch {
                offline = error.localizedDescription
            }
        }
    }

    private struct ServerMessage: Decodable {
        let error: String
    }

    private struct EventBody: Encodable {
        let version: UInt64
        let id: String
        let value: JSONValue
    }

    private func list() async {
        do {
            struct Listed: Decodable { let name: String }
            let listed = try await fetch([Listed].self, URLRequest(url: baseURL.appending(path: "views")))
            views = listed.map(\.name)
            if name == nil { name = views.first }
            offline = nil
        } catch is CancellationError {
        } catch {
            offline = error.localizedDescription
        }
    }

    private func follow() {
        following?.cancel()
        snapshot = nil
        guard listing != nil, let name else { return }
        following = Task { [weak self] in
            var since: UInt64?
            while !Task.isCancelled {
                guard let self else { return }
                do {
                    var url = self.viewURL(name)
                    if let since { url.append(queryItems: [URLQueryItem(name: "since", value: String(since))]) }
                    let snapshot = try await self.fetch(Snapshot.self, URLRequest(url: url))
                    self.accept(snapshot)
                    since = self.snapshot?.revision
                    self.offline = nil
                } catch is CancellationError {
                    return
                } catch {
                    if Task.isCancelled { return }
                    self.offline = error.localizedDescription
                    try? await Task.sleep(for: .seconds(1))
                }
            }
        }
    }

    /// Keep the newest snapshot of the view being followed.
    private func accept(_ s: Snapshot) {
        guard s.name == name else { return }
        if let current = snapshot, s.revision < current.revision { return }
        snapshot = s
    }

    private func viewURL(_ name: String) -> URL {
        baseURL.appending(path: "views").appending(path: name)
    }

    private func fetch<T: Decodable>(_ type: T.Type, _ request: URLRequest) async throws -> T {
        let (data, response) = try await urlSession.data(for: request)
        if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
            let message = (try? JSONDecoder().decode(ServerMessage.self, from: data))?.error
            throw URLError(.badServerResponse, userInfo: [NSLocalizedDescriptionKey: message ?? "HTTP \(http.statusCode)"])
        }
        return try JSONDecoder().decode(T.self, from: data)
    }
}

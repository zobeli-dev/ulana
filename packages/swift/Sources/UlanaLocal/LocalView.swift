import Foundation
import Ulana
import UlanaCore

/// Why the runtime refused a call. `message` is written to be read by a
/// person or handed back to the model that wrote the script.
public struct UlanaError: Error, Sendable, Equatable, CustomStringConvertible {
    /// `syntax`, `runtime`, `memory`, `timeout`, `invalid`, `stale`,
    /// `unknown_event`, `argument`, or `panic`.
    public let kind: String
    public let message: String
    public var description: String { message }
}

/// What the app accepts beyond the built-in blocks.
public struct HostSpec: Codable, Sendable, Equatable {
    /// Names a view may give `ui.custom`, such as `calendar`.
    public var blocks: [String]
    /// Names a view may give `ui.host`, such as `send`.
    public var actions: [String]

    public init(blocks: [String] = [], actions: [String] = []) {
        self.blocks = blocks
        self.actions = actions
    }
}

/// One view running on this device: the Rust runtime with Luau in its
/// sandbox (no network or files, read-only libraries, 16 MB and 100 ms per
/// call), and the same checks as on a server.
///
/// Not thread-safe: call it from one thread or actor at a time.
/// `LocalSession` wraps it for SwiftUI on the main actor.
public final class LocalView {
    private let handle: OpaquePointer

    /// Load a script, with state saved earlier if there is any.
    public init(source: String, host: HostSpec = HostSpec(), state: [String: JSONValue] = [:]) throws {
        let hostJSON = try String(decoding: JSONEncoder().encode(host), as: UTF8.self)
        let stateJSON = try String(decoding: JSONEncoder().encode(state), as: UTF8.self)
        var errorOut: UnsafeMutablePointer<CChar>?
        guard let handle = ulana_view_new(source, hostJSON, stateJSON, &errorOut) else {
            throw Self.decodeError(Self.take(errorOut))
        }
        self.handle = handle
    }

    deinit {
        ulana_view_free(handle)
    }

    /// Run the view with `data` and check the tree it returns.
    public func render(data: JSONValue = .null) throws -> Frame {
        try call(Frame.self) { ulana_view_render(handle, try json(data)) }
    }

    /// Run the handler an `event` action named in frame `version`. Render
    /// again afterwards to see the change.
    public func event(version: UInt64, id: String, value: JSONValue = .null) throws {
        _ = try call(JSONValue.self) { ulana_view_event(handle, version, id, try json(value)) }
    }

    /// Swap in a new script and keep the state. On an error the old script stays.
    public func reload(source: String) throws {
        _ = try call(JSONValue.self) { ulana_view_reload(handle, source) }
    }

    /// The view's state, to save and pass back to `init` later.
    public func state() throws -> [String: JSONValue] {
        try call([String: JSONValue].self) { ulana_view_state(handle) }
    }

    /// What the script printed since the last call.
    public func takeLogs() throws -> [String] {
        try call([String].self) { ulana_view_take_logs(handle) }
    }

    private func json(_ value: JSONValue) throws -> String {
        String(decoding: try JSONEncoder().encode(value), as: UTF8.self)
    }

    private struct Envelope<T: Decodable>: Decodable {
        let ok: T?
        let error: Failure?
    }

    private struct Failure: Decodable {
        let kind: String
        let message: String
    }

    private func call<T: Decodable>(_ type: T.Type, _ body: () throws -> UnsafeMutablePointer<CChar>?) throws -> T {
        let text = Self.take(try body())
        let envelope = try JSONDecoder().decode(Envelope<T>.self, from: Data(text.utf8))
        if let failure = envelope.error {
            throw UlanaError(kind: failure.kind, message: failure.message)
        }
        if let ok = envelope.ok { return ok }
        // `{"ok": null}` decodes as nil for an optional; JSONValue spells it `.null`.
        if let null = JSONValue.null as? T { return null }
        throw UlanaError(kind: "internal", message: "the runtime answered without a value")
    }

    private static func take(_ pointer: UnsafeMutablePointer<CChar>?) -> String {
        guard let pointer else { return #"{"error":{"kind":"internal","message":"no answer"}}"# }
        defer { ulana_string_free(pointer) }
        return String(cString: pointer)
    }

    private static func decodeError(_ text: String) -> UlanaError {
        struct Wrapped: Decodable { let error: Failure }
        guard let wrapped = try? JSONDecoder().decode(Wrapped.self, from: Data(text.utf8)) else {
            return UlanaError(kind: "internal", message: text)
        }
        return UlanaError(kind: wrapped.error.kind, message: wrapped.error.message)
    }
}

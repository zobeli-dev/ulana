import Foundation
import Observation
import Ulana

/// Runs one view on the device for SwiftUI.
///
/// It renders at once, after every event, and after `reload`. A script with
/// an error keeps its last good frame and reports the error, like a server
/// does. Calls run on the main actor; each is capped at 100 ms and a
/// typical one takes well under a millisecond.
///
/// ```swift
/// @State var session = try LocalSession(source: script, host: HostSpec(blocks: ["calendar"]))
/// UlanaView(frame: session.frame!, onEvent: session.send)
/// ```
@MainActor
@Observable
public final class LocalSession: UlanaSession {
    public private(set) var frame: Frame?
    public private(set) var error: String?
    public private(set) var logs: [String] = []
    /// The data the view's function gets. Setting it renders again.
    public var data: JSONValue {
        didSet { render() }
    }
    /// Called with the state after it may have changed, to save it.
    @ObservationIgnored public var onState: (([String: JSONValue]) -> Void)?

    @ObservationIgnored private var view: LocalView?
    @ObservationIgnored private let host: HostSpec
    @ObservationIgnored private var savedState: [String: JSONValue]

    /// Load a script. A script that does not load leaves `frame` empty and
    /// `error` set; `reload` can fix it.
    public init(source: String, host: HostSpec = HostSpec(), data: JSONValue = .null, state: [String: JSONValue] = [:]) {
        self.host = host
        self.data = data
        savedState = state
        reload(source: source)
    }

    public func send(version: UInt64, id: String, value: JSONValue) {
        guard let view else { return }
        do {
            try view.event(version: version, id: id, value: value)
            render()
        } catch let e as UlanaError where e.kind == "stale" {
            // The screen had an old frame; the latest is already showing.
        } catch {
            fail(error)
        }
    }

    /// Swap in a new script and keep the state.
    public func reload(source: String) {
        do {
            if let view {
                try view.reload(source: source)
            } else {
                view = try LocalView(source: source, host: host, state: savedState)
            }
            render()
        } catch {
            fail(error)
        }
    }

    /// Forget the state and start the script again.
    public func reset(source: String) {
        view = nil
        savedState = [:]
        frame = nil
        logs = []
        reload(source: source)
    }

    /// The view's state, to save.
    public var state: [String: JSONValue] {
        (try? view?.state()) ?? savedState
    }

    private func render() {
        guard let view else { return }
        do {
            frame = try view.render(data: data)
            error = nil
        } catch {
            fail(error)
        }
        takeLogs()
        if let state = try? view.state() {
            savedState = state
            onState?(state)
        }
    }

    private func fail(_ e: Error) {
        error = (e as? UlanaError)?.message ?? e.localizedDescription
        takeLogs()
    }

    private func takeLogs() {
        guard let lines = try? view?.takeLogs(), !lines.isEmpty else { return }
        logs = Array((logs + lines).suffix(50))
    }
}

import SwiftUI

/// Draws whatever a session (`ServerSession`, or `LocalSession` from
/// UlanaLocal) has: its frame, a note while the first one loads, and the
/// script's error above the last good frame.
///
/// ```swift
/// @State var session = ServerSession(baseURL: URL(string: "http://127.0.0.1:8790")!)
///
/// var body: some View {
///     ScrollView { UlanaSessionView(session: session).padding() }
///         .onAppear { session.start(); session.name = "counter" }
/// }
/// ```
public struct UlanaSessionView: View {
    let session: any UlanaSession
    let theme: UlanaTheme
    let showsErrors: Bool
    let onHostAction: (_ name: String, _ value: JSONValue) -> Void
    let blocks: [String: (CustomBlockProps) -> AnyView]

    /// - Parameter showsErrors: show the script's error above the view. Turn it
    ///   off in production to show only the last frame that worked.
    public init(
        session: any UlanaSession,
        theme: UlanaTheme = .system,
        showsErrors: Bool = true,
        onHostAction: @escaping (_ name: String, _ value: JSONValue) -> Void = { _, _ in },
        blocks: [String: (CustomBlockProps) -> AnyView] = [:]
    ) {
        self.session = session
        self.theme = theme
        self.showsErrors = showsErrors
        self.onHostAction = onHostAction
        self.blocks = blocks
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: theme.space(.md)) {
            if showsErrors, let error = session.error {
                VStack(alignment: .leading, spacing: 4) {
                    Text("The view has an error").font(.subheadline.weight(.semibold))
                    Text(error).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                }
                .foregroundStyle(theme.tone(.danger).fg)
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(theme.tone(.danger).soft, in: RoundedRectangle(cornerRadius: 12))
                .accessibilityElement(children: .combine)
            }
            if let frame = session.frame {
                UlanaView(
                    frame: frame,
                    theme: theme,
                    onEvent: { version, id, value in session.send(version: version, id: id, value: value) },
                    onHostAction: onHostAction,
                    blocks: blocks
                )
            } else if session.error == nil {
                ProgressView().frame(maxWidth: .infinity)
            }
        }
    }
}

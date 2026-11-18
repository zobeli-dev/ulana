// Views running on a server that speaks the `ulana serve` protocol.

import SwiftUI
import Ulana

struct ServerScreen: View {
    /// The simulator reaches this Mac on 127.0.0.1. A phone needs the Mac's
    /// address on the network and `ulana serve --bind 0.0.0.0:8790`.
    @AppStorage("serverURL") private var address = "http://127.0.0.1:8790"
    @State private var session: ServerSession?
    @State private var draft = ""

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                TextField("Server", text: $draft)
                    .textFieldStyle(.roundedBorder)
                    .keyboardType(.URL)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .onSubmit(connect)
                Button("Connect", action: connect)
                    .buttonStyle(.bordered)
                    .disabled(draft == session?.baseURL.absoluteString)
            }
            .padding(.horizontal)
            .padding(.bottom, 8)

            if let session {
                if let offline = session.offline {
                    Banner(
                        title: "Cannot reach the server",
                        text: "\(offline)\n\nStart it with:\ncargo run -p ulana-cli -- serve examples/views --block calendar --action send",
                        tint: .orange
                    )
                    .padding(.horizontal)
                }
                ViewTabs(names: session.views, picked: session.name ?? "") { session.name = $0 }
                if session.name != nil {
                    SessionView(session: session, footer: "Runs on \(session.baseURL.host() ?? "the server")")
                } else {
                    Spacer()
                }
            }
        }
        .onAppear {
            draft = address
            if session == nil { connect() }
        }
        .onDisappear { session?.stop() }
    }

    private func connect() {
        guard let url = URL(string: draft.trimmingCharacters(in: .whitespaces)), url.scheme != nil else { return }
        address = url.absoluteString
        draft = address
        session?.stop()
        let next = ServerSession(baseURL: url)
        next.start()
        session = next
    }
}

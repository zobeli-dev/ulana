// Views running on the device. The bundled scripts can be edited in the app;
// edits and each view's state are kept in UserDefaults.

import SwiftUI
import Ulana
import UlanaLocal

/// The scripts bundled with the app (examples/views), and the edits made to them.
@MainActor
@Observable
final class Library {
    private(set) var names: [String] = []
    /// Views whose script was edited on this iPhone.
    private(set) var edited: Set<String> = []
    private var originals: [String: String] = [:]
    private var data: [String: JSONValue] = [:]
    private var sessions: [String: LocalSession] = [:]
    private let defaults = UserDefaults.standard
    /// What this app accepts beyond the built-in blocks, the same as `ulana serve --block calendar --action send`.
    private let host = HostSpec(blocks: ["calendar"], actions: ["send"])

    init() {
        guard let folder = Bundle.main.url(forResource: "views", withExtension: nil),
              let files = try? FileManager.default.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil)
        else { return }
        for file in files where file.pathExtension == "luau" {
            let name = file.deletingPathExtension().lastPathComponent
            originals[name] = (try? String(contentsOf: file, encoding: .utf8)) ?? ""
            let json = try? String(contentsOf: file.deletingPathExtension().appendingPathExtension("json"), encoding: .utf8)
            data[name] = (try? JSONValue.parse(json)) ?? .null
        }
        names = originals.keys.sorted()
        edited = Set(names.filter { defaults.string(forKey: "source.\($0)") != nil })
    }

    func source(_ name: String) -> String {
        defaults.string(forKey: "source.\(name)") ?? originals[name] ?? ""
    }

    func isEdited(_ name: String) -> Bool {
        edited.contains(name)
    }

    func original(_ name: String) -> String {
        originals[name] ?? ""
    }

    /// The view's session, made on first use with its saved state.
    func session(_ name: String) -> LocalSession {
        if let session = sessions[name] { return session }
        let saved = defaults.data(forKey: "state.\(name)")
            .flatMap { try? JSONDecoder().decode([String: JSONValue].self, from: $0) } ?? [:]
        let session = LocalSession(source: source(name), host: host, data: data[name] ?? .null, state: saved)
        session.onState = { [defaults] state in
            defaults.set(try? JSONEncoder().encode(state), forKey: "state.\(name)")
        }
        sessions[name] = session
        return session
    }

    /// Keep an edit (or forget it, with nil) and reload the view.
    func setSource(_ name: String, _ source: String?) {
        if let source, source != originals[name] {
            defaults.set(source, forKey: "source.\(name)")
            edited.insert(name)
        } else {
            defaults.removeObject(forKey: "source.\(name)")
            edited.remove(name)
        }
        session(name).reload(source: self.source(name))
    }

    func reset(_ name: String) {
        session(name).reset(source: source(name))
    }
}

struct LocalScreen: View {
    @State private var library = Library()
    @AppStorage("localView") private var picked = "counter"
    @State private var editing = false

    var body: some View {
        let name = library.names.contains(picked) ? picked : (library.names.first ?? "")
        VStack(spacing: 0) {
            ViewTabs(names: library.names, picked: name) { picked = $0 }
            if name.isEmpty {
                ContentUnavailableView("No views", systemImage: "square.dashed", description: Text("The app has no bundled views."))
            } else {
                SessionView(session: library.session(name), footer: library.isEdited(name) ? "Edited on this iPhone" : "Runs on this iPhone")
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                Menu {
                    Button("Edit script", systemImage: "chevron.left.forwardslash.chevron.right") { editing = true }
                    Button("Reset state", systemImage: "arrow.counterclockwise") { library.reset(name) }
                    if library.isEdited(name) {
                        Button("Undo my edits", systemImage: "arrow.uturn.backward", role: .destructive) {
                            library.setSource(name, nil)
                        }
                    }
                } label: {
                    Label("View", systemImage: "ellipsis.circle")
                }
                .disabled(name.isEmpty)
            }
        }
        .sheet(isPresented: $editing) {
            ScriptEditor(name: name, library: library)
        }
    }
}

/// Edit a script and see it reload. An error is shown here and the screen
/// keeps its last good frame.
struct ScriptEditor: View {
    let name: String
    let library: Library
    @State private var text = ""
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        let session = library.session(name)
        NavigationStack {
            VStack(alignment: .leading, spacing: 8) {
                if let error = session.error {
                    Banner(title: "The view has an error", text: error, tint: .red)
                }
                TextEditor(text: $text)
                    .font(.system(.footnote, design: .monospaced))
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)
                    .scrollContentBackground(.hidden)
                    .padding(8)
                    .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 12))
                Text("Luau runs in a sandbox on this iPhone: no network, no files, 100 ms and 16 MB per call.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .padding()
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle("\(name).luau")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Done") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Run") { library.setSource(name, text) }
                        .disabled(text == library.source(name) && session.error == nil)
                }
            }
        }
        .onAppear { text = library.source(name) }
    }
}

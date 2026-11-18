// Ulana's example iPhone app: the same views, run two ways.
//
// - On this iPhone: the views bundled with the app run in the Rust runtime
//   on the device. Edit a script in the app and it reloads at once.
// - On a server: the app draws whatever `ulana serve` runs, and redraws a
//   view as soon as its file is saved.

import SwiftUI

@main
struct UlanaExampleApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

enum Mode: String, CaseIterable, Identifiable {
    case local, server
    var id: Self { self }
    var title: String {
        switch self {
        case .local: "On this iPhone"
        case .server: "On a server"
        }
    }
}

struct ContentView: View {
    @AppStorage("mode") private var mode: Mode = .local

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Picker("Where views run", selection: $mode) {
                    ForEach(Mode.allCases) { Text($0.title).tag($0) }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal)
                .padding(.bottom, 8)
                switch mode {
                case .local: LocalScreen()
                case .server: ServerScreen()
                }
            }
            .navigationTitle("Ulana")
            .navigationBarTitleDisplayMode(.inline)
            .background(Color(uiColor: .systemGroupedBackground))
        }
    }
}

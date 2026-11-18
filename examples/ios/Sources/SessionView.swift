// What both modes share: the view tabs, the drawn view, its error and print
// output, a stand-in for an app's own block, and host actions.

import SwiftUI
import Ulana

/// Draws one session's frame, whichever side runs the script.
struct SessionView: View {
    let session: any UlanaSession
    let footer: String
    @State private var hostAction: String?
    @State private var showLogs = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                UlanaSessionView(
                    session: session,
                    onHostAction: { name, value in show("\(name): \(value.display)") },
                    blocks: ["calendar": { props in AnyView(CalendarStandIn(props: props)) }]
                )
                if !session.logs.isEmpty {
                    DisclosureGroup("Print output (\(session.logs.count))", isExpanded: $showLogs) {
                        Text(session.logs.joined(separator: "\n"))
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                }
                Text(footer).font(.caption).foregroundStyle(.tertiary)
            }
            .padding()
        }
        .overlay(alignment: .bottom) {
            if let hostAction {
                Text("Host action · \(hostAction)")
                    .font(.subheadline)
                    .foregroundStyle(.white)
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(.black.opacity(0.85), in: RoundedRectangle(cornerRadius: 12))
                    .padding()
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                    .accessibilityAddTraits(.updatesFrequently)
            }
        }
        .animation(.snappy, value: hostAction)
    }

    private func show(_ text: String) {
        hostAction = text
        Task {
            try? await Task.sleep(for: .seconds(3))
            if hostAction == text { hostAction = nil }
        }
    }
}

/// A row of view names to pick from.
struct ViewTabs: View {
    let names: [String]
    let picked: String
    let pick: (String) -> Void

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(names, id: \.self) { name in
                    Button(name) { pick(name) }
                        .buttonStyle(.bordered)
                        .buttonBorderShape(.capsule)
                        .tint(name == picked ? .primary : .secondary)
                        .fontWeight(name == picked ? .semibold : .regular)
                        .accessibilityAddTraits(name == picked ? .isSelected : [])
                }
            }
            .padding(.horizontal)
        }
        .padding(.bottom, 4)
    }
}

struct Banner: View {
    let title: String
    let text: String
    let tint: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.subheadline.weight(.semibold))
            Text(text).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
        }
        .foregroundStyle(tint)
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(tint.opacity(0.12), in: RoundedRectangle(cornerRadius: 12))
        .accessibilityElement(children: .combine)
    }
}

/// Stands in for an app's own calendar, to show a `custom` block.
struct CalendarStandIn: View {
    let props: CustomBlockProps

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Label("Calendar block from the app", systemImage: "calendar")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.tint)
            Text(JSONValue.object(props.props).display)
                .font(.system(.caption, design: .monospaced))
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.accentColor.opacity(0.1), in: RoundedRectangle(cornerRadius: 12))
        .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(Color.accentColor.opacity(0.5), style: StrokeStyle(lineWidth: 1, dash: [4])))
    }
}

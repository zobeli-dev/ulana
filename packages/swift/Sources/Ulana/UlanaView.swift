import SwiftUI

/// What a host view for a `custom` block receives.
public struct CustomBlockProps {
    public let name: String
    public let props: [String: JSONValue]
    public let theme: UlanaTheme
}

/// Draws a frame with SwiftUI's own views.
///
/// ```swift
/// UlanaView(frame: frame,
///           onEvent: { version, id, value in session.send(version: version, id: id, value: value) },
///           onHostAction: { name, value in if name == "send" { chat.send(value.display) } },
///           blocks: ["calendar": { props in AnyView(CalendarView(props.props)) }])
/// ```
public struct UlanaView: View {
    let frame: Frame
    let context: RenderContext

    public init(
        frame: Frame,
        theme: UlanaTheme = .system,
        onEvent: @escaping (_ version: UInt64, _ id: String, _ value: JSONValue) -> Void = { _, _, _ in },
        onHostAction: @escaping (_ name: String, _ value: JSONValue) -> Void = { _, _ in },
        blocks: [String: (CustomBlockProps) -> AnyView] = [:]
    ) {
        self.frame = frame
        let version = frame.version
        context = RenderContext(
            theme: theme,
            run: { action, value in
                switch action {
                case .event(let id): onEvent(version, id, value)
                case .host(let name, let hostValue): onHostAction(name, hostValue)
                }
            },
            blocks: blocks
        )
    }

    public var body: some View {
        BlockView(block: frame.root, context: context)
    }
}

struct RenderContext {
    let theme: UlanaTheme
    let run: (Action, JSONValue) -> Void
    let blocks: [String: (CustomBlockProps) -> AnyView]
}

struct BlockView: View {
    let block: Block
    let context: RenderContext

    private var theme: UlanaTheme { context.theme }

    var body: some View {
        switch block {
        case .stack(let b):
            let align = b.align ?? .stretch
            VStack(alignment: align.horizontal, spacing: theme.space(b.gap)) {
                ForEach(Array(b.children.enumerated()), id: \.offset) { _, child in
                    if align == .stretch {
                        BlockView(block: child, context: context).frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        BlockView(block: child, context: context)
                    }
                }
            }
            .padding(theme.space(b.padding))
        case .row(let b):
            RowView(block: b, context: context)
        case .text(let b):
            Text(b.text)
                .font(theme.font(b.style))
                .foregroundStyle(theme.tone(b.tone).fg)
                .lineLimit(b.lines)
                .accessibilityAddTraits(b.style == .title || b.style == .heading ? .isHeader : [])
        case .card(let b):
            VStack(alignment: .leading, spacing: theme.space(.sm)) {
                if let title = b.title {
                    Text(title).font(theme.font(.heading)).accessibilityAddTraits(.isHeader)
                }
                ForEach(Array(b.children.enumerated()), id: \.offset) { _, child in
                    BlockView(block: child, context: context).frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .padding(theme.space(.md))
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(theme.surface, in: RoundedRectangle(cornerRadius: theme.cardRadius, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: theme.cardRadius, style: .continuous).stroke(theme.border, lineWidth: 0.5))
        case .list(let b):
            VStack(spacing: 0) {
                ForEach(Array(b.children.enumerated()), id: \.offset) { i, child in
                    if i > 0 { Divider() }
                    BlockView(block: child, context: context).frame(maxWidth: .infinity, alignment: .leading)
                }
            }
        case .item(let b):
            ItemView(block: b, context: context)
        case .button(let b):
            ButtonView(block: b, context: context)
        case .toggle(let b):
            ToggleView(block: b, context: context)
        case .progress(let b):
            VStack(alignment: .leading, spacing: theme.space(.xs)) {
                if let label = b.label {
                    Text(label).font(theme.font(.caption)).foregroundStyle(.secondary)
                }
                ProgressView(value: min(1, max(0, b.value)))
                    .tint(theme.tone(b.tone ?? .accent).fg)
            }
            .accessibilityElement(children: .combine)
        case .badge(let b):
            let tone = theme.tone(b.tone)
            Text(b.text)
                .font(theme.font(.caption).weight(.medium))
                .foregroundStyle(tone.fg)
                .padding(.horizontal, theme.space(.sm))
                .padding(.vertical, 2)
                .background(tone.soft, in: Capsule())
        case .divider:
            Divider()
        case .spacer(let b):
            if let size = b.size {
                Color.clear.frame(width: theme.space(size), height: theme.space(size))
            } else {
                Spacer(minLength: 0)
            }
        case .custom(let b):
            if let make = context.blocks[b.name] {
                make(CustomBlockProps(name: b.name, props: b.props, theme: theme))
            } else {
                Unknown(kind: b.name)
            }
        case .unknown(let type):
            Unknown(kind: type)
        }
    }
}

struct RowView: View {
    let block: RowBlock
    let context: RenderContext

    var body: some View {
        let theme = context.theme
        let children = Array(block.children.enumerated())
        let gap = theme.space(block.gap ?? .sm)
        Group {
            if block.wrap == true {
                FlowLayout(spacing: gap) {
                    ForEach(children, id: \.offset) { _, child in BlockView(block: child, context: context) }
                }
            } else {
                HStack(alignment: (block.align ?? .center).vertical, spacing: gap) {
                    let justify = block.justify ?? .start
                    if justify == .end || justify == .center { Spacer(minLength: 0) }
                    ForEach(children, id: \.offset) { i, child in
                        if justify == .between && i > 0 { Spacer(minLength: 0) }
                        BlockView(block: child, context: context)
                    }
                    if justify == .start || justify == .center { Spacer(minLength: 0) }
                }
            }
        }
        .padding(theme.space(block.padding))
    }
}

struct ItemView: View {
    let block: ItemBlock
    let context: RenderContext

    var body: some View {
        let theme = context.theme
        let row = HStack(spacing: theme.space(.sm)) {
            VStack(alignment: .leading, spacing: 2) {
                Text(block.title).font(theme.font(.body)).foregroundStyle(.primary)
                if let subtitle = block.subtitle {
                    Text(subtitle).font(theme.font(.caption)).foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: 0)
            if let trailing = block.trailing {
                Text(trailing).font(theme.font(.body)).foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, theme.space(.sm))
        .frame(minHeight: theme.hitSize)
        .contentShape(Rectangle())
        if let action = block.action {
            Button { context.run(action, .null) } label: { row }.buttonStyle(.plain)
        } else {
            row.accessibilityElement(children: .combine)
        }
    }
}

struct ButtonView: View {
    let block: ButtonBlock
    let context: RenderContext

    var body: some View {
        let button = Button(block.label) { context.run(block.action, .null) }
            .controlSize(.large)
            .tint(context.theme.tone(.accent).fg)
        switch block.style ?? .primary {
        case .primary: button.buttonStyle(.borderedProminent)
        case .secondary: button.buttonStyle(.bordered)
        case .plain: button.buttonStyle(.borderless)
        }
    }
}

struct ToggleView: View {
    let block: ToggleBlock
    let context: RenderContext
    /// Shows the new value at once; the next frame confirms or corrects it.
    @State private var shown: Bool?

    var body: some View {
        Toggle(block.label, isOn: Binding(
            get: { shown ?? block.value },
            set: { next in
                guard let action = block.action else { return }
                shown = next
                context.run(action, .bool(next))
            }
        ))
        .disabled(block.action == nil)
        .tint(context.theme.tone(.accent).fg)
        .frame(minHeight: context.theme.hitSize)
        .onChange(of: block.value) { shown = nil }
    }
}

struct Unknown: View {
    let kind: String
    var body: some View {
        Text("This app cannot show “\(kind)” yet.").font(.footnote).foregroundStyle(.secondary)
    }
}

/// Lays children out left to right and wraps them onto new lines.
struct FlowLayout: Layout {
    var spacing: CGFloat

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? .infinity
        var x: CGFloat = 0, y: CGFloat = 0, line: CGFloat = 0, widest: CGFloat = 0
        for view in subviews {
            let size = view.sizeThatFits(.unspecified)
            if x > 0 && x + size.width > width {
                y += line + spacing
                x = 0
                line = 0
            }
            x += size.width + spacing
            line = max(line, size.height)
            widest = max(widest, x - spacing)
        }
        return CGSize(width: min(widest, width), height: y + line)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        var x = bounds.minX, y = bounds.minY, line: CGFloat = 0
        for view in subviews {
            let size = view.sizeThatFits(.unspecified)
            if x > bounds.minX && x + size.width > bounds.maxX {
                y += line + spacing
                x = bounds.minX
                line = 0
            }
            view.place(at: CGPoint(x: x, y: y), proposal: ProposedViewSize(size))
            x += size.width + spacing
            line = max(line, size.height)
        }
    }
}

extension Token.Align {
    var horizontal: HorizontalAlignment {
        switch self {
        case .start, .stretch: .leading
        case .center: .center
        case .end: .trailing
        }
    }

    var vertical: VerticalAlignment {
        switch self {
        case .start: .top
        case .center, .stretch: .center
        case .end: .bottom
        }
    }
}

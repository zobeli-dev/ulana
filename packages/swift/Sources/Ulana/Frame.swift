// The frame a view renders. Mirrors crates/ulana/src/block.rs; the JSON
// Schema in spec/frame.schema.json is generated from those types, and
// Tests/UlanaTests/SchemaTests.swift fails when this file drifts from it.

import Foundation

/// One rendered tree and the version its events must name.
public struct Frame: Codable, Sendable, Equatable {
    public var version: UInt64
    public var root: Block

    public init(version: UInt64, root: Block) {
        self.version = version
        self.root = root
    }
}

/// Tokens a view names. The host's theme decides how they look.
public enum Token {
    public enum Space: String, Codable, Sendable, CaseIterable { case none, xs, sm, md, lg, xl }
    public enum Align: String, Codable, Sendable, CaseIterable { case start, center, end, stretch }
    public enum Justify: String, Codable, Sendable, CaseIterable { case start, center, end, between }
    public enum TextStyle: String, Codable, Sendable, CaseIterable { case display, title, heading, body, caption, mono }
    public enum Tone: String, Codable, Sendable, CaseIterable { case `default`, muted, accent, success, warning, danger }
    public enum ButtonStyle: String, Codable, Sendable, CaseIterable { case primary, secondary, plain }
    public enum FieldKind: String, Codable, Sendable, CaseIterable { case text, number, email, search }
    /// The icons a view may name. `systemName` maps each to an SF Symbol.
    public enum Icon: String, Codable, Sendable, CaseIterable {
        case add, remove, close, check
        case chevronLeft = "chevron_left", chevronRight = "chevron_right"
        case search, settings, info, warning, error, star, heart, bookmark, calendar, clock, location
        case person, people, mail, phone, message, home, bell, sun, moon, cloud, rain, snow, wind, bolt
        case cart, gift, music, camera, photo, link, share, trash, edit, refresh, lock, flag, play, pause
    }
}

/// What a press or a change does.
public enum Action: Codable, Sendable, Equatable {
    /// Run the view's own handler, then render again.
    case event(id: String)
    /// Ask the host app to do something it named, such as sending a message.
    case host(name: String, value: JSONValue)

    private enum Keys: String, CodingKey { case kind, id, name, value }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: Keys.self)
        switch try c.decode(String.self, forKey: .kind) {
        case "event": self = .event(id: try c.decode(String.self, forKey: .id))
        case "host":
            self = .host(
                name: try c.decode(String.self, forKey: .name),
                value: try c.decodeIfPresent(JSONValue.self, forKey: .value) ?? .null)
        case let other:
            throw DecodingError.dataCorruptedError(forKey: .kind, in: c, debugDescription: "unknown action kind \(other)")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: Keys.self)
        switch self {
        case .event(let id):
            try c.encode("event", forKey: .kind)
            try c.encode(id, forKey: .id)
        case .host(let name, let value):
            try c.encode("host", forKey: .kind)
            try c.encode(name, forKey: .name)
            try c.encode(value, forKey: .value)
        }
    }
}

public struct StackBlock: Codable, Sendable, Equatable {
    public var gap: Token.Space?
    public var padding: Token.Space?
    public var align: Token.Align?
    public var children: [Block]

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        gap = try c.decodeIfPresent(Token.Space.self, forKey: .gap)
        padding = try c.decodeIfPresent(Token.Space.self, forKey: .padding)
        align = try c.decodeIfPresent(Token.Align.self, forKey: .align)
        children = try c.decodeIfPresent([Block].self, forKey: .children) ?? []
    }
}

public struct RowBlock: Codable, Sendable, Equatable {
    public var gap: Token.Space?
    public var padding: Token.Space?
    public var align: Token.Align?
    public var justify: Token.Justify?
    public var wrap: Bool?
    public var children: [Block]

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        gap = try c.decodeIfPresent(Token.Space.self, forKey: .gap)
        padding = try c.decodeIfPresent(Token.Space.self, forKey: .padding)
        align = try c.decodeIfPresent(Token.Align.self, forKey: .align)
        justify = try c.decodeIfPresent(Token.Justify.self, forKey: .justify)
        wrap = try c.decodeIfPresent(Bool.self, forKey: .wrap)
        children = try c.decodeIfPresent([Block].self, forKey: .children) ?? []
    }
}

public struct TextBlock: Codable, Sendable, Equatable {
    public var text: String
    public var style: Token.TextStyle?
    public var tone: Token.Tone?
    public var lines: Int?
}

public struct CardBlock: Codable, Sendable, Equatable {
    public var title: String?
    public var tone: Token.Tone?
    public var padding: Token.Space?
    public var children: [Block]

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        title = try c.decodeIfPresent(String.self, forKey: .title)
        tone = try c.decodeIfPresent(Token.Tone.self, forKey: .tone)
        padding = try c.decodeIfPresent(Token.Space.self, forKey: .padding)
        children = try c.decodeIfPresent([Block].self, forKey: .children) ?? []
    }
}

/// Children in equal columns, row by row.
public struct GridBlock: Codable, Sendable, Equatable {
    public var columns: Int
    public var gap: Token.Space?
    public var children: [Block]

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        columns = try c.decode(Int.self, forKey: .columns)
        gap = try c.decodeIfPresent(Token.Space.self, forKey: .gap)
        children = try c.decodeIfPresent([Block].self, forKey: .children) ?? []
    }
}

public struct ListBlock: Codable, Sendable, Equatable {
    public var children: [Block]

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        children = try c.decodeIfPresent([Block].self, forKey: .children) ?? []
    }
}

public struct ItemBlock: Codable, Sendable, Equatable {
    public var title: String
    public var icon: Token.Icon?
    public var subtitle: String?
    public var trailing: String?
    public var action: Action?
}

public struct ButtonBlock: Codable, Sendable, Equatable {
    public var label: String
    public var action: Action
    public var icon: Token.Icon?
    public var style: Token.ButtonStyle?
}

public struct ToggleBlock: Codable, Sendable, Equatable {
    public var label: String
    public var value: Bool
    public var action: Action?
}

/// A text input. `action` gets the text after each change, `submit` when return is pressed.
public struct FieldBlock: Codable, Sendable, Equatable {
    public var value: String
    public var label: String?
    public var placeholder: String?
    public var kind: Token.FieldKind?
    public var action: Action?
    public var submit: Action?

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        value = try c.decodeIfPresent(String.self, forKey: .value) ?? ""
        label = try c.decodeIfPresent(String.self, forKey: .label)
        placeholder = try c.decodeIfPresent(String.self, forKey: .placeholder)
        kind = try c.decodeIfPresent(Token.FieldKind.self, forKey: .kind)
        action = try c.decodeIfPresent(Action.self, forKey: .action)
        submit = try c.decodeIfPresent(Action.self, forKey: .submit)
    }
}

/// One of 2 to 8 options, side by side.
public struct ChoiceBlock: Codable, Sendable, Equatable {
    public var options: [String]
    public var value: String
    public var label: String?
    public var action: Action?
}

public struct IconBlock: Codable, Sendable, Equatable {
    public var name: Token.Icon
    public var tone: Token.Tone?
    public var style: Token.TextStyle?
    public var label: String?
}

/// A picture from an address the host allows. `aspect` is width / height.
public struct ImageBlock: Codable, Sendable, Equatable {
    public var src: String
    public var alt: String
    public var aspect: Double?
}

public struct ProgressBlock: Codable, Sendable, Equatable {
    public var value: Double
    public var label: String?
    public var tone: Token.Tone?
}

public struct BadgeBlock: Codable, Sendable, Equatable {
    public var text: String
    public var tone: Token.Tone?
}

public struct SpacerBlock: Codable, Sendable, Equatable {
    public var size: Token.Space?
}

public struct CustomBlock: Codable, Sendable, Equatable {
    public var name: String
    public var props: [String: JSONValue]

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        name = try c.decode(String.self, forKey: .name)
        props = try c.decodeIfPresent([String: JSONValue].self, forKey: .props) ?? [:]
    }
}

/// A piece of UI. The renderer maps each case to a SwiftUI view.
public indirect enum Block: Codable, Sendable, Equatable {
    case stack(StackBlock)
    case row(RowBlock)
    case text(TextBlock)
    case card(CardBlock)
    case grid(GridBlock)
    case list(ListBlock)
    case item(ItemBlock)
    case button(ButtonBlock)
    case toggle(ToggleBlock)
    case field(FieldBlock)
    case choice(ChoiceBlock)
    case icon(IconBlock)
    case image(ImageBlock)
    case progress(ProgressBlock)
    case badge(BadgeBlock)
    case divider
    case spacer(SpacerBlock)
    case custom(CustomBlock)
    /// A type this version does not know. The renderer says so instead of failing.
    case unknown(String)

    private enum TypeKey: String, CodingKey { case type }

    public init(from decoder: Decoder) throws {
        let type = try decoder.container(keyedBy: TypeKey.self).decode(String.self, forKey: .type)
        switch type {
        case "stack": self = .stack(try StackBlock(from: decoder))
        case "row": self = .row(try RowBlock(from: decoder))
        case "text": self = .text(try TextBlock(from: decoder))
        case "card": self = .card(try CardBlock(from: decoder))
        case "grid": self = .grid(try GridBlock(from: decoder))
        case "field": self = .field(try FieldBlock(from: decoder))
        case "choice": self = .choice(try ChoiceBlock(from: decoder))
        case "icon": self = .icon(try IconBlock(from: decoder))
        case "image": self = .image(try ImageBlock(from: decoder))
        case "list": self = .list(try ListBlock(from: decoder))
        case "item": self = .item(try ItemBlock(from: decoder))
        case "button": self = .button(try ButtonBlock(from: decoder))
        case "toggle": self = .toggle(try ToggleBlock(from: decoder))
        case "progress": self = .progress(try ProgressBlock(from: decoder))
        case "badge": self = .badge(try BadgeBlock(from: decoder))
        case "divider": self = .divider
        case "spacer": self = .spacer(try SpacerBlock(from: decoder))
        case "custom": self = .custom(try CustomBlock(from: decoder))
        default: self = .unknown(type)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: TypeKey.self)
        try c.encode(kind, forKey: .type)
        switch self {
        case .stack(let b): try b.encode(to: encoder)
        case .row(let b): try b.encode(to: encoder)
        case .text(let b): try b.encode(to: encoder)
        case .card(let b): try b.encode(to: encoder)
        case .grid(let b): try b.encode(to: encoder)
        case .field(let b): try b.encode(to: encoder)
        case .choice(let b): try b.encode(to: encoder)
        case .icon(let b): try b.encode(to: encoder)
        case .image(let b): try b.encode(to: encoder)
        case .list(let b): try b.encode(to: encoder)
        case .item(let b): try b.encode(to: encoder)
        case .button(let b): try b.encode(to: encoder)
        case .toggle(let b): try b.encode(to: encoder)
        case .progress(let b): try b.encode(to: encoder)
        case .badge(let b): try b.encode(to: encoder)
        case .spacer(let b): try b.encode(to: encoder)
        case .custom(let b): try b.encode(to: encoder)
        case .divider, .unknown: break
        }
    }

    /// The block's type as it appears in JSON.
    public var kind: String {
        switch self {
        case .stack: "stack"
        case .row: "row"
        case .text: "text"
        case .card: "card"
        case .grid: "grid"
        case .field: "field"
        case .choice: "choice"
        case .icon: "icon"
        case .image: "image"
        case .list: "list"
        case .item: "item"
        case .button: "button"
        case .toggle: "toggle"
        case .progress: "progress"
        case .badge: "badge"
        case .divider: "divider"
        case .spacer: "spacer"
        case .custom: "custom"
        case .unknown(let type): type
        }
    }

    /// The children of a container, or none.
    public var children: [Block] {
        switch self {
        case .stack(let b): b.children
        case .row(let b): b.children
        case .card(let b): b.children
        case .grid(let b): b.children
        case .list(let b): b.children
        default: []
        }
    }
}

/// Any JSON value: custom block props, host action values, view data, state.
public enum JSONValue: Codable, Sendable, Hashable {
    case null
    case bool(Bool)
    case number(Double)
    case string(String)
    case array([JSONValue])
    case object([String: JSONValue])

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() {
            self = .null
        } else if let b = try? c.decode(Bool.self) {
            self = .bool(b)
        } else if let n = try? c.decode(Double.self) {
            self = .number(n)
        } else if let s = try? c.decode(String.self) {
            self = .string(s)
        } else if let a = try? c.decode([JSONValue].self) {
            self = .array(a)
        } else {
            self = .object(try c.decode([String: JSONValue].self))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .null: try c.encodeNil()
        case .bool(let b): try c.encode(b)
        case .number(let n): try c.encode(n)
        case .string(let s): try c.encode(s)
        case .array(let a): try c.encode(a)
        case .object(let o): try c.encode(o)
        }
    }

    /// The text of a string, or the JSON of anything else.
    public var display: String {
        if case .string(let s) = self { return s }
        let data = (try? JSONEncoder().encode(self)) ?? Data()
        return String(decoding: data, as: UTF8.self)
    }

    /// Parse JSON text, or `.null` when there is none.
    public static func parse(_ text: String?) throws -> JSONValue {
        guard let text, !text.isEmpty else { return .null }
        return try JSONDecoder().decode(JSONValue.self, from: Data(text.utf8))
    }
}

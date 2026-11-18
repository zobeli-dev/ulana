// The Swift types are written by hand; spec/frame.schema.json is generated
// from the Rust types. For every block and action in the schema, this builds
// a JSON object with every field set, decodes it, encodes it again, and
// expects the same fields back. A field Swift does not know is dropped on
// the way, and the test fails.

import Foundation
import Testing
@testable import Ulana

private let schemaURL = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .appending(path: "../../../../spec/frame.schema.json")
    .standardized

private struct Schema {
    let defs: [String: Any]

    init() throws {
        let data = try Data(contentsOf: schemaURL)
        let root = try JSONSerialization.jsonObject(with: data) as! [String: Any]
        defs = root["$defs"] as! [String: Any]
    }

    func variants(_ def: String) -> [[String: Any]] {
        ((defs[def] as! [String: Any])["oneOf"] as! [[String: Any]])
    }

    /// A valid value for a property, every optional part filled in.
    func sample(_ property: [String: Any], name: String) -> Any {
        if let c = property["const"] { return c }
        if let ref = property["$ref"] as? String { return sample(ref: ref) }
        if let any = property["anyOf"] as? [[String: Any]] {
            let notNull = any.first { ($0["type"] as? String) != "null" }!
            return sample(notNull, name: name)
        }
        let types = (property["type"] as? [String]) ?? [(property["type"] as? String) ?? "object"]
        let type = types.first { $0 != "null" } ?? "object"
        switch type {
        case "string": return "x"
        case "boolean": return true
        case "integer": return 1
        case "number": return 0.5
        case "array": return [] as [Any]
        default: return name == "value" ? NSNull() : ["k": "v"]
        }
    }

    func sample(ref: String) -> Any {
        let name = String(ref.split(separator: "/").last!)
        let def = defs[name] as! [String: Any]
        if let values = def["enum"] as? [String] { return values[0] }
        if name == "Action" { return ["kind": "host", "name": "send", "value": "x"] }
        if name == "Block" { return ["type": "divider"] }
        fatalError("no sample for \(name)")
    }

    func object(_ variant: [String: Any]) -> [String: Any] {
        let props = variant["properties"] as! [String: [String: Any]]
        return props.reduce(into: [:]) { out, entry in out[entry.key] = sample(entry.value, name: entry.key) }
    }
}

@Test func everyBlockKeepsEveryField() throws {
    let schema = try Schema()
    var seen: [String] = []
    for variant in schema.variants("Block") {
        let object = schema.object(variant)
        let type = object["type"] as! String
        seen.append(type)
        let data = try JSONSerialization.data(withJSONObject: object)
        let block = try JSONDecoder().decode(Block.self, from: data)
        #expect(block.kind == type, "Swift does not know the block type \(type)")
        let back = try JSONSerialization.jsonObject(with: JSONEncoder().encode(block)) as! [String: Any]
        #expect(Set(back.keys) == Set(object.keys), "\(type): Swift keeps \(back.keys.sorted()), the schema has \(object.keys.sorted())")
    }
    #expect(seen.count == 18)
}

@Test func everyActionKeepsEveryField() throws {
    let schema = try Schema()
    for variant in schema.variants("Action") {
        var object = schema.object(variant)
        if object["kind"] as? String == "host" { object["value"] = "x" }
        let action = try JSONDecoder().decode(Action.self, from: JSONSerialization.data(withJSONObject: object))
        let back = try JSONSerialization.jsonObject(with: JSONEncoder().encode(action)) as! [String: Any]
        #expect(Set(back.keys) == Set(object.keys))
    }
}

@Test func tokensMatchTheSchema() throws {
    let schema = try Schema()
    func values(_ name: String) -> Set<String> {
        Set((schema.defs[name] as! [String: Any])["enum"] as! [String])
    }
    #expect(values("Space") == Set(Token.Space.allCases.map(\.rawValue)))
    #expect(values("Align") == Set(Token.Align.allCases.map(\.rawValue)))
    #expect(values("Justify") == Set(Token.Justify.allCases.map(\.rawValue)))
    #expect(values("TextStyle") == Set(Token.TextStyle.allCases.map(\.rawValue)))
    #expect(values("Tone") == Set(Token.Tone.allCases.map(\.rawValue)))
    #expect(values("ButtonStyle") == Set(Token.ButtonStyle.allCases.map(\.rawValue)))
    #expect(values("FieldKind") == Set(Token.FieldKind.allCases.map(\.rawValue)))
    #expect(values("IconName") == Set(Token.Icon.allCases.map(\.rawValue)))
}

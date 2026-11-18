import Foundation
import Testing
@testable import Ulana

private func decode(_ json: String) throws -> Frame {
    try JSONDecoder().decode(Frame.self, from: Data(json.utf8))
}

@Test func aFrameFromTheRuntimeDecodes() throws {
    // `ulana render examples/views/counter.luau`
    let frame = try decode("""
    {"version":3,"root":{"type":"card","title":"Counter","children":[
      {"type":"text","text":"2","style":"title"},
      {"type":"row","gap":"sm","children":[
        {"type":"button","label":"Less","action":{"kind":"event","id":"h1"},"style":"secondary"},
        {"type":"button","label":"More","action":{"kind":"event","id":"h2"}}]}]}}
    """)
    #expect(frame.version == 3)
    guard case .card(let card) = frame.root else { Issue.record("not a card"); return }
    #expect(card.title == "Counter")
    #expect(card.children.first == .text(try JSONDecoder().decode(TextBlock.self, from: Data(#"{"text":"2","style":"title"}"#.utf8))))
    guard case .row(let row) = card.children[1], case .button(let more) = row.children[1] else {
        Issue.record("no button"); return
    }
    #expect(more.action == .event(id: "h2"))
    #expect(more.style == nil)
}

@Test func anUnknownBlockIsKeptNotFatal() throws {
    let frame = try decode(#"{"version":1,"root":{"type":"stack","children":[{"type":"video","src":"x"},{"type":"divider"}]}}"#)
    #expect(frame.root.children == [.unknown("video"), .divider])
}

@Test func containersMayLeaveOutChildrenAndCustomMayLeaveOutProps() throws {
    let frame = try decode(#"{"version":1,"root":{"type":"stack","children":[{"type":"list"},{"type":"custom","name":"calendar"}]}}"#)
    guard case .custom(let custom) = frame.root.children[1] else { Issue.record("no custom"); return }
    #expect(frame.root.children[0].children.isEmpty)
    #expect(custom.props.isEmpty)
}

@Test func hostActionsCarryAnyValue() throws {
    let action = try JSONDecoder().decode(Action.self, from: Data(#"{"kind":"host","name":"send","value":{"text":"hi","n":[1,true,null]}}"#.utf8))
    #expect(action == .host(name: "send", value: .object(["text": .string("hi"), "n": .array([.number(1), .bool(true), .null])])))
    let bare = try JSONDecoder().decode(Action.self, from: Data(#"{"kind":"host","name":"send"}"#.utf8))
    #expect(bare == .host(name: "send", value: .null))
    #expect(throws: DecodingError.self) {
        try JSONDecoder().decode(Action.self, from: Data(#"{"kind":"teleport"}"#.utf8))
    }
}

@Test func jsonValuesReadAndShow() throws {
    #expect(try JSONValue.parse(nil) == .null)
    #expect(try JSONValue.parse(#"{"a":[1,2]}"#) == .object(["a": .array([.number(1), .number(2)])]))
    #expect(JSONValue.string("hi").display == "hi")
    #expect(JSONValue.array([.bool(true)]).display == "[true]")
}

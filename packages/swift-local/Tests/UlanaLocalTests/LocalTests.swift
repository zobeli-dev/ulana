import Foundation
import Testing
import Ulana
@testable import UlanaLocal

private let counter = """
return function(data)
    local n = ui.state("n", 0)
    print("render", n:get())
    return ui.stack{ ui.text((data and data.label or "n") .. "=" .. n:get()),
        ui.button{ "+", on_press = function() n:set(n:get() + 1) end },
        ui.custom("calendar") }
end
"""

private let host = HostSpec(blocks: ["calendar"], actions: ["send"])

private func firstText(_ frame: Frame) -> String? {
    guard case .text(let t) = frame.root.children.first else { return nil }
    return t.text
}

@Test func aLocalViewRendersHandlesEventsReloadsAndKeepsState() throws {
    let view = try LocalView(source: counter, host: host)
    let first = try view.render(data: .object(["label": .string("count")]))
    #expect(firstText(first) == "count=0")
    try view.event(version: first.version, id: "h1")
    #expect(firstText(try view.render()) == "n=1")
    #expect(throws: UlanaError.self) { try view.event(version: first.version, id: "h1") }

    do {
        try view.reload(source: "return function()")
        Issue.record("a broken script loaded")
    } catch let e as UlanaError {
        #expect(e.kind == "syntax")
        #expect(e.message.hasPrefix("syntax error: view:1:"))
    }
    try view.reload(source: counter.replacingOccurrences(of: "\"=\"", with: "\":\""))
    #expect(firstText(try view.render()) == "n:1")
    #expect(try view.state() == ["n": .number(1)])
    #expect(try view.takeLogs().first == "render\t0")

    let again = try LocalView(source: counter, host: host, state: try view.state())
    #expect(firstText(try again.render()) == "n=1")
}

@Test func theSandboxAndTheChecksHoldOnTheDevice() throws {
    #expect(throws: UlanaError.self) { try LocalView(source: "while true do end") }
    do {
        _ = try LocalView(source: "while true do end")
    } catch let e as UlanaError {
        #expect(e.kind == "timeout")
    }
    let noCalendar = try LocalView(source: counter)
    do {
        _ = try noCalendar.render()
        Issue.record("a custom block the host does not accept was drawn")
    } catch let e as UlanaError {
        #expect(e.kind == "invalid")
        #expect(e.message.contains("root.children[2]"))
    }
    let escape = try LocalView(source: "return ui.text(type(io) .. type(require) .. type(os.execute))")
    guard case .text(let t) = try escape.render().root else { Issue.record("not text"); return }
    #expect(t.text == "nilnilnil")
}

@MainActor
@Test func aLocalSessionKeepsTheLastFrameThroughAnError() throws {
    var saved: [[String: JSONValue]] = []
    let session = LocalSession(source: counter, host: host)
    session.onState = { saved.append($0) }
    #expect(firstText(session.frame!) == "n=0")
    session.send(version: session.frame!.version, id: "h1", value: .null)
    #expect(firstText(session.frame!) == "n=1")
    #expect(saved.last == ["n": .number(1)])

    session.reload(source: "return function( end")
    #expect(session.error?.hasPrefix("syntax error") == true)
    #expect(firstText(session.frame!) == "n=1")

    session.reload(source: counter)
    #expect(session.error == nil)
    session.data = .object(["label": .string("cups")])
    #expect(firstText(session.frame!) == "cups=1")
    #expect(session.logs.contains("render\t1"))

    session.reset(source: counter)
    #expect(firstText(session.frame!) == "cups=0")
    #expect(saved.last == ["n": .number(0)])
}

@MainActor
@Test func theExampleViewsRunOnTheDevice() throws {
    let views = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        .appending(path: "../../../../examples/views").standardized
    let host = HostSpec(blocks: ["calendar"], actions: ["send"], images: ["https://picsum.photos/"])
    for name in ["counter", "todo", "agenda", "weather"] {
        let source = try String(contentsOf: views.appending(path: "\(name).luau"), encoding: .utf8)
        let dataURL = views.appending(path: "\(name).json")
        let data = try JSONValue.parse(try? String(contentsOf: dataURL, encoding: .utf8))
        let session = LocalSession(source: source, host: host, data: data)
        #expect(session.error == nil, "\(name): \(session.error ?? "")")
        #expect(session.frame != nil, "\(name) did not render")
    }
}

@MainActor
@Test func theNewBlocksRunOnTheDevice() throws {
    let source = """
    return function()
        local city = ui.state("city", "Lisbon")
        local unit = ui.state("unit", "°C")
        return ui.grid{ columns = 2,
            ui.field{ "City", value = city:get(), on_submit = function(t) city:set(t) end },
            ui.choice{ "°C", "°F", value = unit:get(), on_change = function(u) unit:set(u) end },
            ui.icon{ "sun", tone = "warning" },
            ui.image{ src = "https://picsum.photos/seed/" .. city:get() .. "/400/300", alt = city:get() },
        }
    end
    """
    let session = LocalSession(source: source, host: HostSpec(images: ["https://picsum.photos/"]))
    #expect(session.error == nil, "\(session.error ?? "")")
    guard case .grid(let grid) = session.frame?.root, case .field(let field) = grid.children[0],
          case .choice(let choice) = grid.children[1], case .icon(let icon) = grid.children[2],
          case .image(let image) = grid.children[3]
    else { Issue.record("unexpected tree"); return }
    #expect(grid.columns == 2)
    #expect(field.label == "City")
    #expect(choice.options == ["°C", "°F"])
    #expect(icon.name == .sun && icon.name.systemName == "sun.max.fill")
    #expect(image.alt == "Lisbon")

    guard case .event(let submit) = field.submit, case .event(let change) = choice.action else {
        Issue.record("no handlers"); return
    }
    session.send(version: session.frame!.version, id: submit, value: .string("Porto"))
    session.send(version: session.frame!.version, id: change, value: .string("°F"))
    #expect(session.state == ["city": .string("Porto"), "unit": .string("°F")])
    guard case .grid(let after) = session.frame?.root, case .image(let moved) = after.children[3] else { return }
    #expect(moved.src == "https://picsum.photos/seed/Porto/400/300")

    let refused = LocalSession(source: source)
    #expect(refused.error == "at root.children[3].src: this app shows no images")
}

use serde_json::{json, Value};
use ulana::{Action, Block, Error, Host, Limits, View};

fn view(source: &str) -> View {
    View::new(
        source,
        Host::new().block("calendar").action("send"),
        Limits::default(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn event_ids(block: &Block, out: &mut Vec<String>) {
    match block {
        Block::Button {
            action: Action::Event { id },
            ..
        } => out.push(id.clone()),
        Block::Toggle {
            action: Some(Action::Event { id }),
            ..
        } => out.push(id.clone()),
        Block::Item {
            action: Some(Action::Event { id }),
            ..
        } => out.push(id.clone()),
        _ => {}
    }
    for child in block.children() {
        event_ids(child, out);
    }
}

fn ids(block: &Block) -> Vec<String> {
    let mut out = Vec::new();
    event_ids(block, &mut out);
    out
}

const COUNTER: &str = r#"
    return function()
        local count = ui.state("count", 0)
        return ui.stack{ gap = "md",
            ui.text{ "Count: " .. count:get(), style = "title" },
            ui.button{ "Add", on_press = function() count:set(count:get() + 1) end },
        }
    end
"#;

#[test]
fn a_counter_counts() {
    let mut v = view(COUNTER);
    let f1 = v.render(&Value::Null).unwrap();
    assert_eq!(f1.version, 1);
    assert_eq!(
        f1.root.children()[0],
        Block::Text {
            text: "Count: 0".into(),
            style: Some(ulana::TextStyle::Title),
            tone: None,
            lines: None,
        }
    );
    let id = ids(&f1.root).remove(0);
    v.event(f1.version, &id, &Value::Null).unwrap();
    let f2 = v.render(&Value::Null).unwrap();
    assert_eq!(f2.version, 2);
    assert!(matches!(&f2.root.children()[0], Block::Text { text, .. } if text == "Count: 1"));
    assert_eq!(v.state()["count"], json!(1));
}

#[test]
fn the_frame_is_plain_json() {
    let mut v = view(COUNTER);
    let frame = v.render(&Value::Null).unwrap();
    assert_eq!(
        serde_json::to_value(&frame).unwrap(),
        json!({ "version": 1, "root": { "type": "stack", "gap": "md", "children": [
            { "type": "text", "text": "Count: 0", "style": "title" },
            { "type": "button", "label": "Add", "action": { "kind": "event", "id": "h1" } }
        ]}})
    );
}

#[test]
fn data_reaches_the_view() {
    let mut v = view(
        r#"
        return function(data)
            local rows = {}
            for _, e in data.events do
                table.insert(rows, ui.item{ e.title, trailing = e.time })
            end
            return ui.list{ rows }
        end
    "#,
    );
    let frame = v
        .render(&json!({ "events": [
            { "title": "Standup", "time": "09:00" },
            { "title": "Lunch", "time": "12:30" }
        ]}))
        .unwrap();
    assert_eq!(frame.root.children().len(), 2);
    assert!(
        matches!(&frame.root.children()[1], Block::Item { title, trailing: Some(t), .. } if title == "Lunch" && t == "12:30")
    );
}

#[test]
fn children_skip_nil_and_false_and_take_text() {
    let mut v = view(
        r#"
        return function()
            local show = false
            return ui.stack{ "plain text", 42, show and ui.text("hidden"), nil, ui.divider(), { ui.badge("a"), ui.badge("b") } }
        end
    "#,
    );
    let frame = v.render(&Value::Null).unwrap();
    let kinds: Vec<_> = frame.root.children().iter().map(Block::kind).collect();
    assert_eq!(kinds, ["text", "text", "divider", "badge", "badge"]);
}

#[test]
fn a_script_can_return_a_tree_or_a_list() {
    let mut v = view(r#"return ui.card{ title = "Hi", ui.text("static") }"#);
    assert_eq!(v.render(&Value::Null).unwrap().root.kind(), "card");
    let mut v = view(r#"return function() return { ui.text("a"), ui.text("b") } end"#);
    let root = v.render(&Value::Null).unwrap().root;
    assert_eq!(root.kind(), "stack");
    assert_eq!(root.children().len(), 2);
}

#[test]
fn a_static_tree_keeps_its_handlers() {
    let mut v = view(
        r#"
        local n = ui.state("n", 0)
        return ui.button{ "Tap", on_press = function() n:set(n:get() + 1) end }
    "#,
    );
    let f = v.render(&Value::Null).unwrap();
    v.event(f.version, "h1", &Value::Null).unwrap();
    assert_eq!(v.state()["n"], json!(1));
}

#[test]
fn a_toggle_gets_its_new_value() {
    let mut v = view(
        r#"
        return function()
            local on = ui.state("compact", false)
            return ui.toggle{ "Compact", value = on:get(), on_change = function(v) on:set(v) end }
        end
    "#,
    );
    let f = v.render(&Value::Null).unwrap();
    v.event(f.version, "h1", &json!(true)).unwrap();
    let f = v.render(&Value::Null).unwrap();
    assert!(matches!(f.root, Block::Toggle { value: true, .. }));
}

#[test]
fn events_must_name_the_latest_frame_and_a_real_handler() {
    let mut v = view(COUNTER);
    let old = v.render(&Value::Null).unwrap();
    v.render(&Value::Null).unwrap();
    assert_eq!(
        v.event(old.version, "h1", &Value::Null),
        Err(Error::Stale { latest: 2 })
    );
    assert_eq!(
        v.event(2, "h9", &Value::Null),
        Err(Error::UnknownEvent("h9".into()))
    );
    assert_eq!(v.state()["count"], json!(0));
}

#[test]
fn reload_keeps_state_and_a_bad_reload_keeps_the_old_script() {
    let mut v = view(COUNTER);
    let f = v.render(&Value::Null).unwrap();
    v.event(f.version, "h1", &Value::Null).unwrap();

    let err = v.reload("return function( end").unwrap_err();
    assert!(matches!(err, Error::Syntax(_)), "{err}");
    let f = v.render(&Value::Null).unwrap();
    assert!(matches!(&f.root.children()[0], Block::Text { text, .. } if text == "Count: 1"));

    v.reload(
        r#"
        return function()
            local count = ui.state("count", 0)
            return ui.text("Now at " .. count:get())
        end
    "#,
    )
    .unwrap();
    assert!(matches!(
        v.event(f.version, "h1", &Value::Null),
        Err(Error::Stale { .. })
    ));
    let f = v.render(&Value::Null).unwrap();
    assert!(matches!(f.root, Block::Text { ref text, .. } if text == "Now at 1"));
}

#[test]
fn saved_state_comes_back() {
    let mut v = view(COUNTER);
    let f = v.render(&Value::Null).unwrap();
    v.event(f.version, "h1", &Value::Null).unwrap();
    let saved = v.state();

    let mut again = View::with_state(COUNTER, Host::new(), Limits::default(), saved).unwrap();
    let f = again.render(&Value::Null).unwrap();
    assert!(matches!(&f.root.children()[0], Block::Text { text, .. } if text == "Count: 1"));
}

#[test]
fn custom_blocks_and_host_actions_follow_the_host() {
    let mut v = view(
        r#"
        return ui.stack{
            ui.custom("calendar"),
            ui.custom("calendar", { range = "week" }),
            ui.button{ "Ask", action = ui.host("send", "What's on today?") },
        }
    "#,
    );
    let frame = serde_json::to_value(v.render(&Value::Null).unwrap()).unwrap();
    assert_eq!(frame["root"]["children"][0]["props"], json!({}));
    assert_eq!(
        frame["root"]["children"][1]["props"],
        json!({ "range": "week" })
    );
    assert_eq!(
        frame["root"]["children"][2]["action"],
        json!({ "kind": "host", "name": "send", "value": "What's on today?" })
    );

    let mut bare = View::new(
        r#"return ui.custom("calendar")"#,
        Host::new(),
        Limits::default(),
    )
    .unwrap();
    let err = bare.render(&Value::Null).unwrap_err();
    assert_eq!(
        err.to_string(),
        "at root.name: this app has no custom block `calendar`; it accepts none"
    );
}

#[test]
fn mistakes_name_where_they_are() {
    let mut v = view(r#"return ui.stack{ ui.text{ "x", colour = "red" } }"#);
    let err = v.render(&Value::Null).unwrap_err();
    let Error::Invalid { path, message } = err else {
        panic!("{err}")
    };
    assert!(path.starts_with("root.children[0]"), "{path}");
    assert!(message.contains("colour"), "{message}");

    let mut v = view(r#"return ui.stack{ gap = "huge" }"#);
    assert!(v
        .render(&Value::Null)
        .unwrap_err()
        .to_string()
        .contains("huge"));

    let mut v = view(r#"return ui.button{ "No action" }"#);
    assert!(v
        .render(&Value::Null)
        .unwrap_err()
        .to_string()
        .contains("action"));

    let err = View::new("return 42", Host::new(), Limits::default())
        .err()
        .unwrap();
    assert!(
        err.to_string()
            .contains("must return a function or a block"),
        "{err}"
    );

    let mut v = view(r#"return function() error("boom") end"#);
    let err = v.render(&Value::Null).unwrap_err();
    assert!(
        matches!(&err, Error::Runtime(m) if m.contains("view:1") && m.contains("boom")),
        "{err}"
    );
}

#[test]
fn state_holds_json_only_and_within_its_limit() {
    let mut v = view(
        r#"
        return function()
            local s = ui.state("s")
            return ui.stack{
                ui.button{ "fn", on_press = function() s:set(function() end) end },
                ui.button{ "big", on_press = function() s:set(string.rep("x", 100000)) end },
                ui.button{ "list", on_press = function() s:set({ 1, 2, { a = true } }) end },
            }
        end
    "#,
    );
    let f = v.render(&Value::Null).unwrap();
    let err = v.event(f.version, "h1", &Value::Null).unwrap_err();
    assert_eq!(err.to_string(), "at state.s: a function cannot go here");
    let err = v.event(f.version, "h2", &Value::Null).unwrap_err();
    assert!(err.to_string().contains("at most"), "{err}");
    assert!(!v.state().contains_key("s"));
    v.event(f.version, "h3", &Value::Null).unwrap();
    assert_eq!(v.state()["s"], json!([1, 2, { "a": true }]));
}

#[test]
fn print_goes_to_the_log() {
    let mut v = view(
        r#"print("loaded", 1, true) return function() print("render") return ui.divider() end"#,
    );
    v.render(&Value::Null).unwrap();
    assert_eq!(v.take_logs(), ["loaded\t1\ttrue", "render"]);
    assert!(v.take_logs().is_empty());
}

#[test]
fn the_new_blocks_build_from_luau() {
    let mut v = View::new(
        r#"
        return function()
            local city = ui.state("city", "Lisbon")
            local unit = ui.state("unit", "°C")
            return ui.grid{ columns = 2, gap = "md",
                ui.field{ "City", value = city:get(), placeholder = "Type a city", kind = "search",
                    on_change = function(t) city:set(t) end,
                    on_submit = function(t) city:set(string.upper(t)) end },
                ui.choice{ "°C", "°F", value = unit:get(), on_change = function(u) unit:set(u) end },
                ui.icon{ "sun", tone = "warning", style = "display", label = "Sunny" },
                ui.image{ src = "https://picsum.photos/seed/" .. city:get() .. "/800/450", alt = city:get(), aspect = 16 / 9 },
                ui.card{ tone = "accent", padding = "lg", ui.text{ "21°", style = "display" } },
                ui.item{ "Inbox", icon = "mail", trailing = "3" },
                ui.button{ "Add", icon = "add", on_press = function() end },
            }
        end
    "#,
        Host::new().image("https://picsum.photos/"),
        Limits::default(),
    )
    .unwrap();
    let f = v.render(&Value::Null).unwrap();
    let json = serde_json::to_value(&f).unwrap();
    let kids = &json["root"]["children"];
    assert_eq!(json["root"]["type"], "grid");
    assert_eq!(
        kids[0],
        json!({ "type": "field", "value": "Lisbon", "label": "City", "placeholder": "Type a city", "kind": "search",
        "action": { "kind": "event", "id": "h1" }, "submit": { "kind": "event", "id": "h2" } })
    );
    assert_eq!(kids[1]["options"], json!(["°C", "°F"]));
    assert_eq!(
        kids[2],
        json!({ "type": "icon", "name": "sun", "tone": "warning", "style": "display", "label": "Sunny" })
    );
    assert_eq!(kids[3]["src"], "https://picsum.photos/seed/Lisbon/800/450");
    assert_eq!(kids[4]["tone"], "accent");
    assert_eq!(kids[5]["icon"], "mail");
    assert_eq!(kids[6]["icon"], "add");

    v.event(f.version, "h1", &json!("Porto")).unwrap();
    let f = v.render(&Value::Null).unwrap();
    v.event(f.version, "h3", &json!("°F")).unwrap();
    let f = v.render(&Value::Null).unwrap();
    v.event(f.version, "h2", &json!("faro")).unwrap();
    assert_eq!(v.state()["city"], json!("FARO"));
    assert_eq!(v.state()["unit"], json!("°F"));

    // Without the host allowing it, the image is refused with its path.
    let mut bare = View::new(
        r#"return ui.image{ src = "https://picsum.photos/1", alt = "x" }"#,
        Host::new(),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        bare.render(&Value::Null).unwrap_err().to_string(),
        "at root.src: this app shows no images"
    );
}

/// docs/rust.md: a server may build the view per request from saved state.
#[test]
fn a_view_rebuilt_per_request_handles_presses() {
    let mut saved = ulana::State::new();
    for expected in 1..=3 {
        let mut view = View::with_state(COUNTER, Host::new(), Limits::default(), saved).unwrap();
        view.render(&Value::Null).unwrap();
        view.event(view.version(), "h1", &Value::Null).unwrap();
        let frame = view.render(&Value::Null).unwrap();
        assert!(
            matches!(&frame.root.children()[0], Block::Text { text, .. } if *text == format!("Count: {expected}"))
        );
        saved = view.state();
    }
}

#[test]
fn the_spec_ships_with_the_crate() {
    assert!(ulana::SPEC.starts_with("# Writing an Ulana view"));
    for block in [
        "ui.grid",
        "ui.field",
        "ui.choice",
        "ui.icon",
        "ui.image",
        "ui.custom",
    ] {
        assert!(
            ulana::SPEC.contains(block),
            "{block} is missing from the spec"
        );
    }
}

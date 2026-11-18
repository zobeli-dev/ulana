//! A view is untrusted code. These tests try to get out.

use std::time::{Duration, Instant};

use serde_json::Value;
use ulana::{Error, Host, Limits, View};

fn limits() -> Limits {
    Limits {
        time: Duration::from_millis(50),
        memory: 4 * 1024 * 1024,
        ..Limits::default()
    }
}

fn load(source: &str) -> Result<View, Error> {
    View::new(source, Host::new(), limits())
}

fn render(source: &str) -> Result<ulana::Frame, Error> {
    load(source)?.render(&Value::Null)
}

#[test]
fn an_endless_loop_stops_at_load_render_and_event() {
    let start = Instant::now();
    assert_eq!(load("while true do end").err(), Some(Error::Timeout));
    assert_eq!(
        render("return function() while true do end end").err(),
        Some(Error::Timeout)
    );

    let mut v =
        load(r#"return ui.button{ "x", on_press = function() repeat until false end }"#).unwrap();
    let f = v.render(&Value::Null).unwrap();
    assert_eq!(v.event(f.version, "h1", &Value::Null), Err(Error::Timeout));
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn pcall_cannot_swallow_the_time_limit() {
    let err = render(
        r#"
        return function()
            while true do
                pcall(function() while true do end end)
            end
        end
    "#,
    );
    assert_eq!(err.err(), Some(Error::Timeout));
}

#[test]
fn a_view_still_works_after_a_timeout() {
    let mut v = load(
        r#"
        local slow = ui.state("slow", true)
        return function()
            if slow:get() then
                slow:set(false)
                while true do end
            end
            return ui.text("fine")
        end
    "#,
    )
    .unwrap();
    assert_eq!(v.render(&Value::Null).err(), Some(Error::Timeout));
    assert!(v.render(&Value::Null).is_ok());
}

#[test]
fn memory_is_capped() {
    assert_eq!(
        render(r#"return function() local t = {} for i = 1, 1e8 do t[i] = i end end"#).err(),
        Some(Error::Memory)
    );
    assert_eq!(
        render(r#"return function() local s = "x" while true do s = s .. s end end"#).err(),
        Some(Error::Memory)
    );
}

#[test]
fn the_library_and_globals_are_read_only() {
    for attack in [
        "ui.text = nil",
        "ui.state = function() end",
        "string.rep = nil",
        "table.insert = print",
        "getmetatable('').__index = {}",
    ] {
        let err = load(attack)
            .err()
            .unwrap_or_else(|| panic!("`{attack}` was allowed"));
        assert!(matches!(err, Error::Runtime(_)), "`{attack}`: {err}");
    }
    // The script's own globals are fine; they stay in its own table.
    assert!(load("x = 1 return ui.text(tostring(x))").is_ok());
}

#[test]
fn nothing_reaches_outside() {
    for name in [
        "require",
        "loadstring",
        "getfenv",
        "setfenv",
        "collectgarbage",
        "io",
        "debug",
        "package",
        "dofile",
        "loadfile",
        "coroutine",
    ] {
        let v = render(&format!(r#"return ui.text(type({name}))"#)).unwrap();
        let ulana::Block::Text { text, .. } = v.root else {
            panic!()
        };
        assert_eq!(text, "nil", "{name} is reachable");
    }
    for name in ["execute", "exit", "getenv", "remove", "rename", "tmpname"] {
        let v = render(&format!(r#"return ui.text(type(os.{name}))"#)).unwrap();
        let ulana::Block::Text { text, .. } = v.root else {
            panic!()
        };
        assert_eq!(text, "nil", "os.{name} is reachable");
    }
}

#[test]
fn the_internals_stay_private() {
    let v = render(
        r#"return ui.text(tostring(ui.begin) .. tostring(ui.handler) .. tostring(state_get))"#,
    )
    .unwrap();
    let ulana::Block::Text { text, .. } = v.root else {
        panic!()
    };
    assert_eq!(text, "nilnilnil");
}

#[test]
fn deep_recursion_is_an_error_not_a_crash() {
    let err =
        render("local function f(n) return f(n + 1) + 1 end return function() return f(0) end")
            .unwrap_err();
    assert!(
        matches!(err, Error::Runtime(_) | Error::Timeout | Error::Memory),
        "{err}"
    );
}

#[test]
fn a_table_that_contains_itself_is_refused() {
    let err = render(
        r#"
        return function()
            local t = ui.stack{}
            table.insert(t.children, t)
            return t
        end
    "#,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Invalid { .. }), "{err}");
}

#[test]
fn a_huge_tree_is_refused() {
    let err = render(
        r#"
        return function()
            local rows = {}
            for i = 1, 5000 do rows[i] = ui.text(tostring(i)) end
            return ui.stack{ rows }
        end
    "#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("at most 2000 blocks"), "{err}");
}

#[test]
fn views_do_not_share_anything() {
    let a = load("shared = 'from a' return ui.divider()").unwrap();
    drop(a);
    let v = render("return ui.text(tostring(shared))").unwrap();
    let ulana::Block::Text { text, .. } = v.root else {
        panic!()
    };
    assert_eq!(text, "nil");
}

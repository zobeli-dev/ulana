//! A C interface to the Ulana runtime, so Swift (and any language with a C
//! FFI) can run views on the device. Values cross as JSON strings; see
//! `include/ulana.h`.

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use serde::Deserialize;
use serde_json::{json, Value};
use ulana::{Error, Host, Limits, State, View};

/// An opaque handle to one view.
pub struct UlanaView(View);

#[derive(Deserialize, Default)]
struct HostSpec {
    #[serde(default)]
    blocks: Vec<String>,
    #[serde(default)]
    actions: Vec<String>,
}

fn kind(e: &Error) -> &'static str {
    match e {
        Error::Syntax(_) => "syntax",
        Error::Runtime(_) => "runtime",
        Error::Memory => "memory",
        Error::Timeout => "timeout",
        Error::Invalid { .. } => "invalid",
        Error::Stale { .. } => "stale",
        Error::UnknownEvent(_) => "unknown_event",
    }
}

fn error(kind: &str, message: impl Into<String>) -> Value {
    json!({ "error": { "kind": kind, "message": message.into() } })
}

fn from_ulana(e: Error) -> Value {
    error(kind(&e), e.to_string())
}

fn out(value: Value) -> *mut c_char {
    // JSON never contains a NUL byte, because serde escapes it.
    CString::new(value.to_string()).map_or(ptr::null_mut(), CString::into_raw)
}

/// Read a C string, or `None` for NULL.
unsafe fn text<'a>(p: *const c_char) -> Result<Option<&'a str>, Value> {
    if p.is_null() {
        return Ok(None);
    }
    CStr::from_ptr(p)
        .to_str()
        .map(Some)
        .map_err(|_| error("argument", "text must be UTF-8"))
}

unsafe fn json_arg(p: *const c_char, what: &str) -> Result<Value, Value> {
    match text(p)? {
        None => Ok(Value::Null),
        Some(s) => serde_json::from_str(s).map_err(|e| error("argument", format!("{what}: {e}"))),
    }
}

/// Run `f`, turning a panic into an error instead of unwinding into C.
fn guard(f: impl FnOnce() -> Value) -> *mut c_char {
    out(catch_unwind(AssertUnwindSafe(f))
        .unwrap_or_else(|_| error("panic", "the runtime stopped unexpectedly")))
}

/// # Safety
/// `source` must be a valid C string; `host_json`, `state_json`, and
/// `error_out` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_new(
    source: *const c_char,
    host_json: *const c_char,
    state_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut UlanaView {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<UlanaView, Value> {
        let source = text(source)?.ok_or_else(|| error("argument", "source is required"))?;
        let spec: HostSpec = match json_arg(host_json, "host")? {
            Value::Null => HostSpec::default(),
            v => serde_json::from_value(v).map_err(|e| error("argument", format!("host: {e}")))?,
        };
        let state: State = match json_arg(state_json, "state")? {
            Value::Null => State::new(),
            v => serde_json::from_value(v).map_err(|e| error("argument", format!("state: {e}")))?,
        };
        let mut host = Host::new();
        for b in spec.blocks {
            host = host.block(b);
        }
        for a in spec.actions {
            host = host.action(a);
        }
        View::with_state(source, host, Limits::default(), state)
            .map(UlanaView)
            .map_err(from_ulana)
    }))
    .unwrap_or_else(|_| Err(error("panic", "the runtime stopped unexpectedly")));
    match result {
        Ok(view) => Box::into_raw(Box::new(view)),
        Err(e) => {
            if !error_out.is_null() {
                *error_out = out(e);
            }
            ptr::null_mut()
        }
    }
}

unsafe fn with_view(
    view: *mut UlanaView,
    f: impl FnOnce(&mut View) -> Result<Value, Value>,
) -> *mut c_char {
    if view.is_null() {
        return out(error("argument", "view is NULL"));
    }
    let view = &mut (*view).0;
    guard(|| match f(view) {
        Ok(v) => json!({ "ok": v }),
        Err(e) => e,
    })
}

/// # Safety
/// `view` must come from `ulana_view_new`; `data_json` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_render(
    view: *mut UlanaView,
    data_json: *const c_char,
) -> *mut c_char {
    with_view(view, |v| {
        let data = json_arg(data_json, "data")?;
        let frame = v.render(&data).map_err(from_ulana)?;
        serde_json::to_value(frame).map_err(|e| error("internal", e.to_string()))
    })
}

/// # Safety
/// `view` must come from `ulana_view_new`; `id` must be a valid C string;
/// `value_json` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_event(
    view: *mut UlanaView,
    version: u64,
    id: *const c_char,
    value_json: *const c_char,
) -> *mut c_char {
    with_view(view, |v| {
        let id = text(id)?.ok_or_else(|| error("argument", "id is required"))?;
        let value = json_arg(value_json, "value")?;
        v.event(version, id, &value).map_err(from_ulana)?;
        Ok(Value::Null)
    })
}

/// # Safety
/// `view` must come from `ulana_view_new`; `source` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_reload(
    view: *mut UlanaView,
    source: *const c_char,
) -> *mut c_char {
    with_view(view, |v| {
        let source = text(source)?.ok_or_else(|| error("argument", "source is required"))?;
        v.reload(source).map_err(from_ulana)?;
        Ok(Value::Null)
    })
}

/// # Safety
/// `view` must come from `ulana_view_new`.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_state(view: *mut UlanaView) -> *mut c_char {
    with_view(view, |v| {
        serde_json::to_value(v.state()).map_err(|e| error("internal", e.to_string()))
    })
}

/// # Safety
/// `view` must come from `ulana_view_new`.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_take_logs(view: *mut UlanaView) -> *mut c_char {
    with_view(view, |v| Ok(json!(v.take_logs())))
}

/// # Safety
/// `view` must come from `ulana_view_new` and not be used afterwards. NULL is ignored.
#[no_mangle]
pub unsafe extern "C" fn ulana_view_free(view: *mut UlanaView) {
    if !view.is_null() {
        drop(Box::from_raw(view));
    }
}

/// # Safety
/// `string` must come from this library and not be used afterwards. NULL is ignored.
#[no_mangle]
pub unsafe extern "C" fn ulana_string_free(string: *mut c_char) {
    if !string.is_null() {
        drop(CString::from_raw(string));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    unsafe fn take(p: *mut c_char) -> Value {
        assert!(!p.is_null());
        let v = serde_json::from_str(CStr::from_ptr(p).to_str().unwrap()).unwrap();
        ulana_string_free(p);
        v
    }

    const COUNTER: &str = r#"
        return function(data)
            local n = ui.state("n", 0)
            print("render", n:get())
            return ui.stack{ ui.text((data and data.label or "n") .. "=" .. n:get()),
                ui.button{ "+", on_press = function() n:set(n:get() + 1) end },
                ui.custom("calendar") }
        end
    "#;

    #[test]
    fn a_view_renders_handles_events_reloads_and_keeps_state() {
        unsafe {
            let host = c(r#"{"blocks":["calendar"],"actions":["send"]}"#);
            let view = ulana_view_new(
                c(COUNTER).as_ptr(),
                host.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
            );
            assert!(!view.is_null());

            let r = take(ulana_view_render(view, c(r#"{"label":"count"}"#).as_ptr()));
            assert_eq!(r["ok"]["version"], 1);
            assert_eq!(r["ok"]["root"]["children"][0]["text"], "count=0");

            let e = take(ulana_view_event(view, 1, c("h1").as_ptr(), ptr::null()));
            assert_eq!(e, json!({ "ok": null }));
            let stale = take(ulana_view_event(view, 1, c("h1").as_ptr(), ptr::null()));
            assert_eq!(
                stale,
                json!({ "ok": null }),
                "the event bumped no version yet"
            );
            let r = take(ulana_view_render(view, ptr::null()));
            assert_eq!(r["ok"]["root"]["children"][0]["text"], "n=2");
            let stale = take(ulana_view_event(view, 1, c("h1").as_ptr(), ptr::null()));
            assert_eq!(stale["error"]["kind"], "stale");

            let bad = take(ulana_view_reload(view, c("return function( end").as_ptr()));
            assert_eq!(bad["error"]["kind"], "syntax");
            let ok = take(ulana_view_reload(
                view,
                c(&COUNTER.replace("\"=\"", "\":\"")).as_ptr(),
            ));
            assert_eq!(ok, json!({ "ok": null }));
            let r = take(ulana_view_render(view, ptr::null()));
            assert_eq!(r["ok"]["root"]["children"][0]["text"], "n:2");

            assert_eq!(take(ulana_view_state(view)), json!({ "ok": { "n": 2 } }));
            let logs = take(ulana_view_take_logs(view));
            assert_eq!(logs["ok"][0], "render\t0");
            ulana_view_free(view);
        }
    }

    #[test]
    fn state_comes_back_and_errors_are_envelopes() {
        unsafe {
            let view = ulana_view_new(
                c(COUNTER).as_ptr(),
                c(r#"{"blocks":["calendar"]}"#).as_ptr(),
                c(r#"{"n":41}"#).as_ptr(),
                ptr::null_mut(),
            );
            let r = take(ulana_view_render(view, ptr::null()));
            assert_eq!(r["ok"]["root"]["children"][0]["text"], "n=41");
            let unknown = take(ulana_view_event(view, 1, c("h9").as_ptr(), ptr::null()));
            assert_eq!(unknown["error"]["kind"], "unknown_event");
            let bad = take(ulana_view_render(view, c("{nope").as_ptr()));
            assert_eq!(bad["error"]["kind"], "argument");
            ulana_view_free(view);

            // Without `calendar` in the host the tree is refused, with its path.
            let view = ulana_view_new(
                c(COUNTER).as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
            );
            let r = take(ulana_view_render(view, ptr::null()));
            assert_eq!(r["error"]["kind"], "invalid");
            assert!(r["error"]["message"]
                .as_str()
                .unwrap()
                .contains("root.children[2]"));
            ulana_view_free(view);

            let mut err = ptr::null_mut();
            let none = ulana_view_new(c("return 1 +").as_ptr(), ptr::null(), ptr::null(), &mut err);
            assert!(none.is_null());
            assert_eq!(take(err)["error"]["kind"], "syntax");

            let none = ulana_view_new(
                c("while true do end").as_ptr(),
                ptr::null(),
                ptr::null(),
                &mut err,
            );
            assert!(none.is_null());
            assert_eq!(take(err)["error"]["kind"], "timeout");

            assert_eq!(
                take(ulana_view_render(ptr::null_mut(), ptr::null()))["error"]["kind"],
                "argument"
            );
            ulana_view_free(ptr::null_mut());
            ulana_string_free(ptr::null_mut());
        }
    }
}

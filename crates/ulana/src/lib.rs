//! Ulana: Luau scripts that draw native UI.
//!
//! A view is a small Luau script. It returns a function that builds a tree of
//! blocks with the `ui` library, and the host app draws that tree with its
//! own native components. The script runs in a sandbox with limits on memory
//! and time, never touches the network or files, and can only ask the host to
//! do the things the host named.
//!
//! ```
//! use ulana::{Host, Limits, View};
//!
//! let source = r#"
//!     return function(data)
//!         local count = ui.state("count", 0)
//!         return ui.stack{ gap = "md",
//!             ui.text{ "Hello, " .. data.name, style = "title" },
//!             ui.button{ "Add one", on_press = function()
//!                 count:set(count:get() + 1)
//!             end },
//!             ui.text("Count: " .. count:get()),
//!         }
//!     end
//! "#;
//! let mut view = View::new(source, Host::new(), Limits::default()).unwrap();
//! let frame = view.render(&serde_json::json!({ "name": "Ana" })).unwrap();
//! view.event(frame.version, "h1", &serde_json::Value::Null).unwrap();
//! assert_eq!(view.state()["count"], 1);
//! ```
//!
//! A tree can also come from JSON (a model, a server) through [`parse`],
//! which applies the same checks.

mod block;
mod check;
mod convert;
mod error;
mod runtime;

pub use block::{
    Action, Align, Block, ButtonStyle, FieldKind, Frame, IconName, Justify, Space, TextStyle, Tone,
};
pub use check::{parse, Host, Limits};
pub use error::{Error, Result};
pub use runtime::{State, View};

/// The view API on one page (`spec/ulana.md`), written to be given to a
/// model that writes views. Send it whole, with what your [`Host`] accepts.
pub const SPEC: &str = include_str!("../SPEC.md");

/// The JSON Schema of a [`Frame`], for hosts in other languages.
pub fn frame_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Frame)).expect("a schema is JSON")
}

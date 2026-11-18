//! The checker every tree passes before a host draws it.

use std::collections::BTreeSet;
use std::time::Duration;

use crate::block::{Action, Block};
use crate::error::{Error, Result};

/// What the host app accepts beyond the built-in blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Host {
    blocks: BTreeSet<String>,
    actions: BTreeSet<String>,
}

impl Host {
    pub fn new() -> Self {
        Self::default()
    }

    /// Accept `custom` blocks with this name, such as `calendar`.
    pub fn block(mut self, name: impl Into<String>) -> Self {
        self.blocks.insert(name.into());
        self
    }

    /// Accept host actions with this name, such as `send`.
    pub fn action(mut self, name: impl Into<String>) -> Self {
        self.actions.insert(name.into());
        self
    }

    pub fn blocks(&self) -> impl Iterator<Item = &str> {
        self.blocks.iter().map(String::as_str)
    }

    pub fn actions(&self) -> impl Iterator<Item = &str> {
        self.actions.iter().map(String::as_str)
    }
}

/// How much a view may use. The defaults suit a phone screen and a server
/// answering one person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Bytes the Luau heap may hold.
    pub memory: usize,
    /// Time one load, render, or event may run.
    pub time: Duration,
    /// Blocks in one tree.
    pub blocks: usize,
    /// How deep blocks may nest.
    pub depth: usize,
    /// Characters in one piece of text. Longer text is cut, not refused.
    pub text: usize,
    /// Bytes of JSON in the view's state.
    pub state: usize,
    /// Bytes of JSON in one custom block's props or one host action's value.
    pub props: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            memory: 16 * 1024 * 1024,
            time: Duration::from_millis(100),
            blocks: 2000,
            depth: 32,
            text: 4000,
            state: 64 * 1024,
            props: 16 * 1024,
        }
    }
}

/// Read a tree from JSON and check it.
///
/// A tree that did not come from a Luau view has no handlers, so an `event`
/// action is refused here.
pub fn parse(json: serde_json::Value, host: &Host, limits: &Limits) -> Result<Block> {
    check(read(json)?, host, limits, &|_| false)
}

/// How deep [`read`] follows children before giving up, whatever the limits say,
/// so a deep tree cannot overflow the stack before the checker counts it.
const READ_DEPTH: usize = 128;

/// Read a block from JSON, naming the block where something does not fit.
///
/// Children are read one by one, so an error names `root.children[2]`
/// rather than only `root`.
pub(crate) fn read(json: serde_json::Value) -> Result<Block> {
    read_at(json, "root", 0)
}

fn read_at(mut json: serde_json::Value, path: &str, depth: usize) -> Result<Block> {
    if depth > READ_DEPTH {
        return Err(Error::invalid(path, "blocks nest too deep"));
    }
    let children = json.as_object_mut().and_then(|map| map.remove("children"));
    let mut block: Block =
        serde_json::from_value(json).map_err(|e| Error::invalid(path, e.to_string()))?;
    if let Some(children) = children {
        let kind = block.kind();
        let Some(slot) = block.children_mut() else {
            return Err(Error::invalid(
                path,
                format!("a {kind} block has no children"),
            ));
        };
        let serde_json::Value::Array(items) = children else {
            return Err(Error::invalid(
                join(path, "children"),
                "children must be a list",
            ));
        };
        for (i, item) in items.into_iter().enumerate() {
            slot.push(read_at(item, &format!("{path}.children[{i}]"), depth + 1)?);
        }
    }
    Ok(block)
}

/// Check a tree and return it with long text cut.
///
/// `has_event` says whether an `event` action names a handler that exists.
pub(crate) fn check(
    mut block: Block,
    host: &Host,
    limits: &Limits,
    has_event: &dyn Fn(&str) -> bool,
) -> Result<Block> {
    let mut count = 0;
    let mut checker = Checker {
        host,
        limits,
        has_event,
        count: &mut count,
    };
    checker.block(&mut block, "root", 1)?;
    Ok(block)
}

struct Checker<'a> {
    host: &'a Host,
    limits: &'a Limits,
    has_event: &'a dyn Fn(&str) -> bool,
    count: &'a mut usize,
}

impl Checker<'_> {
    fn block(&mut self, block: &mut Block, path: &str, depth: usize) -> Result<()> {
        *self.count += 1;
        if *self.count > self.limits.blocks {
            return Err(Error::invalid(
                path,
                format!("a tree may have at most {} blocks", self.limits.blocks),
            ));
        }
        if depth > self.limits.depth {
            return Err(Error::invalid(
                path,
                format!("blocks may nest at most {} deep", self.limits.depth),
            ));
        }
        let text = self.limits.text;
        match block {
            Block::Stack { .. } | Block::Row { .. } | Block::List { .. } | Block::Divider {} => {}
            Block::Spacer { .. } => {}
            Block::Card { title, .. } => {
                if let Some(t) = title {
                    cut(t, text);
                }
            }
            Block::Text { text: t, lines, .. } => {
                cut(t, text);
                if let Some(n) = lines {
                    if *n == 0 {
                        return Err(Error::invalid(
                            join(path, "lines"),
                            "lines must be 1 or more",
                        ));
                    }
                }
            }
            Block::Item {
                title,
                subtitle,
                trailing,
                action,
            } => {
                required(title, &join(path, "title"))?;
                cut(title, text);
                if let Some(s) = subtitle {
                    cut(s, text);
                }
                if let Some(s) = trailing {
                    cut(s, text);
                }
                if let Some(a) = action {
                    self.action(a, &join(path, "action"))?;
                }
            }
            Block::Button { label, action, .. } => {
                required(label, &join(path, "label"))?;
                cut(label, text);
                self.action(action, &join(path, "action"))?;
            }
            Block::Toggle { label, action, .. } => {
                required(label, &join(path, "label"))?;
                cut(label, text);
                if let Some(a) = action {
                    self.action(a, &join(path, "action"))?;
                }
            }
            Block::Progress { value, label, .. } => {
                if !value.is_finite() || *value < 0.0 || *value > 1.0 {
                    return Err(Error::invalid(
                        join(path, "value"),
                        "value must be a number from 0 to 1",
                    ));
                }
                if let Some(l) = label {
                    cut(l, text);
                }
            }
            Block::Badge { text: t, .. } => {
                required(t, &join(path, "text"))?;
                cut(t, text);
            }
            Block::Custom { name, props } => {
                if !self.host.blocks.contains(name.as_str()) {
                    return Err(Error::invalid(
                        join(path, "name"),
                        format!(
                            "this app has no custom block `{name}`{}",
                            accepted(self.host.blocks())
                        ),
                    ));
                }
                // An empty Luau table reads as a list; props are always an object.
                if props.is_null() || props.as_array().is_some_and(Vec::is_empty) {
                    *props = serde_json::Value::Object(Default::default());
                }
                self.size(props, &join(path, "props"))?;
            }
        }
        if let Some(children) = block.children_mut() {
            for (i, child) in children.iter_mut().enumerate() {
                self.block(child, &format!("{path}.children[{i}]"), depth + 1)?;
            }
        }
        Ok(())
    }

    fn action(&self, action: &Action, path: &str) -> Result<()> {
        match action {
            Action::Event { id } => {
                if !(self.has_event)(id) {
                    return Err(Error::invalid(
                        path,
                        format!("no handler `{id}`; only a Luau view can handle events"),
                    ));
                }
            }
            Action::Host { name, value } => {
                if !self.host.actions.contains(name.as_str()) {
                    return Err(Error::invalid(
                        join(path, "name"),
                        format!(
                            "this app has no host action `{name}`{}",
                            accepted(self.host.actions())
                        ),
                    ));
                }
                self.size(value, &join(path, "value"))?;
            }
        }
        Ok(())
    }

    fn size(&self, value: &serde_json::Value, path: &str) -> Result<()> {
        let bytes = serde_json::to_vec(value)
            .map(|v| v.len())
            .unwrap_or(usize::MAX);
        if bytes > self.limits.props {
            return Err(Error::invalid(
                path,
                format!("at most {} bytes of JSON", self.limits.props),
            ));
        }
        Ok(())
    }
}

fn required(text: &str, path: &str) -> Result<()> {
    if text.trim().is_empty() {
        return Err(Error::invalid(path, "must not be empty"));
    }
    Ok(())
}

/// Cut `text` to `limit` characters, ending in an ellipsis.
fn cut(text: &mut String, limit: usize) {
    if let Some((at, _)) = text.char_indices().nth(limit) {
        let keep = text[..at]
            .char_indices()
            .nth(limit.saturating_sub(1))
            .map_or(at, |(i, _)| i);
        text.truncate(keep);
        text.push('…');
    }
}

fn join(path: &str, field: &str) -> String {
    match (path.is_empty(), field.is_empty()) {
        (_, true) => path.to_string(),
        (true, false) => field.to_string(),
        (false, false) if field.starts_with('[') => format!("{path}{field}"),
        (false, false) => format!("{path}.{field}"),
    }
}

fn accepted<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let names: Vec<_> = names.map(|n| format!("`{n}`")).collect();
    if names.is_empty() {
        "; it accepts none".to_string()
    } else {
        format!("; it accepts {}", names.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn host() -> Host {
        Host::new().block("calendar").action("send")
    }

    #[test]
    fn cut_keeps_the_limit_in_characters() {
        let mut s = "héllo wörld".to_string();
        cut(&mut s, 5);
        assert_eq!(s, "héll…");
        assert_eq!(s.chars().count(), 5);
        let mut short = "hi".to_string();
        cut(&mut short, 5);
        assert_eq!(short, "hi");
    }

    #[test]
    fn parse_reads_a_tree() {
        let tree = parse(
            json!({ "type": "stack", "gap": "md", "children": [
                { "type": "text", "text": "Today", "style": "title" },
                { "type": "button", "label": "Ask", "action": { "kind": "host", "name": "send", "value": "hi" } },
                { "type": "custom", "name": "calendar", "props": { "range": "today" } }
            ]}),
            &host(),
            &Limits::default(),
        )
        .unwrap();
        assert_eq!(tree.children().len(), 3);
    }

    #[test]
    fn unknown_fields_and_types_name_the_path() {
        let err = parse(
            json!({ "type": "stack", "children": [{ "type": "text", "text": "x", "colour": "red" }] }),
            &host(),
            &Limits::default(),
        )
        .unwrap_err();
        let Error::Invalid { path, message } = err else {
            panic!()
        };
        assert!(path.starts_with("root.children[0]"), "{path}");
        assert!(message.contains("colour"), "{message}");

        let err = parse(json!({ "type": "video" }), &host(), &Limits::default()).unwrap_err();
        assert!(err.to_string().contains("video"), "{err}");
    }

    #[test]
    fn spacing_is_a_token_not_a_number() {
        let err = parse(
            json!({ "type": "stack", "gap": 12 }),
            &host(),
            &Limits::default(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn host_names_are_checked() {
        let err = parse(
            json!({ "type": "custom", "name": "map" }),
            &host(),
            &Limits::default(),
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "at root.name: this app has no custom block `map`; it accepts `calendar`"
        );
        let err = parse(
            json!({ "type": "button", "label": "Pay", "action": { "kind": "host", "name": "pay" } }),
            &host(),
            &Limits::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no host action `pay`"), "{err}");
    }

    #[test]
    fn events_need_a_view() {
        let err = parse(
            json!({ "type": "button", "label": "Add", "action": { "kind": "event", "id": "h1" } }),
            &host(),
            &Limits::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("only a Luau view"), "{err}");
    }

    #[test]
    fn limits_hold() {
        let limits = Limits {
            blocks: 3,
            depth: 2,
            text: 4,
            props: 10,
            ..Limits::default()
        };
        let many = json!({ "type": "stack", "children": [
            { "type": "divider" }, { "type": "divider" }, { "type": "divider" }
        ]});
        assert!(parse(many, &host(), &limits)
            .unwrap_err()
            .to_string()
            .contains("at most 3 blocks"));

        let deep = json!({ "type": "stack", "children": [{ "type": "stack", "children": [{ "type": "divider" }] }] });
        assert!(parse(deep, &host(), &limits)
            .unwrap_err()
            .to_string()
            .contains("nest"));

        let long = parse(
            json!({ "type": "text", "text": "abcdefgh" }),
            &host(),
            &limits,
        )
        .unwrap();
        assert_eq!(
            long,
            Block::Text {
                text: "abc…".into(),
                style: None,
                tone: None,
                lines: None
            }
        );

        let big = json!({ "type": "custom", "name": "calendar", "props": { "x": "0123456789" } });
        assert!(parse(big, &host(), &limits)
            .unwrap_err()
            .to_string()
            .contains("bytes"));
    }

    #[test]
    fn values_are_checked() {
        let l = Limits::default();
        for bad in [
            json!({ "type": "progress", "value": 1.5 }),
            json!({ "type": "progress", "value": -0.1 }),
            json!({ "type": "text", "text": "x", "lines": 0 }),
            json!({ "type": "button", "label": "  ", "action": { "kind": "host", "name": "send" } }),
            json!({ "type": "item", "title": "" }),
        ] {
            assert!(parse(bad.clone(), &host(), &l).is_err(), "{bad}");
        }
    }
}

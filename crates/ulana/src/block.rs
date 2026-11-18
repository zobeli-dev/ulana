//! The block tree: everything a host app draws.
//!
//! A tree is plain data. A Luau view produces one, and so can anything else
//! that writes JSON (a model, a server, a test). Every tree goes through
//! [`crate::check`] before a host draws it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One rendered tree and the version its events must name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    /// Goes up by one on every render. An event for an older frame is refused.
    pub version: u64,
    pub root: Block,
}

/// A piece of UI. The host maps each type to its own native component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Block {
    /// Children top to bottom.
    Stack {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gap: Option<Space>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        padding: Option<Space>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<Align>,
        #[serde(default)]
        children: Vec<Block>,
    },
    /// Children left to right (start to end in right-to-left languages).
    Row {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gap: Option<Space>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        padding: Option<Space>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<Align>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        justify: Option<Justify>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        wrap: Option<bool>,
        #[serde(default)]
        children: Vec<Block>,
    },
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style: Option<TextStyle>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<Tone>,
        /// Cut the text after this many lines.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lines: Option<u8>,
    },
    /// A surface with an optional title.
    Card {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default)]
        children: Vec<Block>,
    },
    /// Children with dividers between them. Usually `item` blocks.
    List {
        #[serde(default)]
        children: Vec<Block>,
    },
    /// A list row: a title, an optional second line, and an optional value at the end.
    Item {
        title: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subtitle: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trailing: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<Action>,
    },
    Button {
        label: String,
        action: Action,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style: Option<ButtonStyle>,
    },
    Toggle {
        label: String,
        value: bool,
        /// Gets the new value.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<Action>,
    },
    /// A bar from 0 to 1.
    Progress {
        value: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<Tone>,
    },
    /// A short label in a pill.
    Badge {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<Tone>,
    },
    Divider {},
    /// Empty space: a fixed size, or all that is left when there is no size.
    Spacer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<Space>,
    },
    /// A block the host app adds, such as a calendar. The host lists the names it accepts.
    Custom {
        name: String,
        #[serde(default)]
        props: serde_json::Value,
    },
}

/// What a press or a change does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Run the view's own handler, then render again.
    Event { id: String },
    /// Ask the host app to do something it named, such as sending a message.
    Host {
        name: String,
        #[serde(default)]
        value: serde_json::Value,
    },
}

/// Spacing tokens. The host's theme decides the points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Space {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Justify {
    Start,
    Center,
    End,
    Between,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    Title,
    Heading,
    Body,
    Caption,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Default,
    Muted,
    Accent,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Plain,
}

impl Block {
    /// The children of a container, or an empty slice.
    pub fn children(&self) -> &[Block] {
        match self {
            Block::Stack { children, .. }
            | Block::Row { children, .. }
            | Block::Card { children, .. }
            | Block::List { children } => children,
            _ => &[],
        }
    }

    pub(crate) fn children_mut(&mut self) -> Option<&mut Vec<Block>> {
        match self {
            Block::Stack { children, .. }
            | Block::Row { children, .. }
            | Block::Card { children, .. }
            | Block::List { children } => Some(children),
            _ => None,
        }
    }

    /// The block's type as it appears in JSON.
    pub fn kind(&self) -> &'static str {
        match self {
            Block::Stack { .. } => "stack",
            Block::Row { .. } => "row",
            Block::Text { .. } => "text",
            Block::Card { .. } => "card",
            Block::List { .. } => "list",
            Block::Item { .. } => "item",
            Block::Button { .. } => "button",
            Block::Toggle { .. } => "toggle",
            Block::Progress { .. } => "progress",
            Block::Badge { .. } => "badge",
            Block::Divider {} => "divider",
            Block::Spacer { .. } => "spacer",
            Block::Custom { .. } => "custom",
        }
    }
}

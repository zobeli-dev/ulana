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
    /// A surface with an optional title. A tone tints it.
    Card {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<Tone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        padding: Option<Space>,
        #[serde(default)]
        children: Vec<Block>,
    },
    /// Children in equal columns, row by row.
    Grid {
        /// 1 to 6.
        columns: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gap: Option<Space>,
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
        /// An icon before the title.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<IconName>,
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
        /// An icon before the label.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<IconName>,
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
    /// A text input. The host keeps what is typed until the next frame.
    Field {
        #[serde(default)]
        value: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<FieldKind>,
        /// Gets the text after every change.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<Action>,
        /// Gets the text when the user presses return.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        submit: Option<Action>,
    },
    /// One of a few options, side by side (a segmented control).
    Choice {
        /// 2 to 8 options.
        options: Vec<String>,
        /// One of the options.
        value: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        /// Gets the option picked.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<Action>,
    },
    /// An icon from a fixed set, drawn by the host (SF Symbols on Apple platforms).
    Icon {
        name: IconName,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<Tone>,
        /// Sized like text in this style.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style: Option<TextStyle>,
        /// What it means, for screen readers. Without it the icon is decoration.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// A picture from an address the host allows.
    Image {
        src: String,
        /// What it shows, for screen readers. Required.
        alt: String,
        /// Width divided by height, 0.25 to 4. Defaults to 16:9.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        aspect: Option<f64>,
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

/// Text sizes, largest first. `display` is for a hero number or a headline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    Display,
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
pub enum FieldKind {
    Text,
    Number,
    Email,
    Search,
}

/// The icons a view may name. Each host maps them to its own set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IconName {
    Add,
    Remove,
    Close,
    Check,
    ChevronLeft,
    ChevronRight,
    Search,
    Settings,
    Info,
    Warning,
    Error,
    Star,
    Heart,
    Bookmark,
    Calendar,
    Clock,
    Location,
    Person,
    People,
    Mail,
    Phone,
    Message,
    Home,
    Bell,
    Sun,
    Moon,
    Cloud,
    Rain,
    Snow,
    Wind,
    Bolt,
    Cart,
    Gift,
    Music,
    Camera,
    Photo,
    Link,
    Share,
    Trash,
    Edit,
    Refresh,
    Lock,
    Flag,
    Play,
    Pause,
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
            | Block::Grid { children, .. }
            | Block::List { children } => children,
            _ => &[],
        }
    }

    pub(crate) fn children_mut(&mut self) -> Option<&mut Vec<Block>> {
        match self {
            Block::Stack { children, .. }
            | Block::Row { children, .. }
            | Block::Card { children, .. }
            | Block::Grid { children, .. }
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
            Block::Grid { .. } => "grid",
            Block::Field { .. } => "field",
            Block::Choice { .. } => "choice",
            Block::Icon { .. } => "icon",
            Block::Image { .. } => "image",
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

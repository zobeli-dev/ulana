# Writing an Ulana view

This page is written to be given to a model (or a person) that writes views. It is the whole API.

A view is a Luau script. It returns a **function** that takes the host's `data` and returns a **block**. The host calls it again after every event. A script may also return a block directly when nothing changes.

```luau
return function(data)
	local count = ui.state("count", 0)
	return ui.card{ title = "Counter",
		ui.text{ tostring(count:get()), style = "title" },
		ui.button{ "More", on_press = function()
			count:update(function(n) return n + 1 end)
		end },
	}
end
```

## Rules

- Luau only: `string`, `table`, `math`, `utf8`, `bit32`, and `os.time`, `os.clock`, `os.date`. There is no `require`, `io`, network, or file access.
- A call has about 100 ms and 16 MB. A tree has at most 2000 blocks, nested at most 32 deep. Text longer than 4000 characters is cut.
- Spacing, colours, and text sizes are **tokens**, never numbers or hex codes. The host's theme decides how they look.
- Unknown fields are errors, so check spelling. An error names where it happened, such as `at root.children[2]: unknown field 'colour'`.
- Text from `data` is shown as text. It is never run.

## Blocks

Positional entries come first; named fields follow. A block that takes text can also be called with a plain string, as in `ui.text("Hi")`.

| Call | Positional | Named fields |
| --- | --- | --- |
| `ui.stack{ … }` | children, top to bottom | `gap`, `padding` (space), `align` |
| `ui.row{ … }` | children, start to end | `gap` (default `"sm"`), `padding`, `align` (default `"center"`), `justify`, `wrap` (boolean) |
| `ui.card{ … }` | children | `title`, `tone` (tints the card), `padding` (space, default `"md"`) |
| `ui.grid{ … }` | children, in rows | `columns` (1 to 6, required), `gap` (space) |
| `ui.list{ … }` | children, with dividers between (use `ui.item`) | |
| `ui.text{ … }` | the text | `style`, `tone`, `lines` (cut after this many) |
| `ui.item{ … }` | the title | `icon`, `subtitle`, `trailing` (text at the end), `on_press` or `action` |
| `ui.button{ … }` | the label | `on_press` or `action` (one is required), `icon`, `style` |
| `ui.toggle{ … }` | the label | `value` (boolean, required), `on_change` (gets the new value) or `action` |
| `ui.field{ … }` | the label | `value` (text), `placeholder`, `kind`, `on_change` (gets the text after every change), `on_submit` (gets the text when return is pressed) |
| `ui.choice{ … }` | the options, 2 to 8 | `value` (one of the options, required), `label`, `on_change` (gets the option picked) |
| `ui.icon{ … }` or `ui.icon(name)` | the icon's name | `tone`, `style` (sized like text in that style), `label` (what it means, for screen readers) |
| `ui.image{ … }` | | `src` and `alt` (what it shows), both required; `aspect` (width / height, 0.25 to 4, default 16/9). Only addresses the app allows work. |
| `ui.progress{ … }` | the value, 0 to 1 | `label`, `tone` |
| `ui.badge{ … }` | the text | `tone` |
| `ui.divider()` | | |
| `ui.spacer{ … }` | | `size` (space); without it, takes all the room left |
| `ui.custom(name, props)` | | a block the host adds, such as `ui.custom("calendar", { range = "today" })`. Only names the host lists work. |

Children may be blocks, strings or numbers (shown as text), `nil` or `false` (skipped, so `cond and ui.text("x")` works), or a list of blocks (spread in place, so a list built in a loop can be passed as one child).

## Tokens

| Token | Values |
| --- | --- |
| space | `"none"`, `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"` |
| `align` | `"start"`, `"center"`, `"end"`, `"stretch"` |
| `justify` | `"start"`, `"center"`, `"end"`, `"between"` |
| `style` (text) | `"display"` (biggest: a hero number or headline), `"title"`, `"heading"`, `"body"`, `"caption"`, `"mono"` |
| `tone` | `"default"`, `"muted"`, `"accent"`, `"success"`, `"warning"`, `"danger"` |
| `style` (button) | `"primary"`, `"secondary"`, `"plain"` |
| `kind` (field) | `"text"`, `"number"`, `"email"`, `"search"` |
| icon names | `add`, `remove`, `close`, `check`, `chevron_left`, `chevron_right`, `search`, `settings`, `info`, `warning`, `error`, `star`, `heart`, `bookmark`, `calendar`, `clock`, `location`, `person`, `people`, `mail`, `phone`, `message`, `home`, `bell`, `sun`, `moon`, `cloud`, `rain`, `snow`, `wind`, `bolt`, `cart`, `gift`, `music`, `camera`, `photo`, `link`, `share`, `trash`, `edit`, `refresh`, `lock`, `flag`, `play`, `pause` |

## Events and state

`on_press` and `on_change` take a function. The host runs it when the user presses or changes the block, then calls the view's function again.

`ui.state(name, initial)` is a value the host keeps across renders, reloads, and app restarts. It holds JSON only: `nil`, booleans, numbers, strings, and tables that are either a list or have text keys.

- `s:get()` returns a **copy**. Change it, then save it with `s:set(value)`.
- `s:update(fn)` saves `fn(current)`.
- Create state at the top of the function, the same names every time.

```luau
local todos = ui.state("todos", {})
ui.button{ "Add", on_press = function()
	todos:update(function(list)
		table.insert(list, { text = "New", done = false })
		return list
	end)
end }
```

## Patterns

**Tabs or filters**: keep the choice in state and draw what it picks.

```luau
local tab = ui.state("tab", "Today")
return ui.stack{ gap = "md",
	ui.choice{ "Today", "Week", value = tab:get(), on_change = function(t) tab:set(t) end },
	tab:get() == "Today" and ui.text("Today's list") or ui.text("This week"),
}
```

**A form**: a field keeps its own text while the user types; save it in state from `on_change` or `on_submit`.

```luau
local name = ui.state("name", "")
return ui.card{ title = "Your name",
	ui.field{ "Name", value = name:get(), placeholder = "Ana", on_submit = function(t) name:set(t) end },
	name:get() ~= "" and ui.text{ "Hello, " .. name:get(), style = "heading" },
}
```

**A dashboard**: a grid of tinted cards with an icon and a big number.

```luau
ui.grid{ columns = 2, gap = "md",
	ui.card{ tone = "accent", ui.icon{ "mail", tone = "accent" }, ui.text{ "12", style = "display" }, ui.text{ "unread", tone = "muted" } },
	ui.card{ tone = "success", ui.icon{ "check", tone = "success" }, ui.text{ "5", style = "display" }, ui.text{ "done", tone = "muted" } },
}
```

**Images**: the app lists the addresses it shows images from; any other address is refused. When it allows `https://picsum.photos/`, `https://picsum.photos/seed/<word>/<width>/<height>` gives a steady photo for a word.

## Host actions

`ui.host(name, value)` asks the host app to do something it named, such as sending a message: `ui.button{ "Ask", action = ui.host("send", "What's on today?") }`. Only names the host lists work. The host decides what happens, including asking the user first.

## A full example

```luau
return function(data)
	local compact = ui.state("compact", false)
	local rows = {}
	for _, event in data.events do
		table.insert(rows, ui.item{ event.title,
			subtitle = not compact:get() and event.place or nil,
			trailing = event.time })
	end
	return ui.stack{ gap = "md",
		ui.row{ justify = "between",
			ui.text{ "Today", style = "heading" },
			ui.badge{ #data.events .. " events", tone = "accent" },
		},
		ui.toggle{ "Compact", value = compact:get(), on_change = function(on) compact:set(on) end },
		#rows > 0 and ui.list{ rows } or ui.text{ "Nothing today.", tone = "muted" },
	}
end
```

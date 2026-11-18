-- The `ui` library every view sees. It only builds plain tables; the host
-- checks them in Rust before anything is drawn. Loaded before the sandbox
-- turns on, so `ui` is read-only to scripts and the internals stay private.

local state_get, state_set, state_init = ...

local handlers = {}
local next_id = 0

local function begin()
	handlers = {}
	next_id = 0
end

local function handler(id)
	return handlers[id]
end

local function event(fn, field)
	if type(fn) ~= "function" then
		error(field .. " must be a function", 4)
	end
	next_id += 1
	local id = "h" .. next_id
	handlers[id] = fn
	return { kind = "event", id = id }
end

local function is_block(v)
	return type(v) == "table" and type(v.type) == "string"
end

local function last_index(list)
	local n = 0
	for k in pairs(list) do
		if type(k) == "number" and k > n then
			n = k
		end
	end
	return n
end

-- Positional entries become children. `nil` and `false` are skipped, so
-- `cond and ui.text("x")` works; text becomes a text block; a plain list of
-- blocks (from a loop) is flattened in place.
local function add_children(out, list, depth)
	for i = 1, last_index(list) do
		local v = list[i]
		if v == nil or v == false then
			-- skipped
		elseif is_block(v) then
			table.insert(out, v)
		elseif type(v) == "string" or type(v) == "number" then
			table.insert(out, { type = "text", text = tostring(v) })
		elseif type(v) == "table" then
			if depth > 8 then
				error("lists of children nest too deep", 5)
			end
			add_children(out, v, depth + 1)
		else
			error("a child must be a block, text, or a list of blocks, not a " .. type(v), 5)
		end
	end
end

local function props_of(kind, p)
	if p == nil then
		return {}
	end
	if type(p) == "string" or type(p) == "number" then
		return { tostring(p) }
	end
	if type(p) ~= "table" then
		error("ui." .. kind .. " takes a table, not a " .. type(p), 4)
	end
	return p
end

local function named(kind, p, skip)
	local block = { type = kind }
	for k, v in pairs(p) do
		if type(k) == "string" and not (skip and skip[k]) then
			block[k] = v
		end
	end
	return block
end

local function container(kind, p)
	p = props_of(kind, p)
	local block = named(kind, p, { children = true })
	local children = {}
	add_children(children, p, 0)
	if p.children ~= nil then
		if type(p.children) ~= "table" then
			error("children must be a list", 3)
		end
		add_children(children, p.children, 0)
	end
	block.children = children
	return block
end

-- A block whose first positional entry is its text, title, or label.
local function labeled(kind, field, p, on, on_field)
	p = props_of(kind, p)
	local skip = {}
	if on then
		skip[on] = true
	end
	local block = named(kind, p, skip)
	if block[field] == nil and p[1] ~= nil then
		block[field] = p[1]
	end
	if type(block[field]) == "number" then
		block[field] = tostring(block[field])
	end
	if on and p[on] ~= nil then
		if p[on_field] ~= nil then
			error("give " .. kind .. " either " .. on .. " or " .. on_field .. ", not both", 3)
		end
		block[on_field] = event(p[on], on)
	end
	return block
end

local ui = {}

function ui.stack(p)
	return container("stack", p)
end

function ui.row(p)
	return container("row", p)
end

function ui.card(p)
	return container("card", p)
end

function ui.list(p)
	return container("list", p)
end

function ui.text(p)
	return labeled("text", "text", p)
end

function ui.badge(p)
	return labeled("badge", "text", p)
end

function ui.item(p)
	return labeled("item", "title", p, "on_press", "action")
end

function ui.button(p)
	return labeled("button", "label", p, "on_press", "action")
end

function ui.toggle(p)
	return labeled("toggle", "label", p, "on_change", "action")
end

function ui.progress(p)
	if type(p) == "number" then
		p = { p }
	end
	p = props_of("progress", p)
	local block = named("progress", p)
	if block.value == nil then
		block.value = p[1]
	end
	return block
end

function ui.divider()
	return { type = "divider" }
end

function ui.spacer(p)
	return named("spacer", props_of("spacer", p))
end

-- A block the host app adds, such as `ui.custom("calendar", { range = "today" })`.
function ui.custom(name, props)
	if type(name) ~= "string" then
		error("ui.custom needs the block's name", 2)
	end
	return { type = "custom", name = name, props = props or {} }
end

-- Ask the host app to do something it named: `action = ui.host("send", "What's on today?")`.
function ui.host(name, value)
	if type(name) ~= "string" then
		error("ui.host needs the action's name", 2)
	end
	return { kind = "host", name = name, value = value }
end

-- State the host keeps across renders and reloads. `get` returns a copy;
-- call `set` to save a change.
function ui.state(name, initial)
	if type(name) ~= "string" or name == "" then
		error("ui.state needs a name", 2)
	end
	state_init(name, initial)
	local s = {}
	function s:get()
		return state_get(name)
	end
	function s:set(value)
		state_set(name, value)
	end
	function s:update(fn)
		state_set(name, fn(state_get(name)))
	end
	return s
end

return ui, { begin = begin, handler = handler }

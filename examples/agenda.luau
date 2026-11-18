-- Today's agenda from data the host passes, a host-provided calendar block,
-- and a button that asks the host to send a message.
-- Run: ulana render examples/agenda.luau --data examples/agenda.json --block calendar --action send
return function(data)
	local compact = ui.state("compact", false)

	local rows = {}
	for _, event in data.events do
		table.insert(rows, ui.item{
			event.title,
			subtitle = not compact:get() and event.place or nil,
			trailing = event.time,
		})
	end

	return ui.stack{ gap = "md", padding = "md",
		ui.row{ justify = "between", align = "center",
			ui.text{ "Today", style = "heading" },
			ui.badge{ #data.events .. " events", tone = "accent" },
		},
		ui.toggle{ "Compact", value = compact:get(), on_change = function(on) compact:set(on) end },
		#rows > 0 and ui.list{ rows } or ui.text{ "Nothing today.", tone = "muted" },
		ui.custom("calendar", { range = "today" }),
		ui.button{ "What should I prepare?", style = "plain", action = ui.host("send", "What should I prepare for today?") },
	}
end

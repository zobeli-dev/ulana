-- A list kept in state: add, tick off, and clear what is done.
return function()
	local todos = ui.state("todos", {})
	local items = todos:get()

	local rows = {}
	local done = 0
	for i, todo in items do
		if todo.done then done += 1 end
		table.insert(rows, ui.toggle{ todo.text, value = todo.done, on_change = function(on)
			todos:update(function(list) list[i].done = on return list end)
		end })
	end

	return ui.stack{ gap = "md",
		ui.text{ "To do", style = "heading" },
		#items > 0 and ui.progress{ done / #items, label = done .. " of " .. #items .. " done" },
		rows,
		ui.row{ gap = "sm",
			ui.button{ "Add one", on_press = function()
				todos:update(function(list)
					table.insert(list, { text = "Task " .. (#list + 1), done = false })
					return list
				end)
			end },
			done > 0 and ui.button{ "Clear done", style = "secondary", on_press = function()
				todos:update(function(list)
					local left = {}
					for _, t in list do if not t.done then table.insert(left, t) end end
					return left
				end)
			end },
		},
	}
end

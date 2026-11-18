-- A number that goes up and down. State survives a reload of this file.
return function()
	local count = ui.state("count", 0)

	return ui.card{ title = "Counter",
		ui.text{ tostring(count:get()), style = "title" },
		ui.row{ gap = "sm",
			ui.button{ "Less", style = "secondary", on_press = function()
				count:update(function(n) return n - 1 end)
			end },
			ui.button{ "More", on_press = function()
				count:update(function(n) return n + 1 end)
			end },
		},
	}
end

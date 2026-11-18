-- Weather for a city you type: a field, a unit choice, icons, a photo, and a
-- grid of tinted cards. The forecast is made up from the city's name, so the
-- view needs no data. Serve with --image https://picsum.photos/ to allow the photo.
return function()
	local city = ui.state("city", "Lisbon")
	local unit = ui.state("unit", "°C")

	-- A steady pretend forecast for any city name.
	local seed = 0
	for i = 1, #city:get() do
		seed += string.byte(city:get(), i) * i
	end
	local function temp(c)
		if unit:get() == "°F" then
			return math.floor(c * 9 / 5 + 32 + 0.5) .. "°"
		end
		return c .. "°"
	end
	local skies = { { "sun", "Sunny", "warning" }, { "cloud", "Cloudy", "muted" }, { "rain", "Rain", "accent" } }
	local days = { "Mon", "Tue", "Wed", "Thu", "Fri" }

	local today = 14 + seed % 12
	local sky = skies[seed % #skies + 1]
	local rows = {}
	for i, day in days do
		local s = skies[(seed + i) % #skies + 1]
		table.insert(rows, ui.item{ day, icon = s[1], subtitle = s[2], trailing = temp(today + (i * 3 + seed) % 7 - 3) })
	end

	return ui.stack{ gap = "md",
		ui.field{ "City", value = city:get(), placeholder = "Type a city", kind = "search",
			on_submit = function(t)
				if t ~= "" then city:set(t) end
			end },
		ui.choice{ "°C", "°F", value = unit:get(), on_change = function(u) unit:set(u) end },
		ui.image{ src = "https://picsum.photos/seed/" .. string.lower(city:get()) .. "/800/400", alt = "A photo for " .. city:get(), aspect = 2 },
		ui.grid{ columns = 2, gap = "sm",
			ui.card{ tone = sky[3],
				ui.icon{ sky[1], tone = sky[3], style = "title", label = sky[2] },
				ui.text{ temp(today), style = "display" },
				ui.text{ sky[2] .. " in " .. city:get(), tone = "muted" },
			},
			ui.card{ tone = "accent",
				ui.icon{ "wind", tone = "accent", style = "title" },
				ui.text{ (seed % 30 + 5) .. " km/h", style = "title" },
				ui.text{ "Wind", tone = "muted" },
			},
		},
		ui.card{ title = "Next days", ui.list{ rows } },
	}
end

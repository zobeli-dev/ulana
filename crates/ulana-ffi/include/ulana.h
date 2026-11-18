// A C interface to the Ulana runtime. Every call that can fail returns a JSON
// string the caller frees with ulana_string_free:
//
//   {"ok": <value>}                          on success
//   {"error": {"kind": "...", "message": "..."}}   on failure
//
// A view is not thread-safe: call it from one thread at a time.

#ifndef ULANA_H
#define ULANA_H

#include <stdint.h>

typedef struct UlanaView UlanaView;

// Load a script. `host_json` is {"blocks": [...], "actions": [...], "images": [...]};
// `state_json` is the state saved earlier, or NULL. On failure returns NULL
// and, when `error_out` is not NULL, sets it to an error envelope to free.
UlanaView *ulana_view_new(const char *source, const char *host_json, const char *state_json, char **error_out);

// Run the view with `data_json` (or NULL) and return {"ok": <frame>}.
char *ulana_view_render(UlanaView *view, const char *data_json);

// Run the handler `id` from frame `version` with `value_json` (or NULL).
char *ulana_view_event(UlanaView *view, uint64_t version, const char *id, const char *value_json);

// Swap the script and keep the state.
char *ulana_view_reload(UlanaView *view, const char *source);

// {"ok": <state object>}
char *ulana_view_state(UlanaView *view);

// {"ok": ["printed line", ...]}, the lines printed since the last call.
char *ulana_view_take_logs(UlanaView *view);

void ulana_view_free(UlanaView *view);
void ulana_string_free(char *string);

#endif

# Letting a model write views

Ulana's API is small and its errors name what went wrong, so a model can write a view, see it fail, and fix it. This guide covers what to send, how to check what comes back, and what keeps the app safe. The Expo example's [`src/ai.ts`](../examples/expo/src/ai.ts) does all of it in about 200 lines, and its tests in `test/change.test.ts` show the loop with a fake model.

## What to send

1. **The API.** Send [`spec/ulana.md`](../spec/ulana.md) whole. It is written for a model: every block, token, and icon name, with patterns. `ulana serve` serves it at `GET /host`.
2. **What your app accepts.** List the custom blocks, host actions, and image addresses your `Host` allows, and say there are no others. `GET /host` returns these too.
3. **What the tones look like.** A model asked for "orange" cannot know your `warning` tone is amber. Send each tone's colour (`accent #2563EB, warning #B45309, …`) and tell it to pick the closest.
4. **The current script and its data.** Send the whole script and the JSON its function gets, so the model keeps what the user did not ask to change and reads real fields instead of inventing them.
5. **The error, if there is one now.**
6. **The request.**

Ask for one or two plain sentences on what changed, then the whole new script in one ```` ```luau ```` block. Ask for the whole script, never a diff: whole scripts are easier for a model to keep consistent and for you to check.

Rules worth putting in the prompt:

- Change what was asked and keep the rest.
- Keep the names given to `ui.state`, so what the user entered survives the change.
- Use only the API. When the user asks for something it cannot do, do the closest thing it can, send that script, and say so.
- When the request is a question, answer it without a script.

## Checking what comes back

Take the last ```` ```luau ```` (or ```` ```lua ````) block that contains `ui.` or `return`. Load it:

- **On a server:** `View::reload(source)` keeps the state, and the old script stays if the new one fails. With `ulana serve --edit`, `PUT /views/<name>/source` does this and answers with the result.
- **On a device:** `LocalSession.reload(source:)` in Swift.

Then render it. When loading or rendering fails, send the error back as the next message ("That script does not work. Ulana says: … Send the whole script again.") and try again. Two repairs are enough in practice. Keep the script from before the change so the user can undo it.

Errors name the path and what is accepted, which is usually all a model needs:

```
at root.children[2]: unknown field `colour`, expected one of `text`, `style`, `tone`, `lines`
at root.children[0].src: this app only shows images from `https://picsum.photos/`
at root.children[1].value: value must be one of the options, not `Weekly`
```

## What keeps it safe

- **The script is sandboxed.** No network, no files, no `require`, read-only libraries, and memory and time limits. A script cannot reach your app's data or tools.
- **The tree is checked before it is drawn.** Unknown blocks, fields, custom blocks, host actions, and image addresses are refused.
- **A view can only ask.** Anything with an effect beyond the view (sending a message, paying, deleting) is a host action your app named, and your app decides what happens, including asking the user first. Keep confirmations in your app's own UI, never in a block a view can draw.
- **Data is data.** Text a view shows from its data, such as an email subject, is drawn as text. It is never run, and the view's script cannot be changed by it. When the model writing a view also reads untrusted text, treat that text as data in the prompt as well.
- **Images come only from addresses you allow,** so a view cannot load a tracking pixel from anywhere it likes.

## Choosing a model

The example uses `z-ai/glm-5.3-flash` through OpenRouter. In tests it changed views correctly on the first try in 2 to 10 seconds, for well under a tenth of a cent per change. Any chat model that follows instructions and writes code works; set `EXPO_PUBLIC_ULANA_MODEL` in the example to try another.

import { expect, test } from "bun:test";

import { split } from "../src/ai";

test("the sentence and the script come apart", () => {
  const { text, script } = split(
    "Made the count **bigger** and added a reset.\n\n```luau\nreturn function()\n\treturn ui.text(\"hi\")\nend\n```\n",
  );
  expect(text).toBe("Made the count bigger and added a reset.");
  expect(script).toBe('return function()\n\treturn ui.text("hi")\nend\n');
});

test("a plain or lua fence works, and the last script wins", () => {
  const { script } = split("First:\n```lua\nreturn ui.text('a')\n```\nBetter:\n```\nreturn ui.text('b')\n```");
  expect(script).toBe("return ui.text('b')\n");
});

test("an answer without a script is only an answer", () => {
  expect(split("A toggle keeps its value in ui.state.")).toEqual({
    text: "A toggle keeps its value in ui.state.",
    script: null,
  });
});

test("a code block that is not a view is not taken for one", () => {
  expect(split("The data looks like:\n```json\n{ \"events\": [] }\n```").script).toBeNull();
});

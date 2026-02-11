/*
import { describe, it, expect } from "vitest";
import { playbookLanguage } from "../src/index.js";
import { tagHighlighter } from "@lezer/highlight";

describe("Playbook Highlighting", () => {
  it("assigns tags correctly to keywords", () => {
    const input = "players";
    const tree = playbookLanguage.parser.parse(input);
    const playersNode = tree.topNode.firstChild?.firstChild; // Program -> TopLevel -> Players
    expect(playersNode?.name).toBe("Players");

    const kwNode = playersNode?.firstChild; // Players -> "players"
    expect(kwNode?.name).toBe("players");

    // Check if the keyword node has the keyword tag
    console.log(kwNode?.type);
    console.log(tagHighlighter.name);
    const tag = kwNode?.type.is(tagHighlighter.name);
    expect(tag).toBeTruthy();
  });

  it("assigns tags correctly to numbers", () => {
    const input = "123";
    const tree = playbookLanguage.parser.parse(input);
    const numNode = tree.topNode.firstChild?.firstChild; // Program -> TopLevel -> Number
    expect(numNode?.name).toBe("Number");

    const tag = numNode?.type.is(styleTags.name);
    expect(tag).toBeTruthy();
  });
});
*/

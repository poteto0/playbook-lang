import { describe, it, expect } from "vitest";
import { parser } from "../src/syntax.js";

describe("Playbook Grammar", () => {
  it("parses identifiers", () => {
    const input = "p1";
    const tree = parser.parse(input);
    expect(tree.topNode.name).toBe("Program");
    // Program -> TopLevel -> Identifier
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.name).toBe("TopLevel");
    expect(topLevel?.firstChild?.name).toBe("Identifier");
  });

  it("parses numbers", () => {
    const input = "100";
    const tree = parser.parse(input);
    expect(tree.topNode.name).toBe("Program");
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.name).toBe("TopLevel");
    expect(topLevel?.firstChild?.name).toBe("Number");
  });

  it("parses players block", () => {
    const input = "players = { p1, p2 }";
    const tree = parser.parse(input);
    expect(tree.topNode.name).toBe("Program");
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.name).toBe("TopLevel");

    // TopLevel -> Players
    const playersNode = topLevel?.firstChild;
    expect(playersNode?.name).toBe("Players");
  });

  it("parses state block", () => {
    const input = `state = {
      baller = p1,
      position = {
        p1 = (0, 60),
        p2 = (90, -80),
      },
    }`;
    const tree = parser.parse(input);
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("State");
  });

  it("parses action block", () => {
    const input = `action = {
      pass = {
        p1 -> p2,
      }
    }`;
    const tree = parser.parse(input);
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("ActionBlock");
  });

  it("parses actions block (list)", () => {
    const input = `actions = [
      action = { move = { p1 -> (0, 40) } },
      action = { pass = { p1 -> p2:after } }
    ]`;
    const tree = parser.parse(input);
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("ActionsBlock");
  });

  it("parses defenders block", () => {
    const input = "defenders = { d1, d2 }";
    const tree = parser.parse(input);
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("Defenders");
  });

  it("parses state defense block (mark and fixed position)", () => {
    const input = `state = {
      position = { p1 = (0, 60) },
      defense = {
        d1 -> p1,
        d2 = (-90, -80),
        d3 -[7]> p1,
      },
    }`;
    const tree = parser.parse(input);
    expect(tree.toString()).not.toContain("⚠");
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("State");
  });

  it("parses action defense block (explicit offset mark)", () => {
    const input = `action = {
      defense = {
        d1 -[15]> p1,
      }
    }`;
    const tree = parser.parse(input);
    expect(tree.toString()).not.toContain("⚠");
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("ActionBlock");
  });

  it("parses action defense block (fixed position)", () => {
    const input = `action = {
      defense = {
        d1 -> (70, 20),
      }
    }`;
    const tree = parser.parse(input);
    expect(tree.toString()).not.toContain("⚠");
    const topLevel = tree.topNode.firstChild;
    expect(topLevel?.firstChild?.name).toBe("ActionBlock");
  });
});

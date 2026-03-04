---
sidebar_position: 4
---

# CodeMirror Extension

`@playbook-lang/code-mirror` provides syntax highlighting and language support for Playbook in CodeMirror 6.

## Installation

```bash
npm install @playbook-lang/code-mirror
```

## Usage

You can use the `playbook()` function to add Playbook language support to your CodeMirror instance.

```typescript
import { EditorView, basicSetup } from "codemirror";
import { playbook } from "@playbook-lang/code-mirror";

new EditorView({
  doc: "players = { p1 }",
  extensions: [
    basicSetup,
    playbook(),
  ],
  parent: document.body,
});
```

## Features

- **Syntax Highlighting**: Proper highlighting for players, positions, actions, and comments.
- **Auto-completion**: Suggestions for keywords and player IDs.
- **Linter Integration**: (Optional) You can integrate with the Playbook linter to show errors directly in the editor.

## Example: Integrating with Linter

If you use the `@playbook-lang/core` wasm package, you can create a linter for CodeMirror:

```typescript
import { linter, Diagnostic } from "@codemirror/lint";
import { render_playbook } from "@playbook-lang/core";

const playbookLinter = linter((view) => {
  const doc = view.state.doc.toString();
  try {
    render_playbook(doc);
    return [];
  } catch (e: any) {
    const errors = parsePlayBookErrors(e); // See Error Handling section
    return errors.map((err) => ({
      from: view.state.doc.line(err.line).from + err.column,
      to: view.state.doc.line(err.line).from + err.column + err.length,
      severity: "error",
      message: err.message,
    } as Diagnostic));
  }
});
```

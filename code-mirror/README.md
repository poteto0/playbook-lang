# @playbook-lang/code-mirror

CodeMirror extension for the Playbook language.

## Installation

```bash
npm install @playbook-lang/code-mirror
```

## Usage

```javascript
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

## Development

```bash
npm install
npm run build
```

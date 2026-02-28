import { EditorView, basicSetup } from "codemirror";
import { oneDark } from "@codemirror/theme-one-dark";
import { playbook } from "@poteto0/playbook-lang-syntax";
import init from "@poteto0/playbook-lang-linter";
import { playbookLinter } from "./linter";

const initialText = `players = { p1, p2, p3, p4, p5 }

state = {
  baller = p1,
  position = {
    p1 = (0, 60),
    p2 = (90, -80),
    p3 = (-90, -80),
    p4 = (50, -10),
    p5 = (-50, -10),
  },
}

actions = [
  action = {
    move = {
      p5 -> (-20, 40),
    }
  },

  action = {
    move = {
      p2 ~> (70, 20),
      p3 -> (0, -80),
    },
    screen = {
      p4 -> p2:middle,
      p5 ~[r]> (-60, -80)
    },
    pass = {
      p1 -> p2:after,
    },
  }
]
`;

async function main() {
  console.log("Initializing Playbook Editor...");
  
  try {
    // Initialize WASM linter
    await init();
    console.log("WASM Linter initialized.");

    const container = document.getElementById("editor-container");

    if (container) {
      new EditorView({
        doc: initialText,
        extensions: [
            basicSetup, 
            oneDark, 
            playbook(),
            playbookLinter
        ],
        parent: container,
      });
      console.log("Playbook EditorView created.");
    } else {
      console.error("Editor container not found!");
    }
  } catch (e) {
    console.error("Failed to initialize editor:", e);
    const container = document.getElementById("editor-container");
    if (container) {
       container.innerHTML = `<pre style="color: red; padding: 1rem;">Failed to initialize: ${e}</pre>`;
    }
  }
}

main();

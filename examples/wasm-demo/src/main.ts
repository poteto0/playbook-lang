// @ts-ignore
import init, { render_playbook } from "./pkg/playbook_lang_core.js";

async function main() {
  await init();

  const inputEl = document.getElementById("input") as HTMLTextAreaElement;
  const outputEl = document.getElementById("output") as HTMLDivElement;

  const render = () => {
    try {
      const svg = render_playbook(inputEl.value);
      outputEl.innerHTML = svg;
    } catch (e) {
      outputEl.innerHTML = `<pre style="color: red;">Error: ${e}</pre>`;
    }
  };

  inputEl.addEventListener("input", render);

  // Initial render
  render();
}

main();

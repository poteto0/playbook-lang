---
sidebar_position: 1
---

# Wasm

You can use playbook-lang on your WebApp (javascript runtime).

EX) Cloudflare Workers

In fact, the web playground utilizes this.

## QuickStart

You can refer to the following for QuickStart.

- [wasm-demo](https://github.com/poteto0/playbook-lang/tree/main/examples/wasm-demo)

## Step By Step

1. clone the repository

   ```bash
   git clone git@github.com:poteto0/playbook-lang.git
   # or
   git clone https://github.com/poteto0/playbook-lang.git
   ```

2. select version

   [Available Versions](https://github.com/poteto0/playbook-lang/tags)

   ```bash
   git switch -d <version>
   ```

3. build wasm

   ```bash
   just release-wasm
   ```

4. copy to your project
   - `playbook_lang_core.js`
   - `playbook_lang_core_bg.wasm.d.ts`
   - `playbook_lang_core.d.ts`
   - `playbook_lang_core_bg.wasm`

5. configure

   ### `Vite CloudflareWorkers + React + typescript`

   use sync init

   ```tsx title="App.tsx"
   import initSync, { render_playbook } from "./pkg/playbook_lang_core.js";

   /* ... */
   function App() {
     useEffect(() => {
       initSync();
     });

     // on your handler
     compile() => {
      const output = render_playbook(e.target.value);
      /* ... */
     }

     /* ... */
   }
   ```

   ### `Vite + typescript + html`

   configure

   ```ts title="vite.config.ts"
   import { defineConfig } from "vite";

   export default defineConfig({
     assetsInclude: ["**/*.wasm"],
     server: {
       fs: {
         allow: [".."],
       },
     },
   });
   ```

   in your main function

   ```ts title="main.ts"
   // @ts-ignore
   import init, { render_playbook } from "./pkg/playbook_lang_core.js";

   async function main() {
     // initialize wasm
     await init();

     // get compiled svg
     const svg = render_playbook(inputEl.value);

     /* ... */
   }
   ```

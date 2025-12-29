# Playbook WASM Demo

This example demonstrates how to use the `playbook-lang-core` WebAssembly module in a TypeScript application.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/)

## Setup and Run

1.  **Build the WASM module**

    First, you need to build the `core` crate as a WebAssembly package. Run the following command from the project root:

    ```bash
    cd core
    wasm-pack build --target web
    ```

    This will generate the WASM package in `core/pkg`.

2.  **Install dependencies**

    Navigate to this example directory and install the dependencies:

    ```bash
    cd ../examples/wasm-demo
    npm install
    ```

3.  **Run the development server**

    Start the Vite development server:

    ```bash
    npm run dev
    ```

    Open your browser at the URL shown (usually `http://localhost:5173`). You should see the Playbook editor on the left and the rendered SVG on the right.

## Notes

- The `src/main.ts` file imports the WASM module from `../../../core/pkg/playbook_lang_core.js`. Ensure you have built the WASM module before running the dev server.
- We use `vite-plugin-wasm` and `vite-plugin-top-level-await` to handle WASM loading and top-level await support.

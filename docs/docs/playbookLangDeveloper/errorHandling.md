---
sidebar_position: 3
---

# Error Handling (wasm call)

When a compilation error occurs, it returns an error message with the following json array structure w/ prefix(`[Error]:`):

```json
[
  {
    "line": number,
    "column": number,
    "length": number,
    "message": string,
    "found": string,
  },
  ...
]
```

EX)

```
[Error]:[{"line":24, "column":10, "length":1, "message":"Expected -> or ~>", "found":"^"}]
```

Therefore, you can handle it as follows in your TypeScript application:

```ts title="parsePlayBookError.ts"
export interface PlayBookError {
  line: number;
  column: number;
  length: number;
  message: string;
  found: string;
}

export function parsePlayBookErrors(
  errorString: string,
): PlayBookError[] {
  const prefix = "[Error]:";
  if (errorString.startsWith(prefix)) {
    try {
      const jsonString = errorString.substring(prefix.length);
      return JSON.parse(jsonString) as PlayBookError[];
    } catch (e) {
      return [];
    }
  }
  return [];
}
```

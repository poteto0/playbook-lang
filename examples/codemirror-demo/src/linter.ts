import { linter, Diagnostic } from "@codemirror/lint";
import { EditorView } from "@codemirror/view";
import { lint_playbook } from "@poteto0/playbook-lang-linter";

export const playbookLinter = linter((view: EditorView) => {
  const doc = view.state.doc.toString();
  try {
    const rawDiagnostics = lint_playbook(doc);
    // console.log("Linter raw output:", rawDiagnostics); // For debugging
    
    // Check if rawDiagnostics is array
    if (!Array.isArray(rawDiagnostics)) {
        return [];
    }

    return rawDiagnostics.map((d: any) => {
      const from = posToOffset(view.state.doc, d.line, d.column);
      const to = from + 1; // Highlight at least one char
      return {
        from,
        to,
        severity: d.severity === "error" ? "error" : "warning",
        message: d.message,
      } as Diagnostic;
    });
  } catch (e) {
    console.error("Linter error:", e);
    return [];
  }
});

function posToOffset(doc: any, line: number, column: number): number {
  try {
      if (line < 1 || line > doc.lines) return 0;
      const lineInfo = doc.line(line);
      const col = Math.min(Math.max(0, column - 1), lineInfo.length); 
      return lineInfo.from + col;
  } catch (e) {
      console.error("Error calculating offset:", e);
      return 0;
  }
}

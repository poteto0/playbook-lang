import { parser } from "./syntax.js";
import { LRLanguage, LanguageSupport } from "@codemirror/language";
export const playbookLanguage = LRLanguage.define({
    parser: parser.configure({
        props: [
        // We'll add highlighting props later
        ]
    }),
    languageData: {
        commentTokens: { line: "//" }
    }
});
export function playbook() {
    return new LanguageSupport(playbookLanguage);
}

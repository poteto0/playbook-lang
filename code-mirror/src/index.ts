import { parser } from "./syntax.js";
import { LRLanguage, LanguageSupport } from "@codemirror/language";
import { styleTags, tags as t } from "@lezer/highlight";

export const playbookLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      styleTags({
        "players defenders state baller position defense action actions move screen pass before after middle":
          t.keyword,
        Identifier: t.variableName,
        Number: t.number,
        Coordinate: t.atom,
        "=": t.definitionOperator,
        "{ } [ ] ( )": t.paren,
        ",": t.separator,
        ":": t.punctuation,
        "-> ~> CurveArrow OffsetArrow": t.operator,
        LineComment: t.lineComment,
      }),
    ],
  }),
  languageData: {
    commentTokens: { line: "//" },
  },
});

export function playbook() {
  return new LanguageSupport(playbookLanguage);
}

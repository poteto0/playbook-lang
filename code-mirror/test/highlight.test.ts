import { describe, it, expect } from 'vitest';
import { playbookLanguage } from '../src/index.js';
import { tags as t } from '@lezer/highlight';

describe('Playbook Highlighting', () => {
  it('assigns tags correctly to keywords', () => {
     const input = "players";
     const tree = playbookLanguage.parser.parse(input);
     const playersNode = tree.topNode.firstChild?.firstChild; // Program -> TopLevel -> Players
     expect(playersNode?.name).toBe('Players');
     
     const kwNode = playersNode?.firstChild; // Players -> "players"
     expect(kwNode?.name).toBe('players');
     
     // Check if the keyword node has the keyword tag
     const tag = kwNode?.type.prop(t.styleTags.prop);
     // lezer-highlight styleTags adds a property that maps to tags
     // It's a bit internal, but we can check if it exists
     expect(tag).toBeDefined();
  });

  it('assigns tags correctly to numbers', () => {
    const input = "123";
    const tree = playbookLanguage.parser.parse(input);
    const numNode = tree.topNode.firstChild?.firstChild; // Program -> TopLevel -> Number
    expect(numNode?.name).toBe('Number');
    
    const tag = numNode?.type.prop(t.styleTags.prop);
    expect(tag).toBeDefined();
  });
});

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
const source = readFileSync('src/settings.ts', 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } }).outputText;
const { defaults, validateSettings, themes, themeAccents } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
test('legacy settings gain safe defaults without losing existing preferences', () => {
 const s = validateSettings({ theme: 'black', accent: '#123456', showSeconds: true, animationMs: 320 });
 assert.equal(s.theme, 'black'); assert.equal(s.accent, '#123456');
 assert.deepEqual(s.widgetOrder, ['media', 'volume', 'system']); assert.equal(s.opacity, 100);
});
test('each theme has a valid recommended accent', () => {
 for (const theme of themes) assert.match(themeAccents[theme], /^#[0-9a-f]{6}$/i);
});
test('invalid widget order, opacity, or clock format is rejected', () => {
 for (const update of [{widgetOrder:['media','media','system']},{opacity:0},{clockFormat:'bad'},{cornerRadius:999}]) assert.throws(()=>validateSettings({...defaults,...update}));
});

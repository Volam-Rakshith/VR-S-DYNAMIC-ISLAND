import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
test('window uses privacy-conscious defaults', () => {
 const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
 const window = config.app.windows[0];
 assert.equal(window.transparent, true); assert.equal(window.alwaysOnTop, true); assert.equal(window.skipTaskbar, true);
 assert.equal(config.identifier, 'com.vrdevelopments.vrisland');
});

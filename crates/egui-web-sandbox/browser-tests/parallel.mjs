import assert from 'node:assert/strict';
import {mkdir} from 'node:fs/promises';
import {chromium} from 'playwright';
import {SandboxPage} from '../browser/client.mjs';

const base = process.env.SANDBOX_URL ?? 'http://127.0.0.1:8765';
const path = process.env.SANDBOX_DEMO_PATH ?? '/sandbox/';
const browser = await chromium.launch({headless: true,
  ...(process.env.CHROMIUM_EXECUTABLE ? {executablePath: process.env.CHROMIUM_EXECUTABLE} : {})});
const errors = [];
const contexts = [];

async function open(scale = 1, query = '') {
  const context = await browser.newContext({viewport: {width: 1400, height: 900}, deviceScaleFactor: scale,
    locale: 'en-US', timezoneId: 'UTC', colorScheme: 'light'});
  contexts.push(context);
  const page = await context.newPage();
  page.on('pageerror', error => errors.push(String(error)));
  page.on('console', message => { if (message.type() === 'error' && !message.text().includes('404')) errors.push(message.text()); });
  await page.goto(`${base}${path}${query}`);
  await page.locator('#status').waitFor({state: 'detached'});
  const app = new SandboxPage(page);
  await app.ready();
  await app.settle();
  return app;
}

const documentControl = (app, role, name, window = 'Project brief') => app.control(role, name, {window});
const title = app => documentControl(app, 'TextInput', 'Title');
const notes = app => documentControl(app, 'MultilineTextInput', 'Notes');

try {
  const apps = await Promise.all([open(), open(), open(2)]);
  await Promise.all(apps.map(async (app, worker) => {
    const state = await app.snapshot();
    assert.ok(state.bounds[2] > 0 && state.bounds[3] > 0);
    assert.ok(state.windows.some(window => window.title === 'Pipeline'));
    await assert.rejects(app.control('Button', 'Save').read(), /Scope repeated names/);
    await title(app).fill(`worker-${worker}`);
    assert.equal((await title(app).read()).focused, true);
    await title(app).fill('');
    assert.equal((await documentControl(app, 'Button', 'Save').read()).disabled, true);
    await assert.rejects(documentControl(app, 'Button', 'Save').click(), /disabled/);
    await title(app).fill(`worker-${worker}`);
    // Standard TextEdit shortcuts; no custom copy/paste buttons or clipboard injection.
    await app.page.keyboard.press('ControlOrMeta+a');
    await app.page.keyboard.press('ControlOrMeta+c');
    await app.settle();
    assert.ok((await app.snapshot()).clipboard_has_text);
    await notes(app).fill('');
    await app.page.keyboard.press('ControlOrMeta+v');
    await app.settle();
    assert.equal((await notes(app).read()).value, `worker-${worker}`);
    // A host may deliver a semantic paste alongside the real shortcut.
    await notes(app).fill('');
    await app.page.evaluate(() => {
      const pasteWithShortcut = event => {
        if (event.key.toLowerCase() !== 'v' || !(event.ctrlKey || event.metaKey)) return;
        document.removeEventListener('keydown', pasteWithShortcut, true);
        document.dispatchEvent(new ClipboardEvent('paste', {bubbles: true, cancelable: true}));
      };
      document.addEventListener('keydown', pasteWithShortcut, true);
    });
    await app.page.keyboard.press('ControlOrMeta+v');
    await app.settle();
    assert.equal((await notes(app).read()).value, `worker-${worker}`);
    await app.page.keyboard.press('ControlOrMeta+a');
    await app.page.keyboard.press('ControlOrMeta+x');
    await app.settle();
    assert.equal((await notes(app).read()).value, '');
    await app.page.keyboard.press('ControlOrMeta+v');
    await app.settle();
    assert.equal((await notes(app).read()).value, `worker-${worker}`);
    const reviewed = documentControl(app, 'CheckBox', 'Reviewed');
    assert.equal((await reviewed.read()).checked, false);
    await reviewed.click();
    assert.equal((await reviewed.read()).checked, true);
    const priority = documentControl(app, 'Slider', 'Priority');
    const numeric = await priority.read();
    assert.equal(numeric.min, 1);
    assert.equal(numeric.max, 10);
    assert.equal(numeric.numeric_value, 3);
    await priority.click();
    await app.page.keyboard.press('ArrowRight');
    await app.settle();
    assert.ok((await priority.read()).numeric_value > 3);
    await documentControl(app, 'RadioButton', 'Published').click();
    const published = await documentControl(app, 'RadioButton', 'Published').read();
    assert.ok(published.checked || published.selected);
    for (let n = 0; n < worker + 1; n++) await documentControl(app, 'Button', 'Save').click();
    const saved = (await app.snapshot()).controls.filter(control => control.label === `Saved revision: ${worker + 1}`);
    assert.equal(saved.length, 1);
    assert.ok(await app.page.evaluate(() => Object.isFrozen(window.eguiSandbox)));
  }));

  // Custom canvas widgets expose ordinary egui semantics, then use ordinary pointer input.
  const node = apps[0].control('Button', 'Input node', {window: 'Pipeline'});
  const before = (await node.read()).bounds;
  await node.dragBy(60, 15);
  const after = (await node.read()).bounds;
  // egui owns click-vs-drag tolerance; verify movement within the requested extent.
  assert.ok(after[0] > before[0] && after[0] <= before[0] + 60, JSON.stringify({before, after}));
  assert.ok(after[1] > before[1] && after[1] <= before[1] + 15, JSON.stringify({before, after}));

  // Window movement belongs to egui; no Desktop, manual rectangles or event forwarding.
  const state = await apps[0].snapshot();
  const window = state.windows.find(window => window.title === 'Project brief');
  const canvas = await apps[0].page.locator('#canvas').boundingBox();
  const x = canvas.x + (window.bounds[0] + 120) * state.zoom_factor;
  const y = canvas.y + (window.bounds[1] + 12) * state.zoom_factor;
  await apps[0].page.mouse.move(x, y);
  await apps[0].page.mouse.down();
  await apps[0].page.mouse.move(x + 20, y + 15, {steps: 10});
  await apps[0].page.mouse.up();
  await apps[0].settle();
  const moved = (await apps[0].snapshot()).windows.find(window => window.title === 'Project brief');
  // egui begins movement after its own drag threshold; do not prescribe that threshold.
  assert.ok(moved.bounds[0] > window.bounds[0] && moved.bounds[0] <= window.bounds[0] + 20);
  assert.ok(moved.bounds[1] > window.bounds[1] && moved.bounds[1] <= window.bounds[1] + 15);
  await documentControl(apps[0], 'Splitter', 'Resize window bottom-right corner').dragBy(22, 14);
  const resized = (await apps[0].snapshot()).windows.find(window => window.title === 'Project brief');
  assert.ok(resized.bounds[2] - resized.bounds[0] > moved.bounds[2] - moved.bounds[0]);

  await apps[0].control('CheckBox', 'Inspector').click();
  await apps[0].page.waitForFunction(() => window.eguiSandbox.snapshot().windows.some(window => window.title === 'Inspector'));
  await apps[0].reset();
  assert.equal((await title(apps[0]).read()).value, 'Product brief');
  assert.equal((await apps[0].snapshot()).clipboard_has_text, false);
  assert.ok(!(await apps[0].snapshot()).windows.some(window => window.title === 'Inspector'));
  assert.equal((await title(apps[1]).read()).value, 'worker-1');
  assert.equal((await title(apps[2]).read()).value, 'worker-2');

  const observed = await open(1, '?observe');
  assert.equal((await observed.snapshot()).reset_supported, false);
  assert.equal(await observed.page.evaluate(() => typeof window.eguiSandbox.reset), 'undefined');
  await assert.rejects(observed.reset(), /not enabled/);
  await title(observed).fill('observation-only');
  assert.equal((await title(observed).read()).value, 'observation-only');
  await apps[2].page.reload();
  await apps[2].ready();
  assert.equal((await title(apps[2]).read()).value, 'Product brief');
  assert.equal((await apps[2].snapshot()).clipboard_has_text, false);
  assert.equal((await title(apps[1]).read()).value, 'worker-1');
  assert.deepEqual(errors, []);
  await mkdir(new URL('./test-results/', import.meta.url), {recursive: true});
  await Promise.all([...apps, observed].map((app, index) => app.page.screenshot({path: new URL(`./test-results/worker-${index}.png`, import.meta.url).pathname})));
  console.log('PASS: ordinary egui workbench; scoped controls, checkbox/radio/slider state, standard copy/cut/paste, canvas dragging, window movement, embedded viewport, optional reset; parallel isolation and DPR 1/2; no native services');
} finally {
  await Promise.all(contexts.map(context => context.close()));
  await browser.close();
}

import assert from 'node:assert/strict';
import {chromium} from 'playwright';
import {SandboxPage} from '../browser/client.mjs';

const launcher = process.env.LAUNCHER_URL;
assert.ok(launcher, 'Run npm test to start the Notes app');
const cliFailure = process.env.CLI_FAILURE_URL;
assert.ok(cliFailure, 'Run npm test to start the sandbox command with the JS module fixture');
const browser = await chromium.launch({headless: true,
  ...(process.env.CHROMIUM_EXECUTABLE ? {executablePath: process.env.CHROMIUM_EXECUTABLE} : {})});
const context = await browser.newContext({viewport: {width: 1000, height: 720}});
const page = await context.newPage();
const errors = [];
page.on('pageerror', error => errors.push(String(error)));

try {
  await page.goto(launcher);
  await page.locator('#status').waitFor({state: 'detached'});
  const app = new SandboxPage(page);
  await app.ready();
  await app.control('TextInput', 'Title').fill('A reusable app');
  await app.control('Button', 'Save').click();
  await page.waitForFunction(() => window.eguiSandbox.snapshot().controls.some(c => c.label === 'Saved: A reusable app'));

  // A duplicate startup must preserve the currently running app.
  const duplicate = await page.evaluate(async () => {
    try { await (await import('/app.js')).start(); }
    catch (error) { return String(error); }
  });
  assert.match(duplicate, /already has a sandbox bridge/);
  assert.equal((await app.control('TextInput', 'Title').read()).value, 'A reusable app');

  // Browser history may retain a page; only a final pagehide unmounts it.
  await page.evaluate(() => window.dispatchEvent(new PageTransitionEvent('pagehide', {persisted: true})));
  assert.equal((await app.control('TextInput', 'Title').read()).value, 'A reusable app');
  await page.evaluate(() => window.dispatchEvent(new PageTransitionEvent('pagehide')));
  assert.equal(await page.evaluate(() => typeof window.eguiSandbox), 'undefined');

  await page.evaluate(async () => {
    const module = await import('/app.js');
    const previous = await module.start();
    previous.destroy();
    previous.destroy();
    const current = await module.start();
    previous.destroy();
    previous.free();
    current.free();
    const {mount} = await import('/launch.mjs');
    const stop = await mount(() => import('/app.js'));
    stop();
    stop();
    await mount(() => import('/app.js'));
  });
  await app.ready();
  assert.equal((await app.control('TextInput', 'Title').read()).value, 'First note');
  await app.control('TextInput', 'Title').fill('Restarted');
  await app.control('Button', 'Save').click();
  await page.waitForFunction(() => window.eguiSandbox.snapshot().controls.some(c => c.label === 'Saved: Restarted'));

  assert.deepEqual(errors, []);

  await page.goto(new URL('desktop.html', process.env.SITE_URL ?? launcher).href);
  const panel = page.getByRole('region', {name: 'App window'});
  const open = page.getByRole('button', {name: 'Open app'});
  assert.equal(await panel.count(), 0);
  await open.click();
  const frame = await (await page.locator('iframe').elementHandle()).contentFrame();
  const desktopApp = new SandboxPage(page, {frame});
  await desktopApp.ready();

  async function drag(name, dx, dy) {
    const box = await page.getByRole('button', {name}).boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, {steps: 6});
    await page.mouse.up();
  }
  const initial = await panel.boundingBox();
  await drag('Resize app window', -100, -80);
  const resized = await panel.boundingBox();
  assert.equal(resized.width, initial.width - 100);
  assert.equal(resized.height, initial.height - 80);
  await drag('Move app window', 65, 50);
  const moved = await panel.boundingBox();
  assert.equal(moved.x, resized.x + 65);
  assert.equal(moved.y, resized.y + 50);
  await desktopApp.control('TextInput', 'Title').fill('In a desktop window');
  await desktopApp.control('Button', 'Save').click();
  await frame.waitForFunction(() => window.eguiSandbox.snapshot().controls.some(c => c.label === 'Saved: In a desktop window'));
  await page.getByRole('button', {name: 'Close app window'}).click();
  assert.equal(await page.locator('iframe').count(), 0);
  assert.equal(await open.evaluate(element => element === document.activeElement), true);
  await open.click();
  const reopened = await (await page.locator('iframe').elementHandle()).contentFrame();
  const reopenedApp = new SandboxPage(page, {frame: reopened});
  await reopenedApp.ready();
  assert.equal((await reopenedApp.control('TextInput', 'Title').read()).value, 'First note');
  assert.deepEqual(errors, []);
  console.log('PASS: desktop opens, resizes, moves and closes the unchanged Notes app');

  // The Rust constructor combines results from an inline and a nested file-based JS module.
  const modulePage = await context.newPage();
  await modulePage.goto(cliFailure);
  await modulePage.waitForFunction(() => document.querySelector('#status')?.textContent.startsWith('Loading failed:'));
  assert.match(await modulePage.locator('#status').textContent(), /fixture constructor failed/);

  // Retrying a failed constructor must not see a stale bridge or runner.
  const failure = await modulePage.evaluate(async () => {
    try { await (await import('/app.js')).start(); }
    catch (error) { return {error: String(error), bridge: typeof window.eguiSandbox}; }
  });
  assert.match(failure.error, /fixture constructor failed/);
  assert.equal(failure.bridge, 'undefined');
  console.log('PASS: CLI serves inline and nested JS modules at their generated paths without a temp directory');

  console.log('PASS: factory-only launcher, App logic, duplicate startup, page lifecycle, constructor failure, error display and restart');
} finally {
  await context.close();
  await browser.close();
}

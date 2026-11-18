import assert from 'node:assert/strict';
import {chromium} from 'playwright';
import {SandboxPage} from '../browser/client.mjs';

const origin = process.env.SANDBOX_URL ?? 'http://127.0.0.1:8765';
const launcher = process.env.LAUNCHER_URL;
assert.ok(launcher, 'Run npm test to start the embedded app server');
const cli = process.env.CLI_URL;
assert.ok(cli, 'Run npm test to start the sandbox command');
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

  // Constructor failure must leave the page reusable, and the shell must show the error.
  const failure = await page.evaluate(async failureUrl => {
    const status = document.createElement('div');
    status.id = 'status';
    document.body.append(status);
    const {mount} = await import('/launch.mjs');
    try { await mount(() => import(failureUrl)); }
    catch (error) { return {error: String(error), status: status.textContent, bridge: typeof window.eguiSandbox}; }
  }, `${origin}/failure/failure.js`);
  assert.match(failure.error, /fixture constructor failed/);
  assert.match(failure.status, /Loading failed/);
  assert.equal(failure.bridge, 'undefined');

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

  await page.goto(cli);
  await app.ready();
  await app.control('TextInput', 'Title').fill('From the sandbox command');
  await app.control('Button', 'Save').click();
  await page.waitForFunction(() => window.eguiSandbox.snapshot().controls.some(c => c.label === 'Saved: From the sandbox command'));
  assert.deepEqual(errors, []);

  // The Rust constructor combines results from an inline and a nested file-based JS module.
  const modulePage = await context.newPage();
  await modulePage.goto(cliFailure);
  await modulePage.waitForFunction(() => document.querySelector('#status')?.textContent.startsWith('Loading failed:'));
  assert.match(await modulePage.locator('#status').textContent(), /fixture constructor failed/);
  console.log('PASS: CLI serves inline and nested JS modules at their generated paths without a temp directory');

  console.log('PASS: factory-only launcher, App logic, duplicate startup, page lifecycle, constructor failure, error display and restart');
} finally {
  await context.close();
  await browser.close();
}

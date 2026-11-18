import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {mkdtemp, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const empty = await mkdtemp(join(tmpdir(), 'egui-sandbox-test-'));
const target = resolve(root, process.env.CARGO_TARGET_DIR ?? 'target');
const servers = [];

function startServer(executable, args, options) {
  const app = spawn(executable, args, {...options, stdio: ['ignore', 'pipe', 'pipe']});
  const closed = new Promise(resolve => app.once('close', resolve));
  servers.push({app, closed});
  let diagnostics = '';
  app.stderr.on('data', data => { diagnostics += data; });
  return new Promise((resolve, reject) => {
    let output = '';
    const timeout = setTimeout(() => reject(Error(`Server failed to start: ${diagnostics}`)), 60000);
    app.once('error', error => { clearTimeout(timeout); reject(error); });
    app.once('close', code => { clearTimeout(timeout); reject(Error(`Server exited (${code}): ${diagnostics}`)); });
    app.stdout.on('data', data => {
      output += data;
      const url = output.match(/http:\/\/(?:127\.0\.0\.1|\[::1\]):\d+\//)?.[0];
      if (url) { clearTimeout(timeout); resolve(url); }
    });
  });
}

try {
  const command = join(target, 'debug/egui-web-sandbox');
  const wasm = join(target, 'wasm32-unknown-unknown/debug');
  // The command only needs its WASM input and runs without Cargo or a temporary directory.
  const noTemp = join(empty, 'no-temp-directory');
  const options = {cwd: empty, env: {...process.env, PATH: '', TMPDIR: noTemp, TEMP: noTemp, TMP: noTemp}};
  const [workbenchUrl, notesUrl, failureUrl] = await Promise.all([
    startServer(command, [join(wasm, 'examples/basic.wasm'), '0', '-b', 'localhost'], options),
    startServer(command, [join(wasm, 'egui_web_app.wasm'), '--port=0', '--bind', '::1'], options),
    startServer(command, [join(wasm, 'examples/startup_failure.wasm')], options),
  ]);
  assert.equal(new URL(workbenchUrl).hostname, '127.0.0.1');
  assert.equal(new URL(notesUrl).hostname, '[::1]');
  for (const origin of [workbenchUrl, notesUrl]) {
    for (const name of ['Cargo.toml', '../Cargo.toml', '.env', 'src/lib.rs']) {
      assert.equal((await fetch(new URL(name, origin))).status, 404);
    }
    assert.equal((await fetch(new URL('app_bg.wasm', origin), {method: 'HEAD'})).headers.get('content-type'), 'application/wasm');
    assert.equal((await fetch(origin, {method: 'POST'})).status, 405);
  }
  const env = {
    ...process.env,
    SANDBOX_URL: workbenchUrl,
    LAUNCHER_URL: notesUrl,
    CLI_FAILURE_URL: failureUrl,
  };
  for (const script of ['parallel.mjs', 'launcher.mjs']) {
    const child = spawn(process.execPath, [fileURLToPath(new URL(script, import.meta.url))], {env, stdio: 'inherit'});
    const [code] = await once(child, 'exit');
    assert.equal(code, 0, `${script} failed`);
  }
  console.log('PASS: IPv4 and IPv6 browser access, in-memory assets, and HTTP boundaries');
} finally {
  for (const {app} of servers) app.kill();
  await Promise.all(servers.map(({closed}) => closed));
  await rm(empty, {recursive: true, force: true});
}

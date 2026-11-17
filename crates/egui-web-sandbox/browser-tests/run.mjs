import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {mkdtemp, readFile, realpath, rm} from 'node:fs/promises';
import {createServer} from 'node:http';
import {tmpdir} from 'node:os';
import {join, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const directory = await realpath(join(root, '.build/site'));
const empty = await mkdtemp(join(tmpdir(), 'egui-embedded-'));
const executable = process.env.SANDBOX_SERVER ?? join(root, 'target/debug/egui-web-app');
// The finished executable must work without Cargo, Python or a source directory.
const app = spawn(executable, [], {cwd: empty, env: {...process.env, PATH: ''}, stdio: ['ignore', 'pipe', 'pipe']});
const exited = once(app, 'exit');
let diagnostics = '';
app.stderr.on('data', data => { diagnostics += data; });
// Only test fixtures use a disk server. The product's demo is served by its own executable.
const fixtures = createServer(async (request, response) => {
  response.setHeader('Access-Control-Allow-Origin', '*');
  try {
    let path = resolve(directory, `.${new URL(request.url, 'http://localhost').pathname}`);
    if (path.endsWith(`${sep}sandbox`)) path = join(path, 'index.html');
    path = await realpath(path);
    if (!path.startsWith(directory + sep)) throw Error('outside fixture directory');
    const bytes = await readFile(path);
    response.setHeader('Content-Type', path.endsWith('.wasm') ? 'application/wasm' : path.endsWith('.html') ? 'text/html' : 'text/javascript');
    response.end(bytes);
  } catch {
    response.writeHead(404).end();
  }
});

try {
  const url = await new Promise((resolve, reject) => {
    let output = '';
    const timeout = setTimeout(() => reject(Error(`Embedded server failed to start: ${diagnostics}`)), 10000);
    app.once('error', reject);
    app.once('exit', code => { clearTimeout(timeout); reject(Error(`Embedded server exited (${code}): ${diagnostics}`)); });
    app.stdout.on('data', data => {
      output += data;
      const url = output.match(/http:\/\/127\.0\.0\.1:\d+\//)?.[0];
      if (url) { clearTimeout(timeout); resolve(url); }
    });
  });
  for (const name of ['Cargo.toml', '../Cargo.toml', '.env', 'src/main.rs']) {
    assert.equal((await fetch(new URL(name, url))).status, 404);
  }
  assert.equal((await fetch(new URL('app_bg.wasm', url), {method: 'HEAD'})).headers.get('content-type'), 'application/wasm');
  assert.equal((await fetch(url, {method: 'POST'})).status, 405);
  fixtures.listen(0, '127.0.0.1');
  await once(fixtures, 'listening');
  const env = {...process.env, SANDBOX_URL: `http://127.0.0.1:${fixtures.address().port}`, LAUNCHER_URL: url};
  for (const script of ['parallel.mjs', 'launcher.mjs']) {
    const child = spawn(process.execPath, [fileURLToPath(new URL(script, import.meta.url))], {env, stdio: 'inherit'});
    const [code] = await once(child, 'exit');
    assert.equal(code, 0, `${script} failed`);
  }
  console.log('PASS: self-contained native server, empty PATH/cwd, embedded assets and HTTP boundaries');
} finally {
  fixtures.closeAllConnections();
  fixtures.close();
  app.kill();
  await exited;
  await rm(empty, {recursive: true, force: true});
}

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
const target = process.env.CARGO_TARGET_DIR ?? join(root, 'target');
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

// Only test fixtures use a disk server.
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
  // The embedded executable must work without Cargo, Python or a source directory.
  const embedded = process.env.SANDBOX_SERVER ?? join(target, 'debug/egui-embedded-web-app');
  const command = join(target, 'debug/egui-web-sandbox');
  const [url, cliUrl] = await Promise.all([
    startServer(embedded, ['0', '-b', 'localhost'], {cwd: empty, env: {...process.env, PATH: ''}}),
    startServer(command, ['examples/web-app/Cargo.toml', '--port=0', '--bind', '::1'], {cwd: root}),
  ]);
  assert.equal(new URL(url).hostname, '127.0.0.1');
  assert.equal(new URL(cliUrl).hostname, '[::1]');
  for (const origin of [url, cliUrl]) {
    for (const name of ['Cargo.toml', '../Cargo.toml', '.env', 'src/lib.rs']) {
      assert.equal((await fetch(new URL(name, origin))).status, 404);
    }
    assert.equal((await fetch(new URL('app_bg.wasm', origin), {method: 'HEAD'})).headers.get('content-type'), 'application/wasm');
    assert.equal((await fetch(origin, {method: 'POST'})).status, 405);
  }
  fixtures.listen(0, '127.0.0.1');
  await once(fixtures, 'listening');
  const env = {...process.env, SANDBOX_URL: `http://127.0.0.1:${fixtures.address().port}`, LAUNCHER_URL: url, CLI_URL: cliUrl};
  for (const script of ['parallel.mjs', 'launcher.mjs']) {
    const child = spawn(process.execPath, [fileURLToPath(new URL(script, import.meta.url))], {env, stdio: 'inherit'});
    const [code] = await once(child, 'exit');
    assert.equal(code, 0, `${script} failed`);
  }
  console.log('PASS: IPv4 and IPv6 browser access, empty PATH/cwd, embedded assets and HTTP boundaries');
} finally {
  fixtures.closeAllConnections();
  fixtures.close();
  for (const {app} of servers) app.kill();
  await Promise.all(servers.map(({closed}) => closed));
  await rm(empty, {recursive: true, force: true});
}

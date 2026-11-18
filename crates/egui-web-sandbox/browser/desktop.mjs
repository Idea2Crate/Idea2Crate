const workspace = document.querySelector('main');
const panel = document.querySelector('section');
const open = document.querySelector('#open');
const move = document.querySelector('#move');
const resize = document.querySelector('#resize');
const close = document.querySelector('#close');
const minimum = [300, 220];
let geometry;

function place({x, y, width, height}) {
  const available = [workspace.clientWidth, workspace.clientHeight];
  width = Math.min(available[0], Math.max(minimum[0], width));
  height = Math.min(available[1], Math.max(minimum[1], height));
  x = Math.max(0, Math.min(x, available[0] - width));
  y = Math.max(0, Math.min(y, available[1] - height));
  geometry = {x, y, width, height};
  Object.assign(panel.style, {left: `${x}px`, top: `${y}px`, width: `${width}px`, height: `${height}px`});
}

open.addEventListener('click', () => {
  if (!panel.hidden) {
    move.focus();
    return;
  }
  const inset = 16;
  place({x: inset, y: inset, width: workspace.clientWidth - 2 * inset, height: workspace.clientHeight - 2 * inset});
  const frame = document.createElement('iframe');
  frame.title = 'Application';
  // The app owns its normal document, eframe lifecycle and browser input.
  frame.src = './index.html' + location.search;
  resize.before(frame);
  panel.hidden = false;
  move.focus();
});

close.addEventListener('click', () => {
  panel.querySelector('iframe')?.remove();
  panel.hidden = true;
  open.focus();
});

new ResizeObserver(() => {
  if (!panel.hidden) place(geometry);
}).observe(workspace);

function installHandle(handle, fields) {
  const step = 12;
  let gesture;
  function apply(origin, dx, dy) {
    place({...origin, [fields[0]]: origin[fields[0]] + dx, [fields[1]]: origin[fields[1]] + dy});
  }
  handle.addEventListener('pointerdown', event => {
    if (event.button !== 0 || gesture) return;
    gesture = {id: event.pointerId, x: event.clientX, y: event.clientY, origin: geometry};
    handle.setPointerCapture(event.pointerId);
    panel.dataset.moving = '';
  });
  handle.addEventListener('pointermove', event => {
    if (gesture?.id !== event.pointerId) return;
    apply(gesture.origin, event.clientX - gesture.x, event.clientY - gesture.y);
  });
  handle.addEventListener('lostpointercapture', () => {
    gesture = undefined;
    delete panel.dataset.moving;
  });
  handle.addEventListener('keydown', event => {
    const delta = {ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step]}[event.key];
    if (!delta) return;
    event.preventDefault();
    apply(geometry, ...delta);
  });
}

installHandle(move, ['x', 'y']);
installHandle(resize, ['width', 'height']);

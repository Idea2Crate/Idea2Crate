/** Mount a wasm-bindgen module exporting start(), and return an idempotent stop function. */
export async function mount(load, {args = [], status = document.querySelector('#status')} = {}) {
  let handle;
  try {
    const module = await load();
    await module.default();
    handle = await module.start(...args);
    const onPageHide = event => {
      if (!event.persisted) stop();
    };
    const stop = () => {
      window.removeEventListener('pagehide', onPageHide);
      const app = handle;
      handle = null;
      app?.free();
    };
    window.addEventListener('pagehide', onPageHide);
    status?.remove();
    return stop;
  } catch (error) {
    handle?.free();
    if (status) status.textContent = `Loading failed: ${error}`;
    throw error;
  }
}

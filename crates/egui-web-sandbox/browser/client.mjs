// A small Playwright-compatible client. Reads observations; actions use real browser input.
export class SandboxPage {
  constructor(page, {canvas = '#canvas', frame = page} = {}) {
    this.page = page;
    this.frame = frame;
    this.canvas = canvas;
  }

  async ready() {
    await this.frame.waitForFunction(() => window.eguiSandbox?.snapshot()?.ready === true);
    const state = await this.snapshot();
    if (state.schema_version !== 1) throw new Error(`Unsupported sandbox protocol: ${state.schema_version}`);
  }

  snapshot() {
    return this.frame.evaluate(() => window.eguiSandbox.snapshot());
  }

  control(role, name, {window: windowName} = {}) {
    const read = async () => {
      const state = await this.snapshot();
      let windowId;
      if (windowName !== undefined) {
        const windows = state.windows.filter(window => window.title === windowName);
        if (windows.length !== 1) throw new Error(`Expected one window named ${windowName}; found ${windows.length}`);
        windowId = windows[0].id;
      }
      const matches = state.controls.filter(control => control.role === role && control.label === name
        && (windowId === undefined || control.window === windowId));
      if (matches.length !== 1) throw new Error(`Expected one ${role} named ${name}; found ${matches.length}. Scope repeated names to a window.`);
      return {state, control: matches[0]};
    };
    const point = async () => {
      const {state, control} = await read();
      if (control.disabled) throw new Error(`${name} is disabled`);
      const [left, top, right, bottom] = control.bounds;
      const [x0, y0, x1, y1] = state.bounds;
      const clipped = [Math.max(left, x0), Math.max(top, y0), Math.min(right, x1), Math.min(bottom, y1)];
      if (clipped[0] >= clipped[2] || clipped[1] >= clipped[3]) throw new Error(`${name} is outside the viewport`);
      const canvas = await this.frame.locator(this.canvas).boundingBox();
      if (!canvas) throw new Error('Canvas is not visible');
      return [canvas.x + (clipped[0] + clipped[2]) / 2 * state.zoom_factor,
        canvas.y + (clipped[1] + clipped[3]) / 2 * state.zoom_factor];
    };
    const click = async () => {
      await this.page.mouse.click(...await point());
      await this.settle();
    };
    return {
      read: async () => (await read()).control,
      click,
      fill: async text => {
        await click();
        await this.page.keyboard.press('ControlOrMeta+a');
        await this.page.keyboard.press('Backspace');
        if (text) await this.page.keyboard.insertText(text);
        await this.settle();
      },
      dragBy: async (x, y) => {
        const from = await point();
        await this.page.mouse.move(...from);
        await this.page.mouse.down();
        await this.page.mouse.move(from[0] + x, from[1] + y, {steps: 12});
        await this.page.mouse.up();
        await this.settle();
      },
    };
  }

  async reset() {
    if (!(await this.snapshot()).reset_supported) throw new Error('Application reset is not enabled');
    const generation = await this.frame.evaluate(() => window.eguiSandbox.reset());
    await this.frame.waitForFunction(generation => {
      const state = window.eguiSandbox.snapshot();
      return state.ready && state.generation === generation;
    }, generation);
    await this.settle();
  }

  async settle() {
    await this.frame.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  }
}

// Automated DOM smoke test for the shipped adele-web page.
// Run by loading tests/dom_smoke.html over a static server (wasm-pack build
// must have been run first). The page prints the JSON result into #result.

export async function runDomSmoke() {
  const failures = [];
  const errors = [];
  window.addEventListener('error', e => errors.push(String(e.message)));
  window.addEventListener('unhandledrejection', e => errors.push(String(e.reason)));

  const check = (name, condition, detail) => {
    if (!condition) failures.push({ name, detail });
  };

  const wasm = await import('../pkg/adele_web_demo.js');
  await wasm.default();
  const canvas = document.querySelector('#cvkg-canvas');
  const live = () => document.querySelector('[aria-live]')?.textContent ?? '';
  const state = () => JSON.parse(wasm.input_state_json_js());

  check('canvas present', !!canvas, null);
  check('canvas focusable and named', canvas?.tabIndex === 0 && !!canvas?.getAttribute('aria-label'), null);

  const mirrored = [...document.querySelectorAll('[data-cvkg-node-id]')].map(e => e.getAttribute('role'));
  check('a11y bridge mirrors heading+textbox', mirrored.includes('heading') && mirrored.includes('textbox'), mirrored);

  const rect = canvas.getBoundingClientRect();
  const click = (x, y) => {
    canvas.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, clientX: rect.left + x, clientY: rect.top + y }));
    return { live: live(), region: state().focused_region };
  };
  const clickCases = [
    ['sidebar', 100, 100, 'Focused Sidebar', 0],
    ['search', 400, 100, 'Focused Search field', 1],
    ['detail', 400, 250, 'Focused Detail panel', 2],
  ];
  for (const [name, x, y, expectedLive, expectedRegion] of clickCases) {
    const got = click(x, y);
    check(`click ${name}`, got.live === expectedLive && got.region === expectedRegion, got);
  }

  const key = (k, opts = {}) => {
    canvas.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, key: k, ...opts }));
    return state();
  };
  check('Ctrl+K focuses search', key('k', { ctrlKey: true }).last_action === 'focus_search', state());
  check('typing inserts text', key('a').text === 'a', state());
  check('Backspace deletes', key('Backspace').text === '', state());
  check('Tab advances region', key('Tab').focused_region === 2, state());
  check('Escape clears text', key('Escape').last_action === 'clear_text', state());

  const proxy = document.querySelector('textarea[aria-label="Canvas text input proxy"]');
  check('DOM text proxy present', !!proxy, null);
  proxy.value = 'hello';
  proxy.dispatchEvent(new Event('input', { bubbles: true }));
  check('DOM text proxy updates live region', live() === 'Text changed: hello', live());
  check('DOM text proxy updates canvas state', state().text === 'hello' && state().caret === 5, state());

  check('no JS errors', errors.length === 0, errors);

  return {
    pass: failures.length === 0,
    checks: 15,
    failures,
  };
}

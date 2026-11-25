// Browser smoke runner used by CI. Start a static server at :8765 first.
const [, , browserName = 'chromium', deviceName] = process.argv;

const { chromium, webkit, devices } = await import('playwright');
const browsers = { chromium, webkit };
const engine = browsers[browserName];
if (!engine) {
  throw new Error(`unknown browser: ${browserName}`);
}

const browser = await engine.launch();
const device = deviceName
  ? { ...devices[deviceName], viewport: { width: 900, height: 700 } }
  : {};
const context = await browser.newContext(device);
const page = await context.newPage();
const errors = [];
page.on('pageerror', (error) => errors.push(String(error)));

await page.goto('http://127.0.0.1:8765/tests/dom_smoke.html');
await page.waitForFunction(
  () => {
    const text = document.querySelector('#result')?.textContent;
    return Boolean(text && text !== 'running...');
  },
  { timeout: 30000 }
);

const result = JSON.parse(await page.locator('#result').textContent());
await browser.close();

if (!result.pass || errors.length > 0) {
  console.error(JSON.stringify({ result, errors }, null, 2));
  process.exit(1);
}
console.log(`${browserName}${deviceName ? `/${deviceName}` : ''}: pass (${result.checks} checks)`);

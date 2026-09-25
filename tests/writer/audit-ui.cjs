const { createRequire } = require('node:module');
const { readFileSync, mkdirSync, writeFileSync } = require('node:fs');
const { resolve, join } = require('node:path');
const { chromium } = createRequire(resolve('apps/dashboard/package.json'))('@playwright/test');
const output = process.env.VESTIGE_TEST_OUTPUT || '/tmp/vestige-writer-ui-audit';
const base = process.env.VESTIGE_TEST_UI_URL || 'http://127.0.0.1:3931';
const source = readFileSync('apps/dashboard/src/lib/os-routes.ts', 'utf8');
const routes = [...new Set([...source.matchAll(/href:\s*'([^']+)'/g)].map(match => match[1]))];
mkdirSync(output, { recursive: true });
(async () => {
    const browser = await chromium.launch({ headless: true, ...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE } : {}), args: ['--enable-unsafe-webgpu', '--use-angle=metal'] });
    const results = [];
    for (const route of routes) {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        const errors = [], resources = [];
        page.on('pageerror', error => errors.push(String(error)));
        page.on('response', response => { if (response.status() >= 400 && response.url().startsWith(base)) resources.push({ status: response.status(), url: response.url() }); });
        await page.goto(base + '/dashboard' + route, { waitUntil: 'networkidle' });
        await page.locator('body').waitFor();
        const text = await page.locator('body').innerText();
        const english = [...new Set(text.split('\n').map(line => line.trim()).filter(line => /[A-Za-z]{3}/.test(line) && !/[\u3400-\u9fff]/.test(line)))];
        const lang = await page.locator('html').getAttribute('lang');
        if (['/writer', '/palace', '/observatory', '/memories', '/settings'].includes(route)) await page.screenshot({ path: join(output, `page-${route.slice(1)}.png`), fullPage: true });
        results.push({ route, title: await page.title(), lang, errors, resources, untranslatedCandidates: english });
        console.log(JSON.stringify(results[results.length - 1]));
        await page.close();
    }
    await browser.close();
    writeFileSync(join(output, 'route-audit.json'), JSON.stringify(results, null, 2));
    if (results.some(row => row.errors.length || row.resources.length || row.lang !== 'zh-CN')) process.exitCode = 1;
})().catch(error => { console.error(error); process.exitCode = 1; });

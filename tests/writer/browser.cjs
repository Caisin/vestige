// Real browser workflow paired with an external MCP client. Use an isolated
// writer test service; all created content is explicitly synthetic.
const { createRequire } = require('node:module');
const { readFileSync, mkdirSync, writeFileSync } = require('node:fs');
const { resolve, join } = require('node:path');
const requireDashboard = createRequire(resolve('apps/dashboard/package.json'));
const { chromium, expect } = requireDashboard('@playwright/test');
const ui = process.env.VESTIGE_TEST_UI_URL || 'http://127.0.0.1:3931';
const mcp = process.env.VESTIGE_TEST_MCP_URL || 'http://127.0.0.1:3932/mcp';
const token = readFileSync(process.env.VESTIGE_TEST_TOKEN_FILE, 'utf8').trim();
const output = process.env.VESTIGE_TEST_OUTPUT || '/tmp/vestige-writer-browser';
mkdirSync(output, { recursive: true });
let sequence = 0;
const headers = { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream' };
async function rpc(method, params, notification = false) {
    const response = await fetch(mcp, { method: 'POST', headers, body: JSON.stringify({ jsonrpc: '2.0', ...(notification ? {} : { id: ++sequence }), method, params }) });
    if (!response.ok) throw new Error(`MCP HTTP ${response.status}: ${await response.text()}`);
    if (response.headers.get('mcp-session-id')) headers['Mcp-Session-Id'] = response.headers.get('mcp-session-id');
    const body = await response.text(); if (!body) return null;
    const value = JSON.parse(body); if (value.error) throw new Error(JSON.stringify(value.error)); return value.result;
}
async function call(name, action, args = {}) {
    const result = await rpc('tools/call', { name, arguments: { action, ...args } });
    const value = result.structuredContent || JSON.parse(result.content[0].text);
    if (result.isError) throw new Error(JSON.stringify(value)); return value;
}
async function pending(role, kind) {
    const rows = (await call('writer_task', 'list', { role_id: role, status: 'queued' })).tasks;
    const task = rows.find(t => t.kind === kind); if (!task) throw new Error(`No pending ${kind} task`);
    const claimed = await call('writer_task', 'claim', { task_id: task.id, agent_id: 'browser-acceptance-agent' });
    return claimed;
}

(async () => {
    const init = await rpc('initialize', { protocolVersion: '2025-03-26', capabilities: {}, clientInfo: { name: 'writer-browser-acceptance', version: '1' } });
    headers['MCP-Protocol-Version'] = init.protocolVersion; await rpc('notifications/initialized', {}, true);
    const browser = await chromium.launch({ headless: true, ...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE } : {}), args: ['--enable-unsafe-webgpu', '--use-angle=metal'] });
    const page = await browser.newPage({ viewport: { width: 1560, height: 1100 } });
    const errors = []; page.on('pageerror', error => errors.push(String(error)));
    let roleId, projectId;
    try {
        await page.goto(`${ui}/dashboard/writer`, { waitUntil: 'networkidle' });
        if (!(await page.getByLabel('角色名称').isVisible())) await page.getByRole('button', { name: '新建编剧角色', exact: true }).click();
        const name = `悬疑编剧·界面验收 ${Date.now()}`;
        await page.getByLabel('角色名称', { exact: true }).fill(name);
        await page.getByLabel('创作定位', { exact: true }).fill('短剧与网剧优先。合成作品仅用于验证流程，不代表真实作者分析。');
        await page.getByRole('button', { name: '创建角色', exact: true }).click();
        await expect(page.getByRole('heading', { name, exact: true })).toBeVisible();
        roleId = (await call('writer_role', 'list')).roles.find(r => r.name === name).id;
        const text = '第一集 第一场 内景 旧车站 夜\n沈禾：如果我把录音交出去，姐姐就回不了家。\n她把录音笔放在警察面前。\n第二场 外景 街口 夜\n她发现录音里还有自己的声音。';
        await page.locator('input[type="file"]').setInputFiles({ name: '车站秘密（合成验收剧本）.txt', mimeType: 'text/plain', buffer: Buffer.from(text) });
        await expect(page.getByRole('heading', { name: '车站秘密（合成验收剧本）', exact: true })).toBeVisible();
        await page.getByRole('button', { name: '阅读', exact: true }).click();
        await expect(page.getByRole('dialog')).toBeVisible();
        await expect(page.getByRole('dialog').locator('pre')).toContainText('姐姐就回不了家');
        await page.keyboard.press('Escape'); await expect(page.getByRole('dialog')).not.toBeVisible();
        await page.getByRole('button', { name: '提炼编剧方法', exact: true }).click();
        await expect(page.getByText('等待 Agent', { exact: true })).toBeVisible();
        const extraction = await pending(roleId, 'extract');
        const source = (await call('writer_source', 'list', { role_id: roleId })).sources[0];
        const segment = (await call('writer_source', 'get', { role_id: roleId, source_id: source.id })).segments[0];
        const rules = [{ id: 'costly-choice', category: 'conflict', title: '让人物为选择付出代价', instruction: '在关系高潮，让人物在亲情和公共责任之间作出有后果的选择。', rationale: '动作把价值冲突变成可见的决定。', applies_to: '关键关系转折', exceptions: '不强制用于交代信息的过渡场景', evidence: [{ source_id: source.id, segment_id: segment.id, quote: '如果我把录音交出去，姐姐就回不了家。' }] }];
        await call('writer_task', 'complete', { task_id: extraction.task.id, lease_token: extraction.lease_token, result: { summary: '从具体抉择中提炼价值冲突方法。', rules } });
        await page.getByRole('button', { name: /^创作规则/ }).click();
        await expect(page.getByLabel('选择角色规则版本').locator('option[value="1"]')).toBeAttached({ timeout: 15000 });
        await page.getByLabel('选择角色规则版本').selectOption('1');
        await expect(page.getByText('让人物为选择付出代价', { exact: true })).toBeVisible();
        await page.screenshot({ path: join(output, 'rules-evidence.png'), fullPage: true });
        await page.getByRole('button', { name: '采纳此版本', exact: true }).click();
        await expect(page.getByText('生效版本 v1', { exact: true })).toBeVisible();
        await page.getByRole('button', { name: '对话调校', exact: true }).click();
        await page.getByLabel('给编剧角色的调整意见').fill('减少旁白，让犹豫通过动作表现。');
        await page.getByRole('button', { name: '发送调整', exact: true }).click();
        await expect(page.getByText('减少旁白，让犹豫通过动作表现。', { exact: true })).toBeVisible();
        const chat = await pending(roleId, 'role_chat');
        rules.push({ id: 'action-first', category: 'preference', title: '用动作呈现犹豫', instruction: '优先以动作、停顿和对话表现心理，减少解释性旁白。', evidence: [{ message_id: chat.task.input.message_id, quote: '减少旁白，让犹豫通过动作表现。' }] });
        await call('writer_task', 'complete', { task_id: chat.task.id, lease_token: chat.lease_token, result: { reply: '我保留了有代价的选择，并加入动作优先的偏好。你可以对比候选版本再采纳。', summary: '新增用户偏好：减少解释性旁白。', rules } });
        await expect(page.getByText('我保留了有代价的选择，并加入动作优先的偏好。你可以对比候选版本再采纳。', { exact: true })).toBeVisible({ timeout: 15000 });
        await page.screenshot({ path: join(output, 'conversation.png'), fullPage: true });
        await page.getByRole('button', { name: /^创作规则/ }).click(); await page.getByLabel('选择角色规则版本').selectOption('2'); await page.getByRole('button', { name: '采纳此版本', exact: true }).click();
        await expect(page.getByText('生效版本 v2', { exact: true })).toBeVisible();
        await page.getByRole('button', { name: '创作项目', exact: true }).click();
        await page.getByLabel('项目名称', { exact: true }).fill('寄给明天的信（原创验收）');
        await page.getByLabel('故事简报', { exact: true }).fill('失物招领员发现一封写给明天的信，在末班车离站前做出选择。');
        await page.getByLabel('人物设定', { exact: true }).fill('程雨：谨慎，不愿再次错过父亲留下的线索。');
        await page.getByRole('button', { name: /^创建项目 · 固定角色/ }).click();
        await expect(page.getByRole('button', { name: '交给 Agent 创作', exact: true })).toBeVisible();
        projectId = (await call('writer_project', 'list', { role_id: roleId })).projects[0].id;
        await page.getByRole('button', { name: '交给 Agent 创作', exact: true }).click();
        const writing = await pending(roleId, 'write');
        const context = await call('writer_context', 'prepare', { role_id: roleId, project_id: projectId, query: '开场戏' });
        if (context.role_version !== 2 || context.rules.length !== 2) throw new Error('Writer context is not pinned to the adopted version');
        const script = '内景 失物招领处 夜\n程雨把信推到桌沿，指尖却压住了信封上的日期。\n同事：末班车要走了。\n程雨松开手。信封滑落，露出父亲的笔迹。';
        await call('writer_task', 'complete', { task_id: writing.task.id, lease_token: writing.lease_token, result: { kind: 'scene', title: '失物招领处的抉择', content: script } });
        await expect(page.getByRole('button', { name: /失物招领处的抉择/ })).toBeVisible({ timeout: 15000 }); await page.getByRole('button', { name: /失物招领处的抉择/ }).click();
        await expect(page.getByLabel('剧本正文')).toHaveValue(script);
        await page.getByRole('button', { name: '请求 Agent 审稿', exact: true }).click();
        const review = await pending(roleId, 'review');
        await call('writer_task', 'complete', { task_id: review.task.id, lease_token: review.lease_token, result: { summary: '动作表达清晰，可补充错过末班车的具体代价。', findings: [{ severity: 'warning', quote: '末班车要走了。', issue: '留下与离开的代价尚未具体化。', suggestion: '增加程雨必须赶上的约定，让留下成为真正的选择。' }] } });
        await expect(page.getByText('动作表达清晰，可补充错过末班车的具体代价。', { exact: true })).toBeVisible({ timeout: 15000 });
        await page.screenshot({ path: join(output, 'draft-review.png'), fullPage: true });
        await page.reload({ waitUntil: 'networkidle' }); await expect(page.getByRole('heading', { name, exact: true })).toBeVisible();
        await page.setViewportSize({ width: 390, height: 844 }); await page.screenshot({ path: join(output, 'mobile-workbench.png'), fullPage: true });
        if (errors.length) throw new Error(errors.join('\n'));
        const report = { browser: 'Chromium', workflow: 'UI upload → MCP extraction → publish → UI feedback → MCP adjustment → publish → project → MCP draft → review', passed: true, javascriptErrors: errors, screenshots: ['rules-evidence.png', 'conversation.png', 'draft-review.png', 'mobile-workbench.png'] };
        writeFileSync(join(output, 'browser-acceptance.json'), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report));
    } finally {
        if (projectId) await call('writer_project', 'delete', { project_id: projectId, confirm: true }).catch(() => {});
        if (roleId) await call('writer_role', 'delete', { role_id: roleId, confirm: true }).catch(() => {});
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });

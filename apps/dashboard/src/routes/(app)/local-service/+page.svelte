<script lang="ts">
    import { browser } from '$app/environment';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    type LocalMcpStatus = {
        running: boolean; ownedByApp: boolean; pid: number | null; dashboardUrl: string; mcpUrl: string;
        dataDir: string; configFile: string; serviceLog: string; health: Record<string, unknown> | null; config: Record<string, string>;
    };
    let status = $state<LocalMcpStatus | null>(null), loading = $state(true), busy = $state(''), error = $state(''), isDesktop = $state(false);
    let timer: ReturnType<typeof setInterval> | undefined;
    async function readStatus() {
        if (!isDesktop) { loading = false; return; }
        try { status = await invoke<LocalMcpStatus>('local_mcp_status_command'); error = ''; }
        catch (value) { error = value instanceof Error ? value.message : String(value); }
        finally { loading = false; }
    }
    async function run(action: 'start' | 'stop') {
        busy = action; error = '';
        try { status = await invoke<LocalMcpStatus>(action === 'start' ? 'local_mcp_start_command' : 'local_mcp_stop_command'); }
        catch (value) { error = value instanceof Error ? value.message : String(value); }
        finally { busy = ''; await readStatus(); }
    }
    onMount(() => { isDesktop = browser && '__TAURI_INTERNALS__' in window; void readStatus(); if (isDesktop) timer = setInterval(() => void readStatus(), 5000); return () => { if (timer) clearInterval(timer); }; });
    const labels: Record<string, string> = { VESTIGE_DATA_DIR: '数据目录', VESTIGE_DASHBOARD_PORT: '工作台端口', VESTIGE_HTTP_PORT: 'MCP 端口', VESTIGE_HTTP_BIND: '监听地址', VESTIGE_DASHBOARD_ENABLED: '工作台服务', VESTIGE_HTTP_ENABLED: 'MCP HTTP 服务', VESTIGE_AUTH_CONFIG: '身份配置路径', AUTH_PUBLIC_ORIGIN: '公开地址', AUTH_KX_API: 'KX 后台地址', AUTH_KX_CONFIGURED: 'KX 登录', AUTH_OAUTH_CONFIGURED: 'OAuth 登录', VESTIGE_QWEN_DEVICE: '嵌入设备', VESTIGE_RERANKER_MODEL: '重排模型' };
</script>

<svelte:head><title>本地 MCP 服务 · Vestige</title></svelte:head>
<main class="service-page">
    <header><div><span class="eyebrow">VESTIGE · 本地运行时</span><h1>本地 MCP 服务</h1><p>查看 Tauri 当前连接的服务、MCP 地址和运行配置。所有数据仍保存在本机。</p></div><button class="quiet" onclick={() => readStatus()} disabled={loading || !!busy}>刷新状态</button></header>
    {#if !isDesktop}<section class="notice"><strong>此页面在 Tauri 桌面应用中提供服务控制。</strong><p>当前是浏览器窗口。你可以访问工作台，但启动、停止和进程状态由本地桌面应用管理。</p></section>{/if}
    {#if error}<div class="error" role="alert">{error}</div>{/if}
    {#if loading}<section class="panel loading">正在读取本地服务状态…</section>
    {:else if status}
        <section class="hero-panel"><div class="status-line"><span class:online={status.running} class="status-dot"></span><div><strong>{status.running ? '服务运行中' : '服务未运行'}</strong><small>{status.ownedByApp ? `Tauri sidecar · PID ${status.pid ?? '—'}` : '外部本地服务 · 由其他进程管理'}</small></div></div><div class="actions"><button class="primary" onclick={() => run('start')} disabled={!!busy || status.running}>{busy === 'start' ? '正在启动…' : status.running ? '已运行' : '启动本地 MCP'}</button><button onclick={() => run('stop')} disabled={!!busy || !status.ownedByApp}>{busy === 'stop' ? '正在停止…' : '停止 Tauri 服务'}</button></div></section>
        <div class="grid"><section class="panel"><h2>连接地址</h2><div class="endpoint"><span>编剧工作台</span><a href={status.dashboardUrl + '/dashboard/writer'} target="_blank" rel="noreferrer">{status.dashboardUrl}/dashboard/writer ↗</a></div><div class="endpoint"><span>MCP Streamable HTTP</span><code>{status.mcpUrl}</code></div><p class="hint">本地客户端登录后，Agent 直接连接此 loopback 地址，不需要配置 Bearer 密钥。远程网络无法访问；独立机器才需要显式凭据。 </p></section><section class="panel"><h2>进程与日志</h2><div class="kv"><span>数据目录</span><code>{status.dataDir}</code></div><div class="kv"><span>服务日志</span><code>{status.serviceLog}</code></div><div class="kv"><span>身份配置</span><code>{status.configFile}</code></div><p class="hint">配置路径只做展示；密钥、密码和上游 token 不会显示在此页面。</p></section></div>
        <section class="panel"><div class="section-title"><div><h2>运行配置</h2><p>以下是服务实际读取的非敏感配置。</p></div><span class="badge">只读展示</span></div><div class="config-grid">{#each Object.entries(status.config) as [key, value]}<div class="config-row"><span>{labels[key] ?? key}</span><code>{value}</code></div>{/each}</div></section>
    {:else}<section class="panel"><h2>无法读取服务状态</h2><p>请启动 Tauri 应用，或检查本地服务日志。</p></section>{/if}
</main>
<style>
    .service-page{height:100%;overflow:auto;padding:42px 48px 70px max(32px,var(--os-content-left,0px));background:radial-gradient(ellipse at 85% 0,#1e3c2b55,transparent 55%),#09110d;color:#dce9d8;font-family:system-ui,"PingFang SC",sans-serif}.service-page header{display:flex;align-items:start;justify-content:space-between;gap:24px;max-width:1160px;margin:0 auto 30px}.eyebrow{font-size:10px;letter-spacing:.2em;color:#a4bc99}.service-page h1{font-size:32px;margin:12px 0 8px}.service-page h2{font-size:18px;margin:0 0 10px}.service-page p{font-size:12px;line-height:1.8;color:#94aa98}.panel,.hero-panel{max-width:1160px;margin:0 auto 20px;border:1px solid #34513b;border-radius:13px;background:#132119cc;padding:25px}.hero-panel{display:flex;align-items:center;justify-content:space-between;gap:24px}.status-line{display:flex;align-items:center;gap:13px}.status-line strong{display:block;font-size:18px}.status-line small{display:block;margin-top:6px;color:#8ea592}.status-dot{width:12px;height:12px;border-radius:50%;background:#c56c62;box-shadow:0 0 14px #c56c6288}.status-dot.online{background:#73dfa2;box-shadow:0 0 16px #73dfa299}.actions{display:flex;gap:10px;flex-wrap:wrap}.actions button,.quiet{white-space:nowrap;font:inherit;font-size:12px;padding:10px 14px;border:1px solid #466748;border-radius:7px;background:#182b1e;color:#d5e2cf;cursor:pointer}.actions button:disabled,.quiet:disabled{opacity:.45;cursor:not-allowed}.actions .primary{background:#bbc989;color:#172417;border-color:#bbc989}.grid{max-width:1160px;margin:auto;display:grid;grid-template-columns:1fr 1fr;gap:20px}.endpoint,.kv,.config-row{display:grid;grid-template-columns:150px minmax(0,1fr);gap:15px;padding:13px 0;border-bottom:1px solid #2b4432;font-size:12px}.endpoint a{color:#b8d68e;overflow-wrap:anywhere}.endpoint code,.kv code,.config-row code{font-family:ui-monospace,monospace;font-size:11px;color:#b6cdb8;overflow-wrap:anywhere}.hint{margin-bottom:0;font-size:11px!important}.section-title{display:flex;justify-content:space-between;gap:20px;align-items:start}.section-title p{margin-top:4px}.badge{font-size:10px;color:#b9d58c;border:1px solid #617b49;border-radius:20px;padding:6px 10px}.config-grid{display:grid;grid-template-columns:1fr 1fr;column-gap:32px}.notice,.error{max-width:1160px;margin:0 auto 20px;padding:16px 20px;border-radius:9px;font-size:12px}.notice{background:#244d3633;border:1px solid #466b4e;color:#c8dbbd}.notice p{margin-bottom:0}.error{background:#703c3533;border:1px solid #8e554c;color:#f0bbb0}.loading{text-align:center;color:#9bb09d}@media(max-width:900px){.service-page{padding:25px 18px 90px}.service-page header,.hero-panel{display:block}.actions{margin-top:20px}.grid{grid-template-columns:1fr}.config-grid{grid-template-columns:1fr}}@media(max-width:600px){.endpoint,.kv,.config-row{grid-template-columns:1fr;gap:5px}.service-page h1{font-size:26px}}
</style>

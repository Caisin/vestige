<script lang="ts">
    import {onMount} from 'svelte';
    import {base} from '$app/paths';
    let username=$state(''), password=$state(''), error=$state(''), busy=$state(false), loading=$state(true);
    let apps=$state<{app_key:string;app_name:string;is_default:boolean}[]>([]), selected=$state(''), oauth=$state('');
    onMount(()=>{void (async()=>{
        try { const r=await fetch('/api/auth/config');if(!r.ok)throw new Error('登录服务暂时不可用');const c=await r.json();
            if(!c.enabled){location.replace(`${base}/writer`);return;}
            apps=c.dingtalk_apps||[];selected=apps.find(a=>a.is_default)?.app_key||apps[0]?.app_key||'';oauth=c.oauth_name||'';
            if(new URL(location.href).searchParams.has('error'))error='登录请求已过期或未获授权，请重新登录。';
            else if(c.provider_error)error='暂时无法读取钉钉应用，请稍后重试；账号登录仍可使用。';
        }catch(e){error=String(e);}finally{loading=false;}
    })();});
    async function login(e:SubmitEvent){e.preventDefault();busy=true;error='';try{
        const r=await fetch('/api/auth/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username,password})});
        const v=await r.json();if(!r.ok)throw new Error(v.error||'登录失败');password='';location.replace(`${base}/writer`);
    }catch(e){error=e instanceof Error?e.message:String(e);}finally{busy=false;}}
</script>
<svelte:head><title>登录 · Vestige 编剧工作台</title></svelte:head>
<main class="login-page">
    <section class="intro"><span class="eyebrow">VESTIGE · 创作记忆</span><h1>每一种风格，<br/>都有自己的来处。</h1><p>用作品积累方法，用对话磨炼角色。<br/>登录后，进入你的私人空间，或与团队共同培养编剧。</p><div class="principles"><span>私人记忆</span><span>证据提炼</span><span>团队共创</span></div></section>
    <section class="login-card"><span class="eyebrow">欢迎回来</span><h2>登录编剧工作台</h2><p class="muted">使用统一 KX 账号，延续你的创作。</p>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <form onsubmit={login}><label>账号<input name="username" bind:value={username} autocomplete="username" required maxlength="200" placeholder="输入 KX 账号" /></label><label>密码<input name="password" bind:value={password} type="password" autocomplete="current-password" required maxlength="1000" placeholder="输入密码" /></label><button class="primary" disabled={busy||loading}>{busy?'正在登录…':'登录'}</button></form>
    {#if loading}<p class="muted">正在加载登录方式…</p>{/if}
    {#if apps.length}<div class="divider">或使用钉钉</div><label>选择组织<select bind:value={selected}>{#each apps as app}<option value={app.app_key}>{app.app_name}</option>{/each}</select></label><a class="provider" href={`/api/auth/start?provider=kx&app=${encodeURIComponent(selected)}`}>钉钉授权登录 ↗</a>{/if}
    {#if oauth}<a class="provider" href="/api/auth/start?provider=oauth">通过 {oauth} 登录 ↗</a>{/if}
    <p class="note">私人空间仅本人可见。加入共享空间后，该空间内的资料由成员共同使用。</p></section>
</main>
<style>
.login-page{height:100dvh;overflow-y:auto;display:grid;grid-template-columns:1.1fr 1fr;align-items:center;gap:9vw;padding:7vw;color:#e4eee3;background:radial-gradient(ellipse at 18% 35%,#1d372c,transparent 55%),#080f0d;font-family:system-ui,sans-serif}.eyebrow{font-size:11px;letter-spacing:.2em;color:#9aaa8e}.intro h1{font-family:"Songti SC",serif;font-size:clamp(32px,4.5vw,64px);line-height:1.45;font-weight:500;margin:28px 0}.intro p{color:#9eafa3;line-height:2;font-size:14px}.principles{display:flex;gap:28px;margin-top:45px;color:#aab695;font-size:12px}.login-card{max-width:440px;width:100%;padding:34px;border:1px solid #334438;border-radius:18px;background:#101b16dd;box-shadow:0 25px 90px #0005}.login-card h2{font-size:24px;margin:14px 0 8px}.muted,.note{color:#95a398;font-size:12px;line-height:1.8}.note{margin-top:25px;font-size:11px}form{display:grid;gap:18px;margin:26px 0}label{display:grid;gap:8px;font-size:12px;color:#c2d1bd}input,select{width:100%;padding:12px;background:#09110d;color:#e6eee2;border:1px solid #3f5543;border-radius:7px;font:inherit;box-sizing:border-box}input:focus,select:focus{outline:2px solid #a5b77c;outline-offset:2px}.primary,.provider{display:block;width:100%;text-align:center;padding:12px;border-radius:7px;text-decoration:none;font-size:13px;box-sizing:border-box}.primary{background:#c0c88b;color:#182518;border:0;cursor:pointer}.primary:disabled{opacity:.5}.provider{margin-top:14px;border:1px solid #617956;color:#d8e9cf}.divider{text-align:center;color:#728374;font-size:11px;margin:20px 0}.error{padding:12px;border:1px solid #86534d;border-radius:7px;background:#5a2b2422;color:#efb7ab;font-size:12px}@media(max-width:760px){.login-page{grid-template-columns:1fr;gap:30px;padding:35px 20px}.intro h1{font-size:30px;margin:14px 0}.intro p,.principles{display:none}.login-card{margin:auto;padding:25px}}
</style>

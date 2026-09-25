const {createRequire}=require('node:module');
const {mkdirSync,writeFileSync}=require('node:fs');
const {resolve}=require('node:path');
const assert=require('node:assert/strict');
const {chromium}=createRequire(resolve('apps/dashboard/package.json'))('@playwright/test');
const base=process.env.VESTIGE_TEST_UI_URL||'http://127.0.0.1:5173';
const out=process.env.VESTIGE_TEST_OUTPUT||'/tmp/vestige-identity-browser';mkdirSync(out,{recursive:true});
const errors=[];const marker=Date.now().toString();
async function action(p,path,body){return p.evaluate(async({path,body})=>{const r=await fetch(path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});return {status:r.status,value:await r.json()};},{path,body});}
async function login(browser,user){const ctx=await browser.newContext({viewport:{width:1440,height:1000}});const p=await ctx.newPage();p.on('pageerror',e=>errors.push(String(e)));await p.goto(base+'/dashboard/writer');await p.waitForURL('**/dashboard/login');await p.getByLabel('账号',{exact:true}).fill(user);await p.getByLabel('密码',{exact:true}).fill('test-only-password');await p.getByRole('button',{name:'登录',exact:true}).click();await p.waitForURL('**/dashboard/writer');await p.getByRole('heading',{name:'编剧工作台',level:1}).waitFor();return p;}
(async()=>{
const browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{}),args:['--enable-unsafe-webgpu','--use-angle=metal']});
try{
const entry=await browser.newPage();await entry.goto(base+'/dashboard/login');await entry.getByRole('link',{name:'钉钉授权登录 ↗'}).waitFor();await entry.screenshot({path:out+'/login-desktop.png'});await entry.setViewportSize({width:390,height:844});await entry.screenshot({path:out+'/login-mobile.png'});await entry.getByRole('link',{name:'钉钉授权登录 ↗'}).click();await entry.waitForURL('**/dashboard/writer');await entry.close();
const alice=await login(browser,'alice');const bob=await login(browser,'bob');const carol=await login(browser,'carol');
const privateRole=await action(alice,'/api/writer/writer_role',{action:'create',name:'私人角色-'+marker});assert.equal(privateRole.status,200);const privateId=privateRole.value.role.id;
assert.equal((await action(bob,'/api/writer/writer_role',{action:'get',role_id:privateId})).status,404);
await alice.goto(base+'/dashboard/account');await alice.getByLabel('新建共享空间').fill('短剧共创组-'+marker);await alice.getByRole('button',{name:'创建共享空间',exact:true}).click();await alice.waitForURL('**/dashboard/writer');
await alice.goto(base+'/dashboard/account');await alice.getByRole('button',{name:'生成一次性邀请码'}).click();const invitation=await alice.locator('.secret code').innerText();
await bob.goto(base+'/dashboard/account');await bob.getByLabel('接受邀请').fill(invitation);await bob.getByRole('button',{name:'加入共享空间'}).click();await bob.waitForURL('**/dashboard/writer');
const iv=await action(alice,'/api/auth/account',{action:'invite',role:'viewer'});assert.equal(iv.status,200);const joined=await action(carol,'/api/auth/account',{action:'join',invitation:iv.value.invitation});assert.equal(joined.status,200);await action(carol,'/api/auth/account',{action:'switch',workspace:joined.value.workspace});await carol.reload();
await alice.goto(base+'/dashboard/writer');await alice.getByRole('button',{name:'新建编剧角色',exact:true}).click();await alice.getByLabel('角色名称',{exact:true}).fill('悬疑短剧编剧-'+marker);await alice.getByLabel('创作定位').fill('多人共同提炼，重视人物动机与集尾悬念。');await alice.getByRole('button',{name:'创建角色',exact:true}).click();await alice.getByText('编剧角色已创建，现在可以上传作品或提出创作偏好。').waitFor();
const roles=(await action(alice,'/api/writer/writer_role',{action:'list'})).value.roles;const role=roles.find(r=>r.name==='悬疑短剧编剧-'+marker);assert(role);
await alice.locator('input[type=file]').setInputFiles({name:'共同创作-甲.txt',mimeType:'text/plain',buffer:Buffer.from('第一集\n内景 车站 夜\n林舟握紧车票。\n林舟：这不是昨天的日期。\n广播骤然停止。')});await alice.getByText(/已解析为/).waitFor();
const attached=await action(bob,'/api/writer/writer_source',{action:'attach_text',role_id:role.id,title:'乙补充的创作方法',content:'每集先展示主角的明确目标，再以行动阻碍推动冲突；悬念应回到人物选择。'});assert.equal(attached.status,200);
await alice.getByRole('button',{name:'刷新',exact:true}).click();await alice.getByText('乙补充的创作方法',{exact:true}).waitFor();await alice.screenshot({path:out+'/shared-writer.png'});
assert.equal((await action(carol,'/api/writer/writer_source',{action:'list',role_id:role.id})).value.sources.length,2);
assert.equal((await action(carol,'/api/writer/writer_role',{action:'create',name:'forbidden'})).status,403);
assert.equal((await action(bob,'/api/writer/writer_role',{action:'publish',role_id:role.id,version:0,expected_version:0})).status,403);
await bob.reload();await bob.getByRole('button',{name:'对话调校',exact:true}).click();const chat=bob.locator('textarea').filter({hasNot:bob.locator('[readonly]')});
const task=await action(bob,'/api/writer/writer_task',{action:'create',role_id:role.id,kind:'role_chat',input:{message:'减少巧合，让反转来自人物动机。'}});assert.equal(task.status,200);
const credential=await action(bob,'/api/auth/account',{action:'create_token',label:'验收 Agent',role:'editor'});assert.equal(credential.status,200);
const init=await fetch(base+'/mcp',{method:'POST',headers:{Authorization:'Bearer '+credential.value.token,Accept:'application/json, text/event-stream','Content-Type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:1,method:'initialize',params:{protocolVersion:'2025-03-26',capabilities:{},clientInfo:{name:'identity-acceptance',version:'1'}}})});assert.equal(init.status,200);const session=init.headers.get('mcp-session-id');const protocol=init.headers.get('mcp-protocol-version');assert(session);
const r=await fetch(base+'/mcp',{method:'POST',headers:{Authorization:'Bearer '+credential.value.token,Accept:'application/json, text/event-stream','Content-Type':'application/json','Mcp-Session-Id':session,'Mcp-Protocol-Version':protocol},body:JSON.stringify({jsonrpc:'2.0',id:2,method:'tools/call',params:{name:'writer_source',arguments:{action:'list',role_id:role.id}}})});assert.equal(r.status,200);assert(!(await r.json()).result.isError);
await alice.goto(base+'/dashboard/account');await alice.getByRole('heading',{name:/的成员/}).waitFor();await alice.screenshot({path:out+'/members-desktop.png'});await alice.setViewportSize({width:390,height:844});await alice.screenshot({path:out+'/members-mobile.png',fullPage:true});
const members=(await action(alice,'/api/auth/account',{action:'members'})).value.members;const bobUser=members.find(m=>m.name==='陈编剧');assert(bobUser);assert.equal((await action(alice,'/api/auth/account',{action:'remove_member',user_id:bobUser.id})).status,200);
const revoked=await fetch(base+'/mcp',{method:'POST',headers:{Authorization:'Bearer '+credential.value.token,Accept:'*/*','Content-Type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:3,method:'ping'})});assert.equal(revoked.status,401);
assert.deepEqual(errors,[]);writeFileSync(out+'/acceptance.json',JSON.stringify({passed:true,users:3,password:true,dingtalkRoundtrip:'mock KX',privateIsolation:true,sharedContributions:2,viewerWriteDenied:true,editorPublishDenied:true,mcp:true,revocation:true,jsErrors:errors},null,2));console.log('Identity browser acceptance passed');
}finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});

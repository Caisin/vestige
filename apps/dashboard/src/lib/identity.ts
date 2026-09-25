import { writable } from 'svelte/store';
export type Space = { id: string; name: string; kind: 'personal'|'shared'|'legacy'; role: string };
export type Identity = { enabled: boolean; user?: {id:string;name:string;subject:string}; workspace?:string; role?:string; workspaces?:Space[] };
export const identity = writable<Identity>({enabled:false});
export async function account(action:string,args:Record<string,unknown> = {}) {
    const response=await fetch('/api/auth/account',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({action,...args})});
    const result=await response.json();
    if(!response.ok)throw new Error(result.error||'操作失败');
    if (['switch','logout'].includes(action)) localStorage.setItem('vestige-identity-change',String(Date.now()));
    return result;
}
export async function loadIdentity():Promise<Identity> {
    const r=await fetch('/api/auth/me');
    if(r.status===401){identity.set({enabled:true});return {enabled:true};}
    if(!r.ok)throw new Error('暂时无法验证身份，请重试');
    const value=await r.json();identity.set(value);return value;
}

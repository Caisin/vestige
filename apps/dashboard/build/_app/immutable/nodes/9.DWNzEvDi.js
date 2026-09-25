import{$ as e,A as t,E as n,F as r,I as i,M as a,O as o,Q as s,R as c,T as l,W as u,X as d,Y as f,Z as p,_ as m,a as h,b as g,d as _,dt as v,et as y,f as b,ht as x,j as S,k as C,mt as w,n as T,nt as E,r as D,tt as O,u as k,ut as A,v as j,y as M}from"../chunks/BAUNE5Zx.js";import"../chunks/xihTtKlq.js";import{t as N}from"../chunks/BzAMLulQ.js";import{n as P}from"../chunks/BGA2R-TK.js";import{t as F}from"../chunks/Cwq8aIcs.js";import{t as ee}from"../chunks/DWzOdnYU.js";import{a as I,c as L,i as R,o as z,u as B}from"../chunks/DjOZR5rI.js";import{t as V}from"../chunks/DfPA8DhW.js";import{t as te}from"../chunks/DDIu7tpI.js";import{n as H,t as ne}from"../chunks/Bosc6v9J.js";function U(e){return e>=.92?`near-identical`:e>=.8?`strong`:`weak`}function W(e){let t=U(e);return t===`near-identical`?`var(--color-decay)`:t===`strong`?`var(--color-warning)`:`#fde047`}function re(e){let t=U(e);return t===`near-identical`?`Near-identical`:t===`strong`?`Strong match`:`Weak match`}function ie(e){return e>.7?`#10b981`:e>.4?`#f59e0b`:`#ef4444`}function ae(e){if(!e||e.length===0)return null;let t=e[0],n=Number.isFinite(t.retention)?t.retention:-1/0;for(let r=1;r<e.length;r++){let i=e[r],a=Number.isFinite(i.retention)?i.retention:-1/0;a>n&&(t=i,n=a)}return t}function G(e){return e.map(e=>e.id).slice().sort().join(`|`)}function oe(e,t=80){if(!e)return``;let n=e.trim().replace(/\s+/g,` `);return n.length<=t?n:n.slice(0,t)+`…`}function se(e){if(!e||typeof e!=`string`)return``;let t=new Date(e);return Number.isNaN(t.getTime())?``:t.toLocaleDateString(`zh-CN`,{year:`numeric`,month:`short`,day:`numeric`})}function ce(e,t=4){return Array.isArray(e)?e.slice(0,t):[]}function le(e){let t=e.diff?.invalidatedIds?.length??Math.max(0,e.memberIds.length-1),n=typeof e.confidence==`number`?e.confidence:Number.parseFloat(e.confidence),r=Number.isFinite(n)?`${Math.round(n*100)}%`:`unknown`,i=t===1?`memory`:`memories`;return`Keeps ${e.survivorId.slice(0,8)}, folds ${t} ${i} into it. Matcher: ${e.classification} at ${r}. Reversible with dedup undo.`}var ue=a(`<span class="flex-shrink-0 rounded-full border border-warning/50 bg-warning/10 px-3 py-1 text-xs font-medium text-warning"> </span>`),de=a(`<span> </span>`),fe=a(`<span class="rounded bg-recall/15 px-1.5 py-0.5 text-[10px] font-medium text-recall"> </span>`),pe=a(`<span class="rounded bg-white/[0.04] px-1.5 py-0.5 text-[10px] text-muted"> </span>`),me=a(`<div class="text-[11px] text-muted"> </div>`),he=a(`<div><span class="mt-1.5 h-2 w-2 flex-shrink-0 rounded-full"></span> <div class="flex-1 min-w-0 space-y-1.5"><div class="flex flex-wrap items-center gap-1.5"><span class="text-xs text-dim"> </span> <!> <!></div> <p> </p> <!></div> <div class="flex flex-shrink-0 flex-col items-end gap-1"><div class="h-1.5 w-12 overflow-hidden rounded-full bg-deep"><div class="h-full rounded-full"></div></div> <span class="text-[11px] text-muted"> </span></div></div>`),ge=a(`<div class="rounded-xl border border-warning/20 bg-warning/5 p-3 text-xs text-dim"> </div>`),_e=a(`<div class="rounded-xl border border-synapse/25 bg-synapse/5 p-3 text-xs"><div class="font-mono text-[11px] uppercase tracking-[0.18em] text-synapse-glow"> </div> <div class="mt-1 text-text"> </div> <div class="mt-1 text-muted"> </div> <div class="mt-2 max-h-24 overflow-hidden text-muted"> </div> <div class="mt-3 flex flex-wrap items-center gap-2"><button type="button" class="rounded-lg bg-synapse/25 px-3 py-1.5 text-xs font-medium text-synapse-glow transition hover:bg-synapse/35 disabled:opacity-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60"> </button> <button type="button" class="rounded-lg bg-white/[0.04] px-3 py-1.5 text-xs text-dim transition hover:bg-white/[0.08] hover:text-text focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60"> </button></div></div>`),ve=a(`<div class="rounded-xl border border-consolidated/25 bg-consolidated/5 p-3 text-xs text-text"> <span class="font-mono"> </span>.</div>`),ye=a(`<div class="rounded-xl border border-decay/25 bg-decay/5 p-3 text-xs text-decay" role="alert"> </div>`),be=a(`<div class="glass-panel rounded-2xl p-5 space-y-4 transition-all duration-300 hover:border-synapse/20"><div class="flex items-start justify-between gap-4"><div class="flex-1 min-w-0 space-y-1.5"><div class="flex items-center gap-3"><span class="text-sm font-semibold"> </span> <span class="text-xs text-dim"> </span> <span class="text-xs text-muted"> </span></div> <div class="h-2 w-full overflow-hidden rounded-full bg-deep/60" role="progressbar" aria-valuemin="0" aria-valuemax="100"><div class="h-full rounded-full transition-all duration-500"></div></div></div> <!></div> <div class="space-y-2"><!> <!></div> <!> <!> <!> <div class="flex flex-wrap items-center gap-2 pt-1"><button type="button"> </button> <button type="button" class="rounded-lg bg-dream/20 px-3 py-1.5 text-xs font-medium text-dream-glow transition hover:bg-dream/30 focus:outline-none focus-visible:ring-2 focus-visible:ring-dream-glow/60"> </button> <button type="button" class="ml-auto rounded-lg bg-white/[0.04] px-3 py-1.5 text-xs text-dim transition hover:bg-white/[0.08] hover:text-text focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60"> </button></div></div>`);function xe(e,r){v(r,!0);let a=h(r,`oversized`,3,!1),g=O(null),_=O(!1),T=O(!1),D=O(null),k=O(null),F=E(()=>!a()&&!!r.onPlan&&!!r.onApply&&!c(k));async function ee(){if(r.onPlan&&!c(_)){y(_,!0),y(D,null);try{y(g,await r.onPlan(r.memories.map(e=>e.id)),!0)}catch(e){y(D,e instanceof Error?e.message:N(`Could not plan the merge`),!0)}finally{y(_,!1)}}}async function I(){if(r.onApply&&c(g)&&!c(T)){y(T,!0),y(D,null);try{y(k,await r.onApply(c(g).planId),!0),r.onMerged?.(c(k))}catch(e){y(D,e instanceof Error?e.message:N(`Could not apply the merge`),!0)}finally{y(T,!1)}}}let L=O(!1),R=E(()=>ae(r.memories)),z=E(()=>{if(r.memories.length<=12)return r.memories;let e=r.memories.filter(e=>e.id!==c(R)?.id);return c(R)?[c(R),...e.slice(0,11)]:e.slice(0,12)}),B=E(()=>r.memories.length-c(z).length);var V=S(),te=d(V),H=e=>{var d=be(),h=f(d),v=f(h),S=f(v),O=f(S),A=p(O),V=s(O,2),te=p(V,!0),H=s(V,2),ne=p(H);x(S);var U=s(S,2),ae=p(U);x(v);var G=s(v,2),xe=e=>{var n=ue(),r=p(n,!0);u(e=>C(r,e),[()=>N(`REVIEW REQUIRED · NOT SAFE TO MERGE`)]),t(e,n)},K=e=>{var n=de(),i=p(n);u((e,t)=>{j(n,1,`flex-shrink-0 rounded-full border px-3 py-1 text-xs font-medium ${r.suggestedAction===`merge`?`border-recall/40 bg-recall/10 text-recall`:`border-dream-glow/40 bg-dream/10 text-dream-glow`}`),C(i,`${e??``} ${t??``}`)},[()=>N(`Classification:`),()=>r.suggestedAction===`merge`?N(`merge candidate`):N(`review`)]),t(e,n)};o(G,e=>{a()?e(xe):e(K,-1)}),x(h);var q=s(h,2),J=f(q);l(J,17,()=>c(z),e=>e.id,(e,r)=>{var i=he(),a=f(i),d=s(a,2),h=f(d),g=f(h),_=p(g,!0),v=s(g,2),y=e=>{var n=fe(),r=p(n,!0);u(e=>C(r,e),[()=>N(`WINNER`)]),t(e,n)};o(v,e=>{c(r).id===c(R).id&&e(y)});var S=s(v,2);l(S,17,()=>ce(c(r).tags,4),n,(e,n)=>{var r=pe(),i=p(r,!0);u(()=>C(i,c(n))),t(e,r)}),x(h);var w=s(h,2),T=p(w,!0),D=s(w,2),O=e=>{var n=me(),i=p(n,!0);u(e=>C(i,e),[()=>se(c(r).createdAt)]),t(e,n)},k=E(()=>se(c(r).createdAt));o(D,e=>{c(k)&&e(O)}),x(d);var A=s(d,2),M=f(A),F=p(M),ee=s(M,2),I=p(ee);x(A),x(i),u((e,t,n,o)=>{j(i,1,`group flex items-start gap-3 rounded-xl border border-synapse/5 bg-white/[0.02] p-3 transition-all duration-200 hover:border-synapse/20 hover:bg-white/[0.04] ${c(r).id===c(R).id?`ring-1 ring-recall/30`:``}`),m(a,`background: ${(P[c(r).nodeType]||`#8B95A5`)??``}`),b(a,`title`,c(r).nodeType),C(_,e),j(w,1,`text-sm text-text leading-relaxed ${c(L)?`whitespace-pre-wrap`:``}`),C(T,t),m(F,`width: ${c(r).retention*100}%; background: ${n??``}`),C(I,`${o??``}%`)},[()=>N(String(c(r).nodeType)),()=>c(L)?c(r).content:oe(c(r).content),()=>ie(c(r).retention),()=>(c(r).retention*100).toFixed(0)]),t(e,i)});var Y=s(J,2),Se=e=>{var n=ge(),r=p(n);u(e=>C(r,`+${c(B)??``} ${e??``}`),[()=>N(`linked candidates — oversized similarity component. Members chain through pairwise similarity; distant members may be unrelated. Raise the threshold to split it.`)]),t(e,n)};o(Y,e=>{c(B)>0&&e(Se)}),x(q);var Ce=s(q,2),we=e=>{var n=_e(),r=f(n),a=p(r,!0),o=s(r,2),l=p(o,!0),d=s(o,2),m=p(d,!0),h=s(d,2),_=p(h),v=s(h,2),b=f(v),S=p(b,!0),w=s(b,2),E=p(w,!0);x(v),x(n),u((e,t,n,r,i,o)=>{C(a,e),C(l,t),C(m,c(g).explanation),C(_,`${n??``} ${r??``}`),b.disabled=c(T),C(S,i),w.disabled=c(T),C(E,o)},[()=>N(`Merge preview · nothing written yet`),()=>le(c(g)),()=>N(`Result:`),()=>oe(c(g).diff.resultContent,240),()=>c(T)?N(`Applying…`):N(`Apply merge`),()=>N(`Cancel`)]),i(`click`,b,I),i(`click`,w,()=>y(g,null)),t(e,n)};o(Ce,e=>{c(g)&&!c(k)&&e(we)});var Te=s(Ce,2),Ee=e=>{var n=ve(),r=f(n),i=s(r),a=p(i,!0);w(),x(n),u((e,t,n)=>{C(r,`${e??``} ${t??``}${n??``} `),C(a,c(k).operationId)},[()=>N(`Merged into`),()=>c(k).survivorId.slice(0,8),()=>N(`. Reversible: run dedup undo with operation id`)]),t(e,n)};o(Te,e=>{c(k)&&e(Ee)});var De=s(Te,2),Oe=e=>{var n=ye(),r=p(n,!0);u(()=>C(r,c(D))),t(e,n)};o(De,e=>{c(D)&&e(Oe)});var X=s(De,2),Z=f(X),Q=p(Z,!0),$=s(Z,2),ke=p($,!0),Ae=s($,2),je=p(Ae,!0);x(X),x(d),u((e,t,n,i,a,o,s,l,u,d,f,p,h,v,y)=>{m(O,`color: ${e??``}`),C(A,`${t??``}%`),C(te,n),C(ne,`· ${r.memories.length??``} ${i??``}`),b(U,`aria-label`,a),b(U,`aria-valuenow`,o),m(ae,`width: ${s??``}%; background: ${l??``}; box-shadow: 0 0 12px ${u??``}66`),Z.disabled=!c(F)||c(_)||!!c(g),b(Z,`aria-disabled`,!c(F)),b(Z,`aria-label`,d),j(Z,1,M(c(F)?`rounded-lg bg-synapse/20 px-3 py-1.5 text-xs font-medium text-synapse-glow transition hover:bg-synapse/30 disabled:opacity-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60`:`cursor-not-allowed rounded-lg bg-white/[0.03] px-3 py-1.5 text-xs font-medium text-muted/60`)),b(Z,`title`,f),C(Q,p),b($,`aria-expanded`,c(L)),C(ke,h),b(Ae,`aria-label`,v),C(je,y)},[()=>W(r.similarity),()=>(r.similarity*100).toFixed(1),()=>re(r.similarity),()=>N(`memories`),()=>N(`Cosine similarity`),()=>Math.round(r.similarity*100),()=>(r.similarity*100).toFixed(1),()=>W(r.similarity),()=>W(r.similarity),()=>a()?N(`Merge is not safe for an oversized component`):N(`Preview a reversible merge`),()=>a()?N(`Oversized similarity component: members chain through pairwise similarity, so a merge could fold unrelated memories together`):N(`Preview first; nothing is written until you apply`),()=>a()?N(`Merge unsafe here`):c(_)?N(`Planning…`):N(`Preview merge`),()=>c(L)?N(`Collapse`):N(`Review`),()=>N(`Dismiss cluster for this session`),()=>N(`Dismiss cluster`)]),i(`click`,Z,ee),i(`click`,$,()=>y(L,!c(L))),i(`click`,Ae,function(...e){r.onDismiss?.apply(this,e)}),t(e,d)};o(te,e=>{r.memories.length>0&&c(R)&&e(H)}),t(e,V),A()}r([`click`]);var K=`rgba16float`,q=512,J=512,Y=16,Se=16,Ce=`
struct Params {
	frame: f32,
	loop_phase: f32,
	node_count: f32,
	edge_count: f32,
	path_count: f32,
	pulse: f32,
	viewport_w: f32,
	viewport_h: f32,
	brightness: f32,
	demo_id: f32,
	time: f32,
	capture_mode: f32,
	live_kind: f32,
	live_frame: f32,
	live_energy: f32,
	projection_days: f32,
};

struct FusionCell {
	// x/y position in NDC, z retention, w winner flag
	pos_retention: vec4f,
	// x similarity, y threshold, z member slot, w cluster slot
	cluster_meta: vec4f,
	// x mismatch intensity, y merge flag, z radius, w member count
	visual_meta: vec4f,
	// x cell index, y cluster index, z/w spare
	ids: vec4f,
};

struct FusionNeck {
	// x/y winner position, z winner retention, w winner radius
	a: vec4f,
	// x/y candidate position, z candidate retention, w candidate radius
	b: vec4f,
	// x similarity, y threshold, z mismatch intensity, w merge flag
	signals: vec4f,
	// x neck index, y cluster index, z/w spare
	ids: vec4f,
};
`,we=`
${Ce}

// FieldOpts mirrors the membrane's: x=intensity, yz=well center NDC, w=well
// half-w; then well half-h, floor, soft, pad. Cells/necks dim by the same amount
// so nothing blows out under the centered text overlay.
struct FieldOpts {
	intensity_wx_wy_hw: vec4f,
	hh_floor_soft_pad: vec4f,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> cells: array<FusionCell>;
@group(0) @binding(2) var<storage, read> necks: array<FusionNeck>;
@group(0) @binding(5) var<uniform> opts: FieldOpts;

const QUAD = array<vec2f, 6>(
	vec2f(-1.0, -1.0), vec2f(1.0, -1.0), vec2f(1.0, 1.0),
	vec2f(-1.0, -1.0), vec2f(1.0, 1.0), vec2f(-1.0, 1.0)
);

struct VSOut {
	@builtin(position) clip: vec4f,
	@location(0) uv: vec2f,
	@location(1) @interpolate(flat) misc: vec4f,
	@location(2) @interpolate(flat) home: vec2f,
};

fn similarity_neck(similarity: f32) -> f32 {
	return smoothstep(0.78, 0.98, similarity);
}

// Reading-well multiplier at an NDC point (1.0 outside, →floor inside). hw<=0 off.
fn field_dim(ndc: vec2f) -> f32 {
	let intensity = clamp(opts.intensity_wx_wy_hw.x, 0.0, 1.0);
	let hw = opts.intensity_wx_wy_hw.w;
	if (hw <= 0.0) { return intensity; }
	let center = opts.intensity_wx_wy_hw.yz;
	let hh = opts.hh_floor_soft_pad.x;
	let floor_v = opts.hh_floor_soft_pad.y;
	let soft = max(0.02, opts.hh_floor_soft_pad.z);
	let d = abs(ndc - center) - vec2f(hw, hh);
	let outside = length(max(d, vec2f(0.0)));
	let inside = min(max(d.x, d.y), 0.0);
	let sd = outside + inside;
	let t = smoothstep(-soft, 0.0, sd);
	return intensity * mix(floor_v, 1.0, t);
}

@vertex
fn vs_splat(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VSOut {
	var out: VSOut;
	let corner = QUAD[vi];
	let cell_count = u32(params.node_count);
	if (ii < cell_count) {
		let c = cells[ii];
		let merge_gate = c.visual_meta.y;
		let radius = c.visual_meta.z * (1.0 + 0.045 * sin(params.time * 2.0 + c.cluster_meta.w * 6.28318));
		out.clip = vec4f(c.pos_retention.xy + corner * radius, 0.0, 1.0);
		out.uv = corner;
		out.misc = vec4f(c.pos_retention.z, c.cluster_meta.x, c.visual_meta.x, merge_gate);
		out.home = c.pos_retention.xy;
	} else {
		let n = necks[ii - cell_count];
		let a = n.a.xy;
		let b = n.b.xy;
		let center = (a + b) * 0.5;
		let dir = normalize(b - a + vec2f(0.0001, 0.0001));
		let normal = vec2f(-dir.y, dir.x);
		let fused = similarity_neck(n.signals.x);
		let length_half = distance(a, b) * 0.5;
		let thickness = 0.035 + fused * 0.085 + n.signals.z * 0.025;
		let pos = center + dir * corner.x * length_half + normal * corner.y * thickness;
		out.clip = vec4f(pos, 0.0, 1.0);
		out.uv = vec2f(corner.x, corner.y / max(0.001, thickness));
		out.misc = vec4f(n.signals.x, fused, n.signals.z, n.signals.w);
		out.home = center;
	}
	return out;
}

@fragment
fn fs_splat(frag: VSOut) -> @location(0) vec4f {
	let d = length(frag.uv);
	let is_neck = f32(abs(frag.uv.y) > 1.0);
	if (is_neck < 0.5 && d > 1.0) { discard; }
	let retention = clamp(frag.misc.x, 0.0, 1.0);
	let similarity = clamp(frag.misc.y, 0.0, 1.0);
	let mismatch = clamp(frag.misc.z, 0.0, 1.0);
	let merge_gate = frag.misc.w;
	let cell_body = exp(-d * d * 3.15) * (0.38 + retention * 0.62) * (0.5 + similarity * 0.58);
	let cell_rim = smoothstep(0.24, 0.02, abs(d - (0.58 + retention * 0.16))) * (0.2 + similarity * 0.55);
	let neck_body = exp(-frag.uv.y * frag.uv.y * 4.0) * smoothstep(1.05, 0.82, abs(frag.uv.x)) * (0.35 + similarity * 0.9);
	let density = max(cell_body + cell_rim, neck_body * (0.4 + similarity));
	// The splat writes the density FIELD (blurred into the membrane). It must NOT
	// be dimmed here — the membrane fragment applies intensity + reading well once,
	// so dimming both would double-darken. r=density, g=retention, b=mismatch amber.
	return vec4f(density, density * (0.35 + retention * 0.65), mismatch * (0.18 + merge_gate * 0.12), 1.0);
}

@vertex
fn vs_cell(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VSOut {
	var out: VSOut;
	let c = cells[ii];
	let corner = QUAD[vi];
	let winner = c.pos_retention.w;
	let radius = c.visual_meta.z * (0.46 + winner * 0.18);
	out.clip = vec4f(c.pos_retention.xy + corner * radius, 0.0, 1.0);
	out.uv = corner;
	out.misc = vec4f(c.pos_retention.z, c.cluster_meta.x, c.visual_meta.x, winner);
	out.home = c.pos_retention.xy;
	return out;
}

@fragment
fn fs_cell(frag: VSOut) -> @location(0) vec4f {
	let d = length(frag.uv);
	if (d > 1.0) { discard; }
	let retention = clamp(frag.misc.x, 0.0, 1.0);
	let similarity = clamp(frag.misc.y, 0.0, 1.0);
	let mismatch = clamp(frag.misc.z, 0.0, 1.0);
	let winner = frag.misc.w;
	let sediment = vec3f(0.54, 0.29, 0.09);
	let recall = vec3f(0.16, 0.95, 0.66);
	let luciferin = vec3f(0.91, 1.0, 0.72);
	let ivory = vec3f(0.96, 0.945, 0.815);
	let amber = vec3f(1.0, 0.69, 0.08);
	let core = mix(sediment, mix(recall, luciferin, retention), retention);
	let rim = smoothstep(0.98, 0.72, d) * (1.0 - smoothstep(0.72, 0.22, d));
	let body = exp(-d*d*3.2) * (0.20 + retention * 0.44 + winner * 0.16);
	let mismatch_ring = smoothstep(0.16, 0.0, abs(d - 0.80)) * mismatch;
	let color = core * body + ivory * rim * (0.16 + similarity * 0.52) + amber * mismatch_ring * 0.34;
	// Sharp cells draw on TOP of the membrane, so dim them by the same field
	// intensity + reading well or they'd punch through the centered text.
	return vec4f(color * field_dim(frag.home), 1.0);
}

@vertex
fn vs_neck(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VSOut {
	var out: VSOut;
	let n = necks[ii];
	let a = n.a.xy;
	let b = n.b.xy;
	let t = f32(vi / 2u) / 31.0;
	let side = f32(vi % 2u) * 2.0 - 1.0;
	let dir = normalize(b - a + vec2f(0.0001, 0.0001));
	let normal = vec2f(-dir.y, dir.x);
	let midpoint = (a + b) * 0.5;
	let fused = similarity_neck(n.signals.x);
	let threshold_pull = clamp(n.signals.x - n.signals.y + 0.22, 0.0, 1.0);
	let bow = normal * sin(t * 3.14159) * (0.030 + n.signals.z * 0.050) * (1.0 - fused * 0.35);
	let pos = mix(a, b, t) + bow;
	let thickness = 0.005 + fused * 0.025 + threshold_pull * 0.010;
	out.clip = vec4f(pos + normal * side * thickness, 0.0, 1.0);
	out.uv = vec2f(t, side);
	out.misc = vec4f(n.signals.x, n.signals.y, n.signals.z, distance(pos, midpoint));
	out.home = midpoint;
	return out;
}

@fragment
fn fs_neck(frag: VSOut) -> @location(0) vec4f {
	let similarity = clamp(frag.misc.x, 0.0, 1.0);
	let threshold = clamp(frag.misc.y, 0.0, 1.0);
	let mismatch = clamp(frag.misc.z, 0.0, 1.0);
	let pulse = 0.55 + 0.45 * sin(36.0 * frag.uv.x - 8.0 * frag.misc.w);
	let bridge = vec3f(0.10, 0.82, 0.92);
	let luciferin = vec3f(0.91, 1.0, 0.72);
	let amber = vec3f(1.0, 0.69, 0.08);
	let pull = smoothstep(-0.08, 0.20, similarity - threshold);
	let color = mix(bridge, luciferin, pull) + amber * mismatch * pulse * 0.34;
	// Necks draw on TOP of the membrane too — dim by field intensity + reading well.
	return vec4f(color * (0.14 + similarity * 0.55 + mismatch * 0.18) * field_dim(frag.home), 1.0);
}
`,Te=`
${Ce}

// FieldOpts: x=intensity (0..1 overall dim), yz=well center NDC, w=well half-w,
// then well half-h, floor (min emission inside well), soft (edge falloff), pad.
// Lets a text-heavy organ dim the whole field AND carve a reading well under the
// centered DOM overlay so the labels/values read. hw<=0 disables the well.
struct FieldOpts {
	intensity_wx_wy_hw: vec4f,
	hh_floor_soft_pad: vec4f,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(3) var field_sampler: sampler;
@group(0) @binding(4) var field_tex: texture_2d<f32>;
@group(0) @binding(5) var<uniform> opts: FieldOpts;

const QUAD = array<vec2f, 6>(
	vec2f(-1.0, -1.0), vec2f(1.0, -1.0), vec2f(1.0, 1.0),
	vec2f(-1.0, -1.0), vec2f(1.0, 1.0), vec2f(-1.0, 1.0)
);

struct VSOut { @builtin(position) clip: vec4f, @location(0) uv: vec2f };

@vertex
fn vs_fullscreen(@builtin(vertex_index) vi: u32) -> VSOut {
	var out: VSOut;
	let p = QUAD[vi];
	out.clip = vec4f(p, 0.0, 1.0);
	out.uv = p * 0.5 + vec2f(0.5);
	return out;
}

// Reading-well multiplier at an NDC point: 1.0 outside the well, falling toward
// the floor value inside it (smooth edge of width soft). Disabled when hw<=0.
fn reading_well(ndc: vec2f) -> f32 {
	let hw = opts.intensity_wx_wy_hw.w;
	if (hw <= 0.0) { return 1.0; }
	let center = opts.intensity_wx_wy_hw.yz;
	let hh = opts.hh_floor_soft_pad.x;
	let floor_v = opts.hh_floor_soft_pad.y;
	let soft = max(0.02, opts.hh_floor_soft_pad.z);
	let d = abs(ndc - center) - vec2f(hw, hh);
	// signed distance to rect edge: <0 inside, >0 outside
	let outside = length(max(d, vec2f(0.0)));
	let inside = min(max(d.x, d.y), 0.0);
	let sd = outside + inside;
	// sd<=-soft → fully inside (floor); sd>=0 → outside (1.0)
	let t = smoothstep(-soft, 0.0, sd);
	return mix(floor_v, 1.0, t);
}

@fragment
fn fs_membrane(frag: VSOut) -> @location(0) vec4f {
	let f = textureSample(field_tex, field_sampler, frag.uv);
	let density = clamp(f.r, 0.0, 5.0);
	let retention = clamp(f.g, 0.0, 5.0);
	let mismatch = clamp(f.b, 0.0, 3.0);
	let membrane = smoothstep(0.13, 0.88, density) * (1.0 - smoothstep(1.9, 3.8, density));
	let blackwater = vec3f(0.008, 0.012, 0.018);
	let bridge = vec3f(0.10, 0.82, 0.92);
	let luciferin = vec3f(0.66, 1.0, 0.37);
	let ivory = vec3f(0.96, 0.945, 0.815);
	let amber = vec3f(1.0, 0.69, 0.08);
	var color = blackwater * (0.18 + density * 0.055);
	color = color + bridge * density * 0.055 + luciferin * retention * 0.080;
	color = color + ivory * membrane * 0.22 + amber * mismatch * (0.20 + 0.08 * params.pulse);
	let vignette = smoothstep(0.96, 0.18, distance(frag.uv, vec2f(0.5)));
	let ndc = frag.uv * 2.0 - vec2f(1.0);
	let dim = clamp(opts.intensity_wx_wy_hw.x, 0.0, 1.0) * reading_well(ndc);
	return vec4f(color * (0.35 + 0.65 * vignette) * params.brightness * dim, 1.0);
}
`,Ee=`
struct BlurDir { dir: vec2f, _pad: vec2f };
@group(0) @binding(0) var blur_sampler: sampler;
@group(0) @binding(1) var blur_src: texture_2d<f32>;
@group(0) @binding(2) var<uniform> blur_dir: BlurDir;
const QUAD = array<vec2f, 6>(
	vec2f(-1.0, -1.0), vec2f(1.0, -1.0), vec2f(1.0, 1.0),
	vec2f(-1.0, -1.0), vec2f(1.0, 1.0), vec2f(-1.0, 1.0)
);
struct VSOut { @builtin(position) clip: vec4f, @location(0) uv: vec2f };
@vertex
fn vs_fullscreen(@builtin(vertex_index) vi: u32) -> VSOut {
	var out: VSOut;
	let p = QUAD[vi];
	out.clip = vec4f(p, 0.0, 1.0);
	out.uv = p * 0.5 + vec2f(0.5);
	return out;
}
@fragment
fn fs_blur(frag: VSOut) -> @location(0) vec4f {
	let dims = vec2f(textureDimensions(blur_src, 0));
	let stepv = blur_dir.dir / max(dims, vec2f(1.0));
	var acc = textureSampleLevel(blur_src, blur_sampler, frag.uv - stepv * 2.0, 0.0) * 0.06136;
	acc = acc + textureSampleLevel(blur_src, blur_sampler, frag.uv - stepv, 0.0) * 0.24477;
	acc = acc + textureSampleLevel(blur_src, blur_sampler, frag.uv, 0.0) * 0.38774;
	acc = acc + textureSampleLevel(blur_src, blur_sampler, frag.uv + stepv, 0.0) * 0.24477;
	acc = acc + textureSampleLevel(blur_src, blur_sampler, frag.uv + stepv * 2.0, 0.0) * 0.06136;
	return acc;
}
`,De=class{engine;scene=null;resources=null;sampler=null;splatBindLayout=null;blurBindLayout=null;membraneBindLayout=null;splatPipeline=null;blurPipeline=null;membranePipeline=null;cellPipeline=null;neckPipeline=null;cellCount=0;neckCount=0;cellGeometry=[];neckGeometry=[];intensity=.22;well={x:0,y:0,hw:-1,hh:0,floor:.1,soft:.22};constructor(e,t){this.engine=e,this.uploadScene(t)}setIntensity(e){this.intensity=Math.min(1,Math.max(0,Number.isFinite(e)?e:.22));let t=this.engine.gpuDevice;t&&this.writeOpts(t)}setReadingWell(e){let t=(e,t=0)=>Number.isFinite(e)?e:t;this.well={x:t(e.x),y:t(e.y),hw:t(e.hw,-1),hh:t(e.hh),floor:Math.min(1,Math.max(0,t(e.floor??.1,.1))),soft:Math.max(.02,t(e.soft??.22,.22))};let n=this.engine.gpuDevice;n&&this.writeOpts(n)}writeOpts(e){this.resources&&e.queue.writeBuffer(this.resources.optsBuffer,0,new Float32Array([this.intensity,this.well.x,this.well.y,this.well.hw,this.well.hh,this.well.floor,this.well.soft,0]))}uploadScene(e){this.scene=e,this.buildGeometry();let t=this.engine.gpuDevice;t&&(this.ensurePipelines(t),this.ensureResources(t),this.uploadBuffers(t))}ensurePipelines(e){if(this.splatPipeline||!this.engine.paramsBuffer)return;let t=X(e,`duplicates-fusion-splat-wgsl`,we),n=X(e,`duplicates-fusion-blur-wgsl`,Ee),r=X(e,`duplicates-fusion-membrane-wgsl`,Te);this.splatBindLayout=e.createBindGroupLayout({label:`duplicates-fusion-splat-bind-layout`,entries:[{binding:0,visibility:GPUShaderStage.VERTEX|GPUShaderStage.FRAGMENT,buffer:{type:`uniform`}},{binding:1,visibility:GPUShaderStage.VERTEX,buffer:{type:`read-only-storage`}},{binding:2,visibility:GPUShaderStage.VERTEX,buffer:{type:`read-only-storage`}},{binding:5,visibility:GPUShaderStage.FRAGMENT,buffer:{type:`uniform`}}]}),this.blurBindLayout=e.createBindGroupLayout({label:`duplicates-fusion-blur-bind-layout`,entries:[{binding:0,visibility:GPUShaderStage.FRAGMENT,sampler:{type:`filtering`}},{binding:1,visibility:GPUShaderStage.FRAGMENT,texture:{sampleType:`float`}},{binding:2,visibility:GPUShaderStage.FRAGMENT,buffer:{type:`uniform`}}]}),this.membraneBindLayout=e.createBindGroupLayout({label:`duplicates-fusion-membrane-bind-layout`,entries:[{binding:0,visibility:GPUShaderStage.FRAGMENT,buffer:{type:`uniform`}},{binding:3,visibility:GPUShaderStage.FRAGMENT,sampler:{type:`filtering`}},{binding:4,visibility:GPUShaderStage.FRAGMENT,texture:{sampleType:`float`}},{binding:5,visibility:GPUShaderStage.FRAGMENT,buffer:{type:`uniform`}}]});let i=e.createPipelineLayout({label:`duplicates-fusion-splat-layout`,bindGroupLayouts:[this.splatBindLayout]}),a=e.createPipelineLayout({label:`duplicates-fusion-blur-layout`,bindGroupLayouts:[this.blurBindLayout]}),o=e.createPipelineLayout({label:`duplicates-fusion-membrane-layout`,bindGroupLayouts:[this.membraneBindLayout]});this.sampler=e.createSampler({magFilter:`linear`,minFilter:`linear`});let s={color:{srcFactor:`one`,dstFactor:`one`,operation:`add`},alpha:{srcFactor:`one`,dstFactor:`one`,operation:`add`}};this.splatPipeline=e.createRenderPipeline({label:`duplicates-field-additive-splat`,layout:i,vertex:{module:t,entryPoint:`vs_splat`},fragment:{module:t,entryPoint:`fs_splat`,targets:[{format:K,blend:s}]},primitive:{topology:`triangle-list`}}),this.blurPipeline=e.createRenderPipeline({label:`duplicates-field-blur-render-pass`,layout:a,vertex:{module:n,entryPoint:`vs_fullscreen`},fragment:{module:n,entryPoint:`fs_blur`,targets:[{format:K}]},primitive:{topology:`triangle-list`}}),this.membranePipeline=e.createRenderPipeline({label:`duplicates-synaptic-fusion-membrane`,layout:o,vertex:{module:r,entryPoint:`vs_fullscreen`},fragment:{module:r,entryPoint:`fs_membrane`,targets:[{format:this.engine.sceneFormat,blend:s}]},primitive:{topology:`triangle-list`}}),this.cellPipeline=e.createRenderPipeline({label:`duplicates-memory-nuclei`,layout:i,vertex:{module:t,entryPoint:`vs_cell`},fragment:{module:t,entryPoint:`fs_cell`,targets:[{format:this.engine.sceneFormat,blend:s}]},primitive:{topology:`triangle-list`}}),this.neckPipeline=e.createRenderPipeline({label:`duplicates-mismatch-filaments`,layout:i,vertex:{module:t,entryPoint:`vs_neck`},fragment:{module:t,entryPoint:`fs_neck`,targets:[{format:this.engine.sceneFormat,blend:s}]},primitive:{topology:`triangle-strip`}})}ensureResources(e){if(!this.splatBindLayout||!this.blurBindLayout||!this.membraneBindLayout||!this.engine.paramsBuffer||!this.sampler)return;let t=Math.max(16,Math.floor((this.engine.params[6]||1280)/2)),n=Math.max(16,Math.floor((this.engine.params[7]||720)/2)),r=!this.resources||this.resources.fieldSize[0]!==t||this.resources.fieldSize[1]!==n,i=this.resources?.cellBuffer,a=this.resources?.neckBuffer,o=this.resources?.blurHBuffer,s=this.resources?.blurVBuffer,c=this.resources?.optsBuffer;if(i||=e.createBuffer({label:`duplicates-cells`,size:q*Y*4,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_DST}),a||=e.createBuffer({label:`duplicates-necks`,size:J*Se*4,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_DST}),o||(o=e.createBuffer({label:`duplicates-blur-h-dir`,size:16,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST}),e.queue.writeBuffer(o,0,new Float32Array([1,0,0,0]))),s||(s=e.createBuffer({label:`duplicates-blur-v-dir`,size:16,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST}),e.queue.writeBuffer(s,0,new Float32Array([0,1,0,0]))),c||=e.createBuffer({label:`duplicates-field-opts`,size:32,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST}),!r&&this.resources){this.resources.optsBuffer=c,this.writeOpts(e);return}this.resources?.fieldA.destroy(),this.resources?.fieldB.destroy();let l=GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.TEXTURE_BINDING,u=e.createTexture({label:`duplicates-field-a-rgba16float`,size:[t,n],format:K,usage:l}),d=e.createTexture({label:`duplicates-field-b-rgba16float`,size:[t,n],format:K,usage:l}),f=u.createView(),p=d.createView(),m=e.createBindGroup({label:`duplicates-fusion-splat-bind`,layout:this.splatBindLayout,entries:[{binding:0,resource:{buffer:this.engine.paramsBuffer}},{binding:1,resource:{buffer:i}},{binding:2,resource:{buffer:a}},{binding:5,resource:{buffer:c}}]}),h=e.createBindGroup({label:`duplicates-field-blur-h-bind`,layout:this.blurBindLayout,entries:[{binding:0,resource:this.sampler},{binding:1,resource:f},{binding:2,resource:{buffer:o}}]}),g=e.createBindGroup({label:`duplicates-field-blur-v-bind`,layout:this.blurBindLayout,entries:[{binding:0,resource:this.sampler},{binding:1,resource:p},{binding:2,resource:{buffer:s}}]}),_=e.createBindGroup({label:`duplicates-membrane-bind`,layout:this.membraneBindLayout,entries:[{binding:0,resource:{buffer:this.engine.paramsBuffer}},{binding:3,resource:this.sampler},{binding:4,resource:f},{binding:5,resource:{buffer:c}}]});this.resources={cellBuffer:i,neckBuffer:a,blurHBuffer:o,blurVBuffer:s,optsBuffer:c,splatBindGroup:m,blurHBindGroup:h,blurVBindGroup:g,membraneBindGroup:_,fieldA:u,fieldB:d,fieldAView:f,fieldBView:p,fieldSize:[t,n]},this.writeOpts(e)}buildGeometry(){let e=this.scene?.clusters??[],t=Math.max(1,e.length),n=[],r=[],i=Array(e.length).fill(0),a=q;for(let t=0;t<e.length&&a>0;t++)i[t]=1,--a;for(let t=0;t<e.length&&a>0;t++)i[t]===1&&e[t].memories.length>=2&&(i[t]=2,--a);let o=!0;for(;a>0&&o;){o=!1;for(let t=0;t<e.length&&a>0;t++)i[t]>0&&i[t]<Math.min(e[t].memories.length,12)&&(i[t]+=1,--a,o=!0)}for(let a=0;a<e.length;a++){let o=e[a];if(i[a]===0)continue;let s=a/t*Math.PI*2-Math.PI/2,c=.18+.58*Math.sqrt((a+.5)/t),l=Math.cos(s)*c*.86,u=Math.sin(s)*c,d=Math.max(.04,.25-Math.max(0,o.similarity-o.threshold)*.55),f=o.memories.find(e=>e.id===o.winnerId)??o.memories[0],p=[f,...o.memories.filter(e=>e.id!==f.id)].slice(0,i[a]),m=Math.max(1,p.length),h=new Map;for(let e=0;e<p.length&&n.length<q;e++){let t=p[e],r=s+e/m*Math.PI*2+(m%2?0:Math.PI/m),i=t.id===o.winnerId,a=i?d*.18:d+e%3*.025,c=Math.min(1,(t.mismatchTokens?.length??0)/8),f=.085+Math.min(.045,L(t.retention)*2.1)+(i?.012:0),g={cluster:o,memoryId:t.id,x:l+Math.cos(r)*a,y:u+Math.sin(r)*a,retention:Math.max(0,Math.min(1,t.retention||0)),winner:i,mismatch:c,radius:f,memberSlot:e,memberCount:m};h.set(t.id,g),n.push(g)}let g=h.get(f.id);if(g)for(let e of o.memories){if(r.length>=J||e.id===f.id)continue;let t=h.get(e.id);t&&r.push({cluster:o,winnerId:f.id,candidateId:e.id,ax:g.x,ay:g.y,bx:t.x,by:t.y,winnerRetention:g.retention,candidateRetention:t.retention,winnerRadius:g.radius,candidateRadius:t.radius,mismatch:Math.max(t.mismatch,Math.min(1,o.mismatchTokens.length/12))})}}this.cellGeometry=n,this.neckGeometry=r}uploadBuffers(e){if(!this.resources)return;let t=new Float32Array(q*Y),n=new Float32Array(J*Se);this.cellCount=Math.min(q,this.cellGeometry.length),this.neckCount=Math.min(J,this.neckGeometry.length);for(let e=0;e<this.cellCount;e++){let n=this.cellGeometry[e];t.set([n.x,n.y,n.retention,+!!n.winner,n.cluster.similarity,n.cluster.threshold,n.memberSlot,n.cluster.index,n.mismatch,+(n.cluster.suggestedAction===`merge`),n.radius,n.memberCount,e,n.cluster.index,0,0],e*Y)}for(let e=0;e<this.neckCount;e++){let t=this.neckGeometry[e];n.set([t.ax,t.ay,t.winnerRetention,t.winnerRadius,t.bx,t.by,t.candidateRetention,t.candidateRadius,t.cluster.similarity,t.cluster.threshold,t.mismatch,+(t.cluster.suggestedAction===`merge`),e,t.cluster.index,0,0],e*Se)}this.engine.params[2]=this.cellCount,this.engine.params[3]=this.neckCount,this.engine.params[4]=this.neckCount,e.queue.writeBuffer(this.resources.cellBuffer,0,t),e.queue.writeBuffer(this.resources.neckBuffer,0,n)}compute(e){let t=this.engine.gpuDevice;if(!t||!this.resources||!this.splatPipeline||!this.blurPipeline)return;this.ensureResources(t);let n=this.resources,r=e.beginRenderPass({label:`duplicates-field-splat-pass`,colorAttachments:[{view:n.fieldAView,clearValue:{r:0,g:0,b:0,a:0},loadOp:`clear`,storeOp:`store`}]});r.setPipeline(this.splatPipeline),r.setBindGroup(0,n.splatBindGroup),this.cellCount+this.neckCount>0&&r.draw(6,this.cellCount+this.neckCount),r.end();let i=e.beginRenderPass({label:`duplicates-field-blur-h-pass`,colorAttachments:[{view:n.fieldBView,clearValue:{r:0,g:0,b:0,a:0},loadOp:`clear`,storeOp:`store`}]});i.setPipeline(this.blurPipeline),i.setBindGroup(0,n.blurHBindGroup),i.draw(6,1),i.end();let a=e.beginRenderPass({label:`duplicates-field-blur-v-pass`,colorAttachments:[{view:n.fieldAView,clearValue:{r:0,g:0,b:0,a:0},loadOp:`clear`,storeOp:`store`}]});a.setPipeline(this.blurPipeline),a.setBindGroup(0,n.blurVBindGroup),a.draw(6,1),a.end()}render(e){this.resources&&this.membranePipeline&&this.cellPipeline&&this.neckPipeline&&(e.setPipeline(this.membranePipeline),e.setBindGroup(0,this.resources.membraneBindGroup),e.draw(6,1),this.neckCount>0&&(e.setPipeline(this.neckPipeline),e.setBindGroup(0,this.resources.splatBindGroup),e.draw(64,this.neckCount)),this.cellCount>0&&(e.setPipeline(this.cellPipeline),e.setBindGroup(0,this.resources.splatBindGroup),e.draw(6,this.cellCount)))}pickAt(e,t){for(let n=0;n<this.neckGeometry.length;n++){let r=this.neckGeometry[n],i=Oe(e,t,r.ax,r.ay,r.bx,r.by),a=(r.ax+r.bx)*.5,o=(r.ay+r.by)*.5,s=.055+Math.max(0,r.cluster.similarity-r.cluster.threshold)*.45;if(i<=s||Math.hypot(e-a,t-o)<=s)return{id:r.cluster.id,kind:`duplicate-neck`,index:n,payload:r.cluster}}for(let n=0;n<this.cellGeometry.length;n++){let r=this.cellGeometry[n];if(Math.hypot(e-r.x,t-r.y)<=r.radius*.8)return{id:r.memoryId,kind:`duplicate-memory`,index:n,payload:r.cluster}}return null}dispose(){this.resources?.cellBuffer.destroy(),this.resources?.neckBuffer.destroy(),this.resources?.blurHBuffer.destroy(),this.resources?.blurVBuffer.destroy(),this.resources?.optsBuffer.destroy(),this.resources?.fieldA.destroy(),this.resources?.fieldB.destroy(),this.resources=null}};function Oe(e,t,n,r,i,a){let o=i-n,s=a-r,c=e-n,l=t-r,u=o*c+s*l;if(u<=0)return Math.hypot(e-n,t-r);let d=o*o+s*s;if(d<=u)return Math.hypot(e-i,t-a);let f=u/d;return Math.hypot(e-(n+f*o),t-(r+f*s))}function X(e,t,n){e.pushErrorScope(`validation`);let r=e.createShaderModule({label:t,code:n});return r.getCompilationInfo().then(e=>{for(let n of e.messages)console.error(`[observatory] ${t} WGSL ${n.type} ${n.lineNum}:${n.linePos} ${n.message}`)}),e.popErrorScope().then(e=>{e&&console.error(`[observatory] ${t} shader module validation: ${e.message}`)}),r}function Z(e,t){B(I.blackwater),B(z.recall),B(z.luciferin),B(R.trustMembrane);let n=new De(e,t);return n.setIntensity(.22),n.setReadingWell({x:0,y:0,hw:.6,hh:.85,floor:.08,soft:.25}),[n]}function Q(e){return Math.max(0,Math.min(1,Number.isFinite(e)?e:0))}function $(e,t,n){return n?{kind:e,id:t,scalar:n}:{kind:e,id:t||`${e}:unknown`}}function ke(e,t){return{kind:`scalar`,id:`duplicates.${e}`,scalar:{name:e,value:t}}}function Ae(e,t=84){let n=(e||``).trim().replace(/\s+/g,` `);return n.length<=t?n:`${n.slice(0,t)}…`}function je(e){return(e||``).toLowerCase().replace(/[^a-z0-9_\s-]/g,` `).split(/\s+/).filter(e=>e.length>=4).slice(0,80)}function Me(e){if(e.length<2)return[];let t=e.map(e=>new Set(je(e.content))),n=new Map;for(let e of t)for(let t of e)n.set(t,(n.get(t)??0)+1);return Array.from(n.entries()).filter(([,t])=>t>0&&t<e.length).sort((e,t)=>t[1]-e[1]||e[0].localeCompare(t[0])).slice(0,12).map(([e])=>e)}function Ne(e,t,n){let r=Array.isArray(e.memories)?e.memories.filter(e=>e.id):[];if(r.length<2)return null;let i=G(r),a=ae(r),o=Me(r);return{id:i,index:t,similarity:Q(e.similarity),threshold:Q(n),suggestedAction:e.suggestedAction===`merge`?`merge`:`review`,winnerId:a?.id??r[0].id,memories:r.map((e,t)=>({...e,index:t,preview:Ae(e.content),winner:e.id===(a?.id??r[0].id),mismatchTokens:o.filter(t=>je(e.content).includes(t)).slice(0,8)})),mismatchTokens:o,source:$(`pair`,i)}}function Pe(e){let t=Q(e.threshold??.8),n=(Array.isArray(e.clusters)?e.clusters:[]).map((e,n)=>Ne(e,n,t)).filter(e=>e!==null),r=0,i=[],a=new Map;for(let e of n)for(let t of e.memories){if(a.has(t.id))continue;let n=r++;a.set(t.id,n),i.push({source:$(`memory`,t.id),index:n,label:t.preview||t.id.slice(0,8),retention:Q(t.retention),trust:Q(e.similarity),lastAccessed:t.createdAt,tags:[t.nodeType,...t.tags,t.winner?`winner`:`candidate`].filter(Boolean),type:t.nodeType||`memory`})}let o=[];for(let e of n){let t=a.get(e.winnerId);if(t!=null)for(let n of e.memories){let r=a.get(n.id);r!=null&&r!==t&&o.push({source:$(`pair`,`${e.id}:${e.winnerId}:${n.id}`),sourceIndex:t,targetIndex:r,weight:Math.max(.05,e.similarity),kind:e.suggestedAction===`merge`?`fusion-candidate`:`review-candidate`})}}let s=n.map((e,n)=>({source:$(`event`,`duplicates.cluster.${e.id}`),type:e.suggestedAction===`merge`?`DuplicateMergeCandidate`:`DuplicateReviewCandidate`,targetIndex:-1,frame:20+n*14,energy:Math.max(.1,e.similarity-t+.1)})),c=Number.isFinite(e.total)?e.total:n.length,l=i.length,u=n.reduce((e,t)=>Math.max(e,t.similarity),0),d=n.filter(e=>e.suggestedAction===`merge`).length,f=n.length-d;return{organ:`duplicates`,nodes:i,edges:o,events:s,receipts:[],scalars:{threshold:ke(`threshold`,t).scalar?.value??t,clusterCount:n.length,memoryCount:l,maxSimilarity:u,mergeCandidates:d,reviewCandidates:f,total:c},alive:n.length>0,threshold:t,total:c,clusters:n,raw:e}}var Fe=()=>typeof window<`u`&&window.matchMedia?.(`(prefers-reduced-motion: reduce)`).matches;function Ie(e){if(Fe())return{};let t=0;function n(n){let r=e.getBoundingClientRect();cancelAnimationFrame(t),t=requestAnimationFrame(()=>{e.style.setProperty(`--spot-x`,`${n.clientX-r.left}px`),e.style.setProperty(`--spot-y`,`${n.clientY-r.top}px`),e.style.setProperty(`--spot-o`,`1`)})}function r(){e.style.setProperty(`--spot-o`,`0`)}return e.addEventListener(`pointermove`,n),e.addEventListener(`pointerleave`,r),{destroy(){e.removeEventListener(`pointermove`,n),e.removeEventListener(`pointerleave`,r),cancelAnimationFrame(t)}}}var Le=a(`<span class="ping-host flex h-2 w-2 items-center justify-center text-synapse-glow" aria-hidden="true"><span class="breathe h-2 w-2 rounded-full bg-synapse-glow"></span></span>`),Re=a(`<!> <span class="text-xs text-dim"> </span>`,1),ze=a(`<label class="flex w-full flex-col gap-2 text-xs text-dim"><span class="flex items-baseline justify-between gap-3"><span class="whitespace-nowrap"> </span> <span class="font-mono text-sm text-bright"> </span></span> <input type="range" min="0.70" max="0.95" step="0.01" class="w-full accent-synapse"/></label>`),Be=a(`<label class="flex flex-1 min-w-64 items-center gap-3 text-xs text-dim"><span class="whitespace-nowrap"> </span> <input type="range" min="0.70" max="0.95" step="0.01" class="flex-1 accent-synapse"/> <span class="w-14 text-right font-mono text-sm text-bright"> </span></label>`),Ve=a(`<span class="breathe h-2 w-2 rounded-full bg-synapse-glow text-synapse-glow"></span> <span> </span>`,1),He=a(`<span class="h-2 w-2 rounded-full bg-decay"></span> <span class="text-decay"> </span>`,1),Ue=a(`<!> `,1),We=a(`<span class="breathe h-2 w-2 rounded-full bg-synapse-glow text-synapse-glow"></span> <span class="tabular-nums"><!> · <!> </span>`,1),Ge=a(`<div class="flex items-center gap-2 rounded-full border border-synapse/20 bg-synapse/10 px-3 py-1.5 text-xs text-text" role="status" aria-live="polite"><!></div> <button type="button" class="rounded-lg bg-white/[0.04] px-3 py-1.5 text-xs text-dim transition hover:bg-white/[0.08] hover:text-text disabled:opacity-40 focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60"> </button>`,1),Ke=a(`<div class="glass-panel pointer-events-auto rounded-2xl border border-synapse/25 bg-black/30 p-4"><div class="flex flex-wrap items-center justify-between gap-3"><div><div class="font-mono text-[11px] uppercase tracking-[0.18em] text-synapse-glow"> </div> <div class="mt-1 text-sm text-bright"> </div> <div class="mt-1 max-w-2xl text-xs text-muted"> </div></div> <button type="button" class="rounded-lg bg-white/[0.04] px-3 py-1.5 text-xs text-dim transition hover:bg-white/[0.08] hover:text-text focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60"> </button></div></div>`),qe=a(`<div class="glass-panel pointer-events-auto flex flex-col items-center gap-3 rounded-2xl p-10 text-center"><div class="text-sm text-decay"> </div> <div class="max-w-md text-xs text-muted"> </div> <button type="button" class="mt-2 rounded-lg bg-synapse/20 px-4 py-2 text-xs font-medium text-synapse-glow transition hover:bg-synapse/30 focus:outline-none focus-visible:ring-2 focus-visible:ring-synapse/60"> </button></div>`),Je=a(`<div class="glass-subtle shimmer h-40 rounded-2xl"></div>`),Ye=a(`<div class="pointer-events-auto space-y-3"></div>`),Xe=a(`<div class="glass-panel pointer-events-auto enter flex flex-col items-center gap-3 rounded-2xl p-12 text-center"><div class="flex h-14 w-14 items-center justify-center rounded-2xl border border-recall/25 bg-recall/10 text-recall"><!></div> <div class="text-sm font-medium text-bright"> </div> <div class="max-w-sm text-xs text-muted"> </div></div>`),Ze=a(`<div class="glass-subtle rounded-xl border border-warning/30 bg-warning/5 px-4 py-2 text-xs text-dim"> </div>`),Qe=a(`<div class="spotlight-surface lift rounded-2xl"><div class="relative z-[1]"><!></div></div>`),$e=a(`<div class="pointer-events-auto space-y-4"><!> <!></div>`),et=a(`<!> <div class="relative z-10 mx-auto max-h-dvh max-w-5xl space-y-6 overflow-y-auto overscroll-contain p-6 pb-28 pointer-events-none"><!> <div class="glass-panel pointer-events-auto flex flex-wrap items-center gap-5 rounded-2xl p-4"><!> <!></div> <!> <!></div>`,1);function tt(r,a){v(a,!0);let m=O(.8),h=O(e([])),S=O(0),w=O(e(new Set)),j=O(!0),M=O(null),P=O(null),I,L=O(!1);D(()=>{let e=()=>{y(L,window.innerWidth/Math.max(1,window.innerHeight)<.85)};return e(),window.addEventListener(`resize`,e),()=>window.removeEventListener(`resize`,e)});async function R(){y(j,!0),y(M,null),y(P,null);try{let e=await F.duplicates(c(m));y(h,e.clusters,!0),y(S,e.total??e.clusters.length,!0);let t=new Set(c(h).map(e=>G(e.memories))),n=new Set;for(let e of c(w))t.has(e)&&n.add(e);y(w,n,!0)}catch(e){y(M,e instanceof Error?e.message:N(`Failed to detect duplicates`),!0),y(h,[],!0)}finally{y(j,!1)}}function z(){clearTimeout(I),I=setTimeout(R,250)}function B(e){let t=new Set(c(w));t.add(e),y(w,t,!0),c(P)&&G(c(P).memories)===e&&y(P,null)}function U(e){B(e),R()}let W=E(()=>c(h).map(e=>({c:e,key:G(e.memories)})).filter(({key:e})=>!c(w).has(e))),re=E(()=>c(h).reduce((e,t)=>e+t.memories.length,0)),ie=E(()=>c(W).length>50),ae=E(()=>c(ie)?c(W).slice(0,50):c(W)),oe=E(()=>Pe({threshold:c(m),total:c(W).length,clusters:c(W).map(({c:e})=>e)}));function se(e){(e.kind===`duplicate-neck`||e.kind===`duplicate-memory`)&&y(P,e.payload,!0)}D(()=>R()),T(()=>clearTimeout(I));var ce=et(),le=d(ce);{let e=E(()=>`synaptic-fusion:${c(m)}:${c(W).length}:${c(re)}`),t=E(()=>`NO DUPLICATES ABOVE ${(c(m)*100).toFixed(0)}% SIMILARITY`);V(le,{organ:`duplicates`,get seed(){return c(e)},get scene(){return c(oe)},get passes(){return Z},get loading(){return c(j)},get error(){return c(M)},get emptyLabel(){return c(t)},onpick:se})}var ue=s(le,2),de=f(ue);{let e=E(()=>N(`Memory Hygiene: Duplicate Detection`)),n=E(()=>N(`Cosine-similarity clustering over embeddings. Merge previews a reversible plan and applies it only on your say-so; dedup undo reverses it. Oversized similarity components are quarantined for review because they chain through pairwise similarity and are not safe to merge. Dismissed clusters are hidden for this session only.`));te(de,{icon:`duplicates`,get title(){return c(e)},get subtitle(){return c(n)},accent:`synapse`,children:(e,n)=>{var r=Re(),i=d(r),a=e=>{var n=Le();t(e,n)};o(i,e=>{c(M)||e(a)});var l=s(i,2),f=p(l,!0);u(e=>C(f,e),[()=>c(M)?N(`Offline`):c(j)?N(`Refreshing`):N(`Live`)]),t(e,r)},$$slots:{default:!0}})}var fe=s(de,2),pe=f(fe),me=e=>{var n=ze(),r=f(n),a=f(r),o=p(a,!0),l=s(a,2),d=p(l);x(r);var h=s(r,2);_(h),x(n),u((e,t,n)=>{C(o,e),C(d,`${t??``}%`),b(h,`aria-label`,n)},[()=>N(`Similarity threshold`),()=>(c(m)*100).toFixed(0),()=>N(`Similarity threshold`)]),i(`input`,h,z),k(h,()=>c(m),e=>y(m,e)),t(e,n)},he=e=>{var n=Be(),r=f(n),a=p(r,!0),o=s(r,2);_(o);var l=s(o,2),d=p(l);x(n),u((e,t,n)=>{C(a,e),b(o,`aria-label`,t),C(d,`${n??``}%`)},[()=>N(`Similarity threshold`),()=>N(`Similarity threshold`),()=>(c(m)*100).toFixed(0)]),i(`input`,o,z),k(o,()=>c(m),e=>y(m,e)),t(e,n)};o(pe,e=>{c(L)?e(me):e(he,-1)});var ge=s(pe,2),_e=e=>{var n=Ge(),r=d(n),a=f(r),l=e=>{var n=Ve(),r=s(d(n),2),i=p(r,!0);u(e=>C(i,e),[()=>N(`Detecting…`)]),t(e,n)},m=e=>{var n=He(),r=s(d(n),2),i=p(r,!0);u(e=>C(i,e),[()=>N(`Error`)]),t(e,n)},h=e=>{var n=We(),r=s(d(n),2),i=f(r),a=e=>{var n=Ue(),r=d(n);H(r,{get value(){return c(W).length}});var i=s(r);u((e,t)=>C(i,` ${e??``} ${c(S)??``} ${t??``}`),[()=>N(`visible of`),()=>N(`clusters`)]),t(e,n)},l=e=>{var n=Ue(),r=d(n);H(r,{get value(){return c(W).length}});var i=s(r);u(e=>C(i,` ${e??``}`),[()=>c(W).length===1?N(`cluster`):N(`clusters`)]),t(e,n)};o(i,e=>{c(W).length<c(S)?e(a):e(l,-1)});var p=s(i,2);H(p,{get value(){return c(re)}});var m=s(p);x(r),u(e=>C(m,` ${e??``}`),[()=>N(`memories implicated`)]),t(e,n)};o(a,e=>{c(j)?e(l):c(M)?e(m,1):e(h,-1)}),x(r);var g=s(r,2),_=p(g,!0);u(e=>{g.disabled=c(j),C(_,e)},[()=>N(`Rerun`)]),i(`click`,g,R),t(e,n)};o(ge,e=>{c(M)&&c(L)||e(_e)}),x(fe);var ve=s(fe,2),ye=e=>{var n=Ke(),r=f(n),a=f(r),o=f(a),l=p(o,!0),d=s(o,2),m=p(d),h=s(d,2),g=p(h);x(a);var _=s(a,2),v=p(_,!0);x(r),x(n),u((e,t,n,r,i,a,o,s,u)=>{C(l,e),C(m,`${c(P).memories.length??``} ${t??``} ${n??``}${r??``} ${i??``}`),C(g,`${a??``} ${c(P).id??``}${o??``} ${s??``}.`),C(v,u)},[()=>N(`Synaptic neck selected`),()=>N(`memories ·`),()=>(c(P).similarity*100).toFixed(1),()=>N(`% similar · winner`),()=>c(P).winnerId.slice(0,8),()=>N(`Real pair key:`),()=>N(`. Mismatch filaments:`),()=>c(P).mismatchTokens.length?c(P).mismatchTokens.join(`, `):N(`none exposed`),()=>N(`Clear field focus`)]),i(`click`,_,()=>y(P,null)),t(e,n)};o(ve,e=>{c(P)&&e(ye)});var be=s(ve,2),K=e=>{var n=qe(),r=f(n),a=p(r,!0),o=s(r,2),l=p(o,!0),d=s(o,2),m=p(d,!0);x(n),u((e,t)=>{C(a,e),C(l,c(M)),C(m,t)},[()=>N(`Couldn't detect duplicates`),()=>N(`Retry`)]),i(`click`,d,R),t(e,n)},q=e=>{var r=Ye();l(r,20,()=>[,,,],n,(e,n)=>{var r=Je();t(e,r)}),x(r),t(e,r)},J=e=>{var n=Xe(),r=f(n),i=f(r);ee(i,{name:`sparkle`,size:26,draw:!0}),x(r);var a=s(r,2),o=p(a,!0),l=s(a,2),d=p(l);x(n),u((e,t,n,r)=>{C(o,e),C(d,`${t??``} ${n??``}${r??``}`)},[()=>N(`No duplicates found — your memory is clean.`),()=>N(`Nothing clusters above`),()=>(c(m)*100).toFixed(0),()=>N(`% similarity. Lower the threshold to surface looser matches.`)]),t(e,n)},Y=e=>{var n=$e(),r=f(n),i=e=>{var n=Ze(),r=p(n);u((e,t,n)=>C(r,`${e??``} 50 ${t??``} ${c(W).length??``} ${n??``}`),[()=>N(`Showing first`),()=>N(`of`),()=>N(`clusters. Raise the threshold to narrow results.`)]),t(e,n)};o(r,e=>{c(ie)&&e(i)});var a=s(r,2);l(a,19,()=>c(ae),({c:e,key:t})=>t,(e,n,r)=>{let i=()=>c(n).c,a=()=>c(n).key;var o=Qe(),s=f(o),l=f(s);{let e=E(()=>i().memories.length>12);xe(l,{get similarity(){return i().similarity},get memories(){return i().memories},get suggestedAction(){return i().suggestedAction},get oversized(){return c(e)},onDismiss:()=>B(a()),onPlan:e=>F.duplicatesPlan(e),onApply:e=>F.duplicatesApply(e),onMerged:()=>U(a())})}x(s),x(o),g(o,(e,t)=>ne?.(e,t),()=>({delay:Math.min(c(r)*40,400),y:14})),g(o,e=>Ie?.(e)),t(e,o)}),x(n),t(e,n)};o(be,e=>{c(M)?e(K):c(j)?e(q,1):c(W).length===0?e(J,2):e(Y,-1)}),x(ue),t(r,ce),A()}r([`input`,`click`]);export{tt as component};
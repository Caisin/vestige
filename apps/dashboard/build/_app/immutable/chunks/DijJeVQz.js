import{A as e,M as t,O as n,R as r,W as i,X as a,_ as o,a as s,c,dt as l,et as u,j as d,r as f,tt as p,ut as m}from"./BAUNE5Zx.js";import"./xihTtKlq.js";var h=`
struct AmbientParams {
	time: f32,          // seconds (advances only when not reduced-motion)
	count: f32,         // active mote count (<= capacity)
	endangered: f32,    // 0..1 real endangered fraction — storm intensity
	fracture: f32,      // 0..1 real contradiction fraction — rift intensity
	due: f32,           // 0..1 real due-for-review fraction — pulse rate
	aspect: f32,        // viewport w/h
	accent_r: f32,      // route accent (rgb, 0..1) — one accent per §guardrail
	accent_g: f32,
	accent_b: f32,
	dpr: f32,
	reduced: f32,       // 1.0 = prefers-reduced-motion (freeze drift, keep field)
	_pad: f32,
};

@group(0) @binding(0) var<uniform> params: AmbientParams;

struct VSOut {
	@builtin(position) clip: vec4<f32>,
	@location(0) uv: vec2<f32>,
	@location(1) @interpolate(flat) seed: vec2<f32>,
	@location(2) @interpolate(flat) tint: vec4<f32>, // rgb + retention
};

// Golden-ratio hash → a deterministic 0..1 per index (no PRNG state).
fn hash1(n: f32) -> f32 {
	return fract(sin(n * 12.9898) * 43758.5453);
}

const CORNERS = array<vec2<f32>, 6>(
	vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
	vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0)
);

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VSOut {
	var out: VSOut;
	if (ii >= u32(params.count)) {
		out.clip = vec4<f32>(0.0, 0.0, 2.0, 1.0);
		return out;
	}
	let fi = f32(ii);
	// Deterministic base position on a golden-angle lattice across the panel.
	let ga = 2.399963; // golden angle
	let rx = hash1(fi + 1.0);
	let ry = hash1(fi + 37.0);
	// Retention bucket for this mote: bias the population so the endangered
	// share of motes are low-retention (they sink). Real fraction drives it.
	let isEndangered = select(0.0, 1.0, ry < params.endangered);
	let retention = mix(0.55 + 0.4 * hash1(fi + 91.0), 0.05 + 0.18 * hash1(fi + 5.0), isEndangered);

	// Vertical home: high retention floats up, low sinks toward the floor.
	let homeY = mix(-0.9, 0.85, retention);
	// Gentle deterministic drift (frozen when reduced-motion): endangered motes
	// jitter more (the field is agitated by how much is being forgotten).
	let t = params.time;
	let sway = select(1.0, 0.0, params.reduced > 0.5);
	let turb = 0.02 + 0.10 * params.endangered;
	let driftX = sway * turb * sin(t * (0.3 + rx) + fi * ga);
	let driftY = sway * (0.015 + 0.05 * isEndangered) * sin(t * (0.5 + ry) + fi);
	let baseX = (rx * 2.0 - 1.0) * 0.98 + driftX;
	let baseY = homeY + driftY;

	// A rift: the fracture metric opens a horizontal tear that pushes motes apart.
	let rift = params.fracture * 0.25 * sin(baseX * 3.14159 + t * 0.2);
	let center = vec2<f32>(baseX, baseY + rift);

	// Mote size: small; endangered ones a touch larger + dimmer (last flare).
	let size = (0.010 + 0.014 * retention) * (1.0 + 0.4 * isEndangered);
	let corner = CORNERS[vi];
	out.clip = vec4<f32>(center.x + corner.x * size, center.y + corner.y * size * params.aspect, 0.0, 1.0);
	out.uv = corner;
	out.seed = vec2<f32>(rx, ry);
	out.tint = vec4<f32>(params.accent_r, params.accent_g, params.accent_b, retention);
	return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
	let d = length(in.uv);
	if (d > 1.0) { discard; }
	let retention = in.tint.a;
	// Soft mote: hot core + feathered halo, brightness scales with retention so
	// the endangered (dim) vs healthy (bright) split is LEGIBLE at a glance.
	let core = smoothstep(0.5, 0.0, d);
	let halo = pow(max(1.0 - d, 0.0), 2.0);
	// Due-for-review adds a slow global pulse so an overdue route breathes.
	let pulse = 0.85 + 0.15 * sin(params.time * (0.6 + params.due));
	let intensity = (core * 0.9 + halo * 0.35) * (0.25 + 0.75 * retention) * pulse;
	// Endangered motes shift toward a warmer, dimmer ember; healthy toward accent.
	let ember = vec3<f32>(0.62, 0.32, 0.22);
	let col = mix(ember, in.tint.rgb, smoothstep(0.2, 0.6, retention));
	return vec4<f32>(col * intensity, intensity * 0.9);
}
`,g=t(`<canvas class="pointer-events-none absolute inset-0 h-full w-full" aria-hidden="true"></canvas>`);function _(t,_){l(_,!0);let v=s(_,`endangered`,3,0),y=s(_,`fracture`,3,0),b=s(_,`due`,3,0),x=s(_,`count`,3,0),S=s(_,`accent`,19,()=>[.13,.78,.87]),C=s(_,`opacity`,3,.5),w=p(null),T=p(!0);f(()=>{if(!r(w))return;let e=null,t=null,n=null,i=null,a=null,o=0,s=!1,c=!0,l=!0,d=0,f=0,p=window.matchMedia(`(prefers-reduced-motion: reduce)`),m=new Float32Array(12),g=()=>Math.min(x()>0?x():520,window.innerWidth<640?220:520);function _(e){let t=Math.min(window.devicePixelRatio||1,window.innerWidth<640?2:1.5),n=Math.max(1,Math.floor((r(w).clientWidth||1)*t)),i=Math.max(1,Math.floor((r(w).clientHeight||1)*t));(r(w).width!==n||r(w).height!==i)&&(r(w).width=n,r(w).height=i),m[0]=d,m[1]=g(),m[2]=Math.max(0,Math.min(1,v())),m[3]=Math.max(0,Math.min(1,y())),m[4]=Math.max(0,Math.min(1,b())),m[5]=n/Math.max(1,i),m[6]=S()[0],m[7]=S()[1],m[8]=S()[2],m[9]=t,m[10]=+!!p.matches,m[11]=0}async function C(){let c=navigator.gpu;if(!c){u(T,!1);return}let l=null;try{l=await c.requestAdapter()}catch{u(T,!1);return}if(!l||s){u(T,!1);return}try{e=await l.requestDevice()}catch{u(T,!1);return}if(s){e?.destroy();return}let d=r(w).getContext(`webgpu`);if(!d){u(T,!1);return}t=d;let p=c.getPreferredCanvasFormat();t.configure({device:e,format:p,alphaMode:`premultiplied`}),a=e.createBuffer({label:`ambient-params`,size:m.byteLength,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST});let g=e.createShaderModule({label:`ambient-field`,code:h});n=e.createRenderPipeline({label:`ambient-field`,layout:`auto`,vertex:{module:g,entryPoint:`vs_main`},fragment:{module:g,entryPoint:`fs_main`,targets:[{format:p,blend:{color:{srcFactor:`src-alpha`,dstFactor:`one`,operation:`add`},alpha:{srcFactor:`one`,dstFactor:`one`,operation:`add`}}}]},primitive:{topology:`triangle-list`}}),i=e.createBindGroup({label:`ambient-bind`,layout:n.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:a}}]}),f=0,o=requestAnimationFrame(E)}function E(r){if(s||!e||!t||!n||!i||!a)return;if(!c||!l){o=requestAnimationFrame(E);return}f>0&&!p.matches&&(d+=Math.min(r-f,100)/1e3),f=r,_(r),e.queue.writeBuffer(a,0,m);let u;try{u=t.getCurrentTexture().createView()}catch{o=requestAnimationFrame(E);return}let h=e.createCommandEncoder({label:`ambient-frame`}),v=h.beginRenderPass({colorAttachments:[{view:u,clearValue:{r:0,g:0,b:0,a:0},loadOp:`clear`,storeOp:`store`}]});v.setPipeline(n),v.setBindGroup(0,i),v.draw(6,Math.floor(g())),v.end(),e.queue.submit([h.finish()]),o=requestAnimationFrame(E)}let D=()=>{c=document.visibilityState===`visible`};document.addEventListener(`visibilitychange`,D);let O=new IntersectionObserver(e=>{l=e.some(e=>e.isIntersecting)},{threshold:0});return O.observe(r(w)),C(),()=>{s=!0,cancelAnimationFrame(o),document.removeEventListener(`visibilitychange`,D),O.disconnect(),a?.destroy(),e?.destroy()}});var E=d(),D=a(E),O=t=>{var n=g();c(n,e=>u(w,e),()=>r(w)),i(()=>o(n,`opacity: ${C()??``}`)),e(t,n)};n(D,e=>{r(T)&&e(O)}),e(t,E),m()}export{_ as t};
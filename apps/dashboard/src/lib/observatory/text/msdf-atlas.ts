import { base } from '$app/paths';
import type { MsdfGlyph } from './layout';
import { alphaToSdf } from './unicode-sdf';

export type MsdfAtlasMetrics = {
	emSize?: number;
	lineHeight: number;
	ascender?: number;
	descender?: number;
	underlineY?: number;
	underlineThickness?: number;
};

export type LoadedMsdfAtlas = {
	atlas: {
		type?: string;
		distanceRange: number;
		distanceRangeMiddle?: number;
		size: number;
		width: number;
		height: number;
		yOrigin?: string;
	};
	metrics: MsdfAtlasMetrics;
	glyphs: MsdfGlyph[];
	glyphMap: Map<number, MsdfGlyph>;
	texture: GPUTexture;
	textureView: GPUTextureView;
	sampler: GPUSampler;
	dispose: () => void;
	ensureGlyphs: (text: string) => void;
};

type RawAtlas = Omit<LoadedMsdfAtlas, 'glyphMap' | 'texture' | 'textureView' | 'sampler' | 'dispose' | 'ensureGlyphs'>;

const atlasCache = new WeakMap<GPUDevice, { promise: Promise<LoadedMsdfAtlas>; references: number }>();

export async function loadMsdfAtlas(device: GPUDevice): Promise<LoadedMsdfAtlas> {
	let cached = atlasCache.get(device);
	if (!cached) { cached = { promise: createAtlas(device), references: 0 }; atlasCache.set(device, cached); }
	cached.references++;
	try {
		const atlas = await cached.promise;
		let disposed = false; const entry = cached;
		return { ...atlas, dispose() { if (disposed) return; disposed = true; if (--entry.references === 0) { atlas.dispose(); atlasCache.delete(device); } } };
	} catch (error) { cached.references--; atlasCache.delete(device); throw error; }
}

async function createAtlas(device: GPUDevice): Promise<LoadedMsdfAtlas> {
	const jsonUrl = `${base}/msdf/jetbrains-mono.json`;
	const pngUrl = `${base}/msdf/jetbrains-mono.png`;

	const jsonResponse = await fetch(jsonUrl);
	if (!jsonResponse.ok) throw new Error(`MSDF atlas JSON failed: ${jsonResponse.status} ${jsonUrl}`);
	const raw = (await jsonResponse.json()) as RawAtlas;
	if (raw.atlas?.yOrigin !== 'bottom') {
		throw new Error(`MSDF atlas yOrigin must be bottom, got ${raw.atlas?.yOrigin ?? 'missing'}`);
	}

	const pngResponse = await fetch(pngUrl);
	if (!pngResponse.ok) throw new Error(`MSDF atlas PNG failed: ${pngResponse.status} ${pngUrl}`);
	const blob = await pngResponse.blob();
	const bitmap = await createImageBitmap(blob);
	const dimension = Math.min(4096, device.limits.maxTextureDimension2D);
	const originalHeight = bitmap.height;
	const shift = dimension - raw.atlas.height;
	for (const glyph of raw.glyphs) { if (glyph.atlasBounds) { glyph.atlasBounds.bottom += shift; glyph.atlasBounds.top += shift; } }
	raw.atlas.width = dimension; raw.atlas.height = dimension;
	const texture = device.createTexture({
		label: 'msdf-jetbrains-mono-rgba8unorm',
		size: [dimension, dimension, 1],
		format: 'rgba8unorm',
		usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT
	});
	device.queue.copyExternalImageToTexture(
		{ source: bitmap },
		{ texture },
		{ width: bitmap.width, height: bitmap.height }
	);
	bitmap.close?.();

	const sampler = device.createSampler({
		label: 'msdf-jetbrains-mono-linear-sampler',
		magFilter: 'linear',
		minFilter: 'linear',
		mipmapFilter: 'linear',
		addressModeU: 'clamp-to-edge',
		addressModeV: 'clamp-to-edge'
	});
	const textureView = texture.createView({ label: 'msdf-jetbrains-mono-view' });
	const glyphMap = new Map(raw.glyphs.map((glyph) => [glyph.unicode, glyph]));
	const tile = 64, fontSize = 48, padding = 8, baseline = 49;
	const columns = Math.floor(dimension / tile);
	let slot = Math.ceil(originalHeight / tile) * columns;
	const canvas = document.createElement('canvas'); canvas.width = tile; canvas.height = tile;
	const context = canvas.getContext('2d', { willReadFrequently: true });
	const ensureGlyphs = (text: string) => {
		if (!context) return;
		for (const char of Array.from(text)) {
			const unicode = char.codePointAt(0)!;
			if (unicode < 32 || glyphMap.has(unicode) || slot >= columns * Math.floor(dimension / tile)) continue;
			context.clearRect(0, 0, tile, tile); context.font = `${fontSize}px "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif`; context.textBaseline = 'alphabetic'; context.fillStyle = '#fff'; context.fillText(char, padding, baseline);
			const x = (slot % columns) * tile, y = Math.floor(slot / columns) * tile; slot++;
			const image = alphaToSdf(context.getImageData(0, 0, tile, tile).data, tile, tile);
			device.queue.writeTexture({ texture, origin: [x, y] }, image, { bytesPerRow: tile * 4 }, { width: tile, height: tile });
			const glyph: MsdfGlyph = { unicode, advance: Math.max(0.4, context.measureText(char).width / fontSize), planeBounds: { left: -padding / fontSize, right: (tile - padding) / fontSize, bottom: (baseline - tile) / fontSize, top: baseline / fontSize }, atlasBounds: { left: x, right: x + tile, top: dimension - y, bottom: dimension - y - tile } };
			raw.glyphs.push(glyph); glyphMap.set(unicode, glyph);
		}
	};
	return {
		...raw,
		glyphMap,
		texture,
		textureView,
		sampler,
		ensureGlyphs,
		dispose: () => texture.destroy()
	};
}

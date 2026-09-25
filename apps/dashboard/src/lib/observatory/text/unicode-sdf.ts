/** Local system-font fallback for glyphs missing from the bundled Latin MSDF.
 * Two-pass chamfer distance fields keep work linear in the glyph tile size.
 * No font download, remote text rendering or user-content translation occurs.
 */
export function alphaToSdf(pixels: Uint8ClampedArray, width: number, height: number, range = 4): Uint8Array {
    const count = width * height;
    const inside = new Float32Array(count); const outside = new Float32Array(count);
    for (let i = 0; i < count; i++) { const filled = pixels[i * 4 + 3] >= 128; inside[i] = filled ? 0 : 1e6; outside[i] = filled ? 1e6 : 0; }
    const distance = (grid: Float32Array) => {
        for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
            const i = y * width + x;
            if (x) grid[i] = Math.min(grid[i], grid[i - 1] + 1);
            if (y) { grid[i] = Math.min(grid[i], grid[i - width] + 1); if (x) grid[i] = Math.min(grid[i], grid[i - width - 1] + Math.SQRT2); if (x + 1 < width) grid[i] = Math.min(grid[i], grid[i - width + 1] + Math.SQRT2); }
        }
        for (let y = height - 1; y >= 0; y--) for (let x = width - 1; x >= 0; x--) {
            const i = y * width + x;
            if (x + 1 < width) grid[i] = Math.min(grid[i], grid[i + 1] + 1);
            if (y + 1 < height) { grid[i] = Math.min(grid[i], grid[i + width] + 1); if (x) grid[i] = Math.min(grid[i], grid[i + width - 1] + Math.SQRT2); if (x + 1 < width) grid[i] = Math.min(grid[i], grid[i + width + 1] + Math.SQRT2); }
        }
    };
    distance(inside); distance(outside);
    const result = new Uint8Array(count * 4);
    for (let i = 0; i < count; i++) { const value = Math.round(255 * Math.max(0, Math.min(1, 0.5 + (outside[i] - inside[i]) / range))); result[i * 4] = value; result[i * 4 + 1] = value; result[i * 4 + 2] = value; result[i * 4 + 3] = 255; }
    return result;
}

import { describe, expect, it } from 'vitest';
import { alphaToSdf } from '../text/unicode-sdf';
describe('Unicode glyph distance fields', () => {
    it('keeps glyph interiors positive and exterior pixels negative', () => {
        const pixels = new Uint8ClampedArray(9 * 9 * 4);
        for (let y = 3; y <= 5; y++) for (let x = 3; x <= 5; x++) pixels[(y * 9 + x) * 4 + 3] = 255;
        const field = alphaToSdf(pixels, 9, 9);
        expect(field[(4 * 9 + 4) * 4]).toBeGreaterThan(128);
        expect(field[0]).toBeLessThan(128);
        expect(field[(4 * 9 + 4) * 4]).toBe(field[(4 * 9 + 4) * 4 + 1]);
        expect(field[3]).toBe(255);
    });
});

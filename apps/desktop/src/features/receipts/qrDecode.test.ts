import jsQR from 'jsqr';
import { describe, expect, it } from 'vitest';

import FIXTURE from './fixtures/qr-nfce-inventado.txt?raw';
import { decodePixels } from './qrDecode';

// QR inventado (chave de acesso e CNPJ de exemplo), um módulo por caractere.
const MATRIX: string[] = FIXTURE.split('\n').filter((l: string) => /^[01]+$/.test(l));
const TEXT =
  'https://www.fazenda.pr.gov.br/nfce/qrcode?p=41261011222333000181650010000012341000012346|2|1|1|0A1B2C3D4E5F60718293A4B5C6D7E8F901234567';

/** Gerador pseudoaleatório fixo (o teste sempre igual). */
function rng(seed: number) {
  let s = seed;
  return () => {
    s = (s * 1103515245 + 12345) & 0x7fffffff;
    return s / 0x7fffffff;
  };
}

interface Photo {
  /** Inclinação em graus. */
  tilt?: number;
  /** Perspectiva: a borda de cima fica esta fração mais estreita. */
  keystone?: number;
  /** Sombra: fator de luz na parte sombreada (0,3 = muito escura). */
  shadow?: number;
  /** Ruído (± níveis de cinza). */
  noise?: number;
  /** Texto do verso aparecendo (linhas cinza no papel). */
  bleed?: boolean;
}

/** "Foto" do cupom: o QR num papel térmico cinza, com inclinação, perspectiva, sombra e ruído. */
function photo(p: Photo, W = 640, H = 760): Uint8ClampedArray {
  const n = MATRIX.length;
  const size = 360; // QR ocupa ~55% da largura
  const cx = W / 2;
  const cy = H / 2;
  const a = ((p.tilt ?? 0) * Math.PI) / 180;
  const rand = rng(7);
  const out = new Uint8ClampedArray(W * H * 4);
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      // De volta ao plano do papel: desfaz a rotação e a perspectiva.
      const dx = x - cx;
      const dy = y - cy;
      let u = dx * Math.cos(-a) - dy * Math.sin(-a);
      const v = dx * Math.sin(-a) + dy * Math.cos(-a);
      const k = 1 - (p.keystone ?? 0) * (0.5 - (v + size / 2) / size);
      u /= k;
      const mx = Math.floor(((u + size / 2) / size) * n);
      const my = Math.floor(((v + size / 2) / size) * n);
      let paper = 205;
      if (p.bleed && Math.floor((y + 3) / 9) % 3 === 0 && (x * 7 + y) % 11 < 6) paper -= 28;
      let val = paper;
      if (mx >= 0 && my >= 0 && mx < n && my < n && MATRIX[my]![mx] === '1') val = 45;
      // Sombra irregular cobrindo metade (a da mão ou do celular), com borda ondulada.
      if (p.shadow != null) {
        const edge = W * 0.52 + Math.sin(y / 37) * 40 + Math.sin(y / 11) * 12;
        const soft = Math.min(1, Math.max(0, (edge - x) / 18));
        val *= 1 - (1 - p.shadow) * soft;
      }
      val += ((rand() - 0.5) * 2 * (p.noise ?? 0)) | 0;
      const i = (y * W + x) * 4;
      out[i] = val;
      out[i + 1] = val;
      out[i + 2] = val;
      out[i + 3] = 255;
    }
  }
  return out;
}

describe('QR do cupom na foto', () => {
  it('lê o QR limpo', () => {
    expect(decodePixels(photo({}), 640, 760)).toBe(TEXT);
  });

  it('lê com sombra forte em metade do QR, papel cinza, verso aparecendo e inclinação', () => {
    const img = photo({ tilt: 7, keystone: 0.08, shadow: 0.3, noise: 14, bleed: true });
    expect(decodePixels(img, 640, 760)).toBe(TEXT);
  });

  it('o jsQR sozinho não lê a foto com sombra muito forte; com o nivelamento, sim', () => {
    const img = photo({ tilt: 4, shadow: 0.18, noise: 10, bleed: true });
    const plain = jsQR(img, 640, 760, { inversionAttempts: 'attemptBoth' });
    expect(plain?.data ?? null).toBeNull();
    expect(decodePixels(img, 640, 760)).toBe(TEXT);
  });
});

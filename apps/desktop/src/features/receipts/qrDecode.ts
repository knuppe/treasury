/**
 * Leitura do QR em pixels (jsQR), robusta a foto de celular de cupom de papel: sombra forte
 * sobre parte do QR, fundo cinza do papel térmico, texto do verso aparecendo e leve inclinação.
 * Além da imagem como veio, tenta:
 *
 * 1. a imagem com a iluminação nivelada (cada pixel dividido pelo fundo local, uma média em
 *    janela grande): tira a sombra e o cinza do papel;
 * 2. a imagem nivelada binarizada por limiar local (média da vizinhança, Bradley), em duas
 *    janelas.
 *
 * Função pura sobre RGBA, para testar sem canvas.
 */
import jsQR from 'jsqr';

type Pixels = Uint8ClampedArray;

function gray(rgba: Pixels, n: number): Float32Array {
  const g = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const k = i * 4;
    g[i] = 0.299 * rgba[k]! + 0.587 * rgba[k + 1]! + 0.114 * rgba[k + 2]!;
  }
  return g;
}

/** Imagem integral (somas acumuladas) para médias em janela em tempo constante. */
function integral(g: Float32Array, w: number, h: number): Float64Array {
  const s = new Float64Array((w + 1) * (h + 1));
  for (let y = 0; y < h; y++) {
    let row = 0;
    for (let x = 0; x < w; x++) {
      row += g[y * w + x]!;
      s[(y + 1) * (w + 1) + x + 1] = s[y * (w + 1) + x + 1]! + row;
    }
  }
  return s;
}

/** Média da janela (2r+1)² em volta de cada pixel. */
function boxMean(g: Float32Array, w: number, h: number, r: number): Float32Array {
  const s = integral(g, w, h);
  const out = new Float32Array(w * h);
  for (let y = 0; y < h; y++) {
    const y0 = Math.max(0, y - r);
    const y1 = Math.min(h, y + r + 1);
    for (let x = 0; x < w; x++) {
      const x0 = Math.max(0, x - r);
      const x1 = Math.min(w, x + r + 1);
      const sum =
        s[y1 * (w + 1) + x1]! -
        s[y0 * (w + 1) + x1]! -
        s[y1 * (w + 1) + x0]! +
        s[y0 * (w + 1) + x0]!;
      out[y * w + x] = sum / ((x1 - x0) * (y1 - y0));
    }
  }
  return out;
}

function toRgba(g: Float32Array, n: number): Pixels {
  const out = new Uint8ClampedArray(n * 4);
  for (let i = 0; i < n; i++) {
    const v = g[i]!;
    out[i * 4] = v;
    out[i * 4 + 1] = v;
    out[i * 4 + 2] = v;
    out[i * 4 + 3] = 255;
  }
  return out;
}

/** Iluminação nivelada: o pixel dividido pelo fundo local (sombra e papel cinza somem). */
export function flatten(g: Float32Array, w: number, h: number): Float32Array {
  const r = Math.max(8, Math.round(Math.min(w, h) / 10));
  // O fundo é o claro da vizinhança: a média puxa para baixo onde há muito preto, então
  // usa o maior entre a média e a média da janela menor.
  const big = boxMean(g, w, h, r);
  const small = boxMean(g, w, h, Math.max(4, Math.round(r / 3)));
  const out = new Float32Array(w * h);
  for (let i = 0; i < out.length; i++) {
    const bg = Math.max(big[i]!, small[i]!, 1);
    out[i] = Math.min(255, (g[i]! / bg) * 200);
  }
  return out;
}

/** Limiar local (Bradley): escuro se ficar `t` abaixo da média da vizinhança. */
export function threshold(
  g: Float32Array,
  w: number,
  h: number,
  r: number,
  t = 0.15,
): Float32Array {
  const mean = boxMean(g, w, h, r);
  const out = new Float32Array(w * h);
  for (let i = 0; i < out.length; i++) out[i] = g[i]! < mean[i]! * (1 - t) ? 0 : 255;
  return out;
}

/** O texto do QR nos pixels, ou `null`. */
export function decodePixels(rgba: Pixels, w: number, h: number): string | null {
  const first = jsQR(rgba, w, h, { inversionAttempts: 'attemptBoth' });
  if (first?.data) return first.data;
  const n = w * h;
  const side = Math.min(w, h);
  const g0 = gray(rgba, n);
  // Primeiro a imagem como veio, nivelada; depois suavizada (o ruído do sensor no escuro da
  // sombra some), cada uma também binarizada por limiar local em duas janelas.
  for (const g of [g0, boxMean(g0, w, h, 1)]) {
    const flat = flatten(g, w, h);
    const tries = [flat];
    for (const div of [16, 8]) {
      tries.push(threshold(flat, w, h, Math.max(6, Math.round(side / div)), 0.1));
    }
    for (const t of tries) {
      const code = jsQR(toRgba(t, n), w, h, { inversionAttempts: 'dontInvert' });
      if (code?.data) return code.data;
    }
  }
  return null;
}

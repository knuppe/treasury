//! PaddleOCR embutido, em Rust puro (docs/ocr-receipts.md §12.2): a detecção do PP-OCRv3 (DBNet)
//! acha as caixas de texto e o reconhecimento latino do PP-OCRv5 *mobile* lê cada uma. Os
//! modelos ONNX vão dentro do binário (`assets/ocr/`) e rodam no `tract`, sem ONNX Runtime nem
//! biblioteca nativa; a foto não sai do computador. Os parâmetros seguem os padrões do RapidOCR,
//! o pacote medido no protótipo.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use image::{DynamicImage, ImageDecoder, ImageReader, RgbImage};
use tract_onnx::prelude::*;

use super::OcrLine;
use crate::error::{Error, Result};
use crate::i18n::t;

/// Leitura de uma linha recortada: o texto e a confiança do reconhecimento.
type LineRead = Result<(String, f32)>;

static DET_ONNX: &[u8] = include_bytes!("../../../assets/ocr/det.onnx");
static REC_ONNX: &[u8] = include_bytes!("../../../assets/ocr/rec_latin.onnx");
static REC_DICT: &str = include_str!("../../../assets/ocr/rec_latin_dict.txt");

/// Lado menor mínimo da imagem na detecção (RapidOCR: `limit_side_len` 736, `limit_type` min).
const DET_MIN_SIDE: f32 = 736.0;
/// Lado maior máximo na detecção: protege a memória e o tempo em máquina fraca (foto de 12 MP
/// cai para ~2,8 MP). No banco de testes, 1920 leu tão bem quanto 2560 (um pouco melhor).
const DET_MAX_SIDE: f32 = 1920.0;
/// Limiar do mapa de probabilidade (DB `thresh`).
const DB_THRESH: f32 = 0.3;
/// Nota mínima da caixa: média da probabilidade dentro dela (DB `box_thresh`).
const DB_BOX_THRESH: f32 = 0.5;
/// Quanto a caixa cresce para cobrir a letra inteira (DB `unclip_ratio`).
const DB_UNCLIP: f32 = 1.6;
/// Lado menor mínimo da caixa, em pixels do mapa (DB `min_size`).
const DB_MIN_SIZE: f32 = 3.0;
/// Caixas consideradas por foto (DB `max_candidates`).
const DB_MAX_CANDIDATES: usize = 1000;
/// Altura da linha no reconhecimento.
const REC_H: usize = 48;
/// Larguras da entrada do reconhecimento (múltiplos de 32, ~1,25× entre si): cada linha vai
/// para a primeira que cabe, completada à direita, e cada largura tem o seu plano otimizado.
const REC_WIDTHS: [usize; 16] = [
    96, 128, 160, 192, 256, 320, 384, 480, 608, 768, 960, 1216, 1536, 1920, 2400, 3008,
];
/// Confiança mínima do texto lido (RapidOCR `text_score`).
const REC_MIN_CONF: f32 = 0.5;
/// Caixa em pé (altura ≥ 2,5× a largura) é texto vertical: gira antes de ler. O RapidOCR gira
/// a partir de 1,5×, o que estraga um "1" sozinho (coluna de quantidade).
const REC_VERTICAL: f32 = 2.5;
/// Faixa mínima (miolo, em pixels do mapa) para dividir a detecção entre núcleos.
const DET_STRIP_MIN: usize = 256;
/// Margem de cada faixa da detecção, descartada (o contexto da borda muda o mapa perto dela).
const DET_MARGIN: usize = 64;
/// Planos de detecção guardados (um por tamanho de imagem).
const DET_PLANS: usize = 4;

type Plan = TypedRunnableModel<TypedModel>;
/// O texto de uma linha e a confiança.
type Reading = (String, f32);

/// Um modelo carregado com as dimensões simbólicas e os planos otimizados por tamanho concreto
/// (no `tract`, o plano concreto roda 2,5× a 8× mais rápido que o simbólico, e otimizar leva
/// ~25 ms).
struct Net {
    model: TypedModel,
    syms: [Symbol; 2],
    plans: Mutex<HashMap<(usize, usize), Arc<Plan>>>,
    cap: usize,
}

impl Net {
    /// `shape` monta a forma da entrada com os dois símbolos (`a`, `b`).
    fn load(bytes: &[u8], shape: impl Fn(&Symbol, &Symbol) -> [TDim; 4]) -> TractResult<Net> {
        let mut m = tract_onnx::onnx().model_for_read(&mut &bytes[..])?;
        let scope = m.symbols.clone();
        let (a, b) = (scope.sym("a"), scope.sym("b"));
        m.set_input_fact(0, f32::fact(shape(&a, &b)).into())?;
        let model = m.into_typed()?.into_decluttered()?;
        Ok(Net {
            model,
            syms: [a, b],
            plans: Mutex::new(HashMap::new()),
            cap: usize::MAX,
        })
    }

    /// O plano otimizado para os valores `a` e `b` dos símbolos (do cache ou feito agora).
    fn plan(&self, a: usize, b: usize) -> Result<Arc<Plan>> {
        if let Some(p) = self
            .plans
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&(a, b))
        {
            return Ok(p.clone());
        }
        let values = SymbolValues::default()
            .with(&self.syms[0], a as i64)
            .with(&self.syms[1], b as i64);
        let plan = Arc::new(
            self.model
                .concretize_dims(&values)
                .and_then(|m| m.into_optimized())
                .and_then(|m| m.into_runnable())
                .map_err(engine_err)?,
        );
        let mut plans = self.plans.lock().unwrap_or_else(|e| e.into_inner());
        if plans.len() >= self.cap {
            plans.clear();
        }
        plans.insert((a, b), plan.clone());
        Ok(plan)
    }
}

fn engine_err(e: impl std::fmt::Display) -> Error {
    Error::Other(format!("PaddleOCR: {e}"))
}

/// Detecção: entrada `[1, 3, 32·a, 32·b]`.
fn det_net() -> Result<&'static Net> {
    static NET: OnceLock<std::result::Result<Net, String>> = OnceLock::new();
    NET.get_or_init(|| {
        Net::load(DET_ONNX, |a, b| {
            [1.into(), 3.into(), a.to_dim() * 32, b.to_dim() * 32]
        })
        .map(|mut n| {
            n.cap = DET_PLANS;
            n
        })
        .map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(engine_err)
}

/// Reconhecimento: entrada `[a, 3, 48, b]` (uma linha por vez: no `tract`, juntar linhas na
/// mesma chamada não ganha tempo; rodar em paralelo ganha).
fn rec_net() -> Result<&'static Net> {
    static NET: OnceLock<std::result::Result<Net, String>> = OnceLock::new();
    NET.get_or_init(|| {
        Net::load(REC_ONNX, |a, b| {
            [a.to_dim(), 3.into(), (REC_H as i64).into(), b.to_dim()]
        })
        .map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(engine_err)
}

/// Os caracteres do CTC: 0 é o branco, depois o dicionário e por fim o espaço (como o
/// `CTCLabelDecode` do RapidOCR).
fn charset() -> &'static [String] {
    static CHARS: OnceLock<Vec<String>> = OnceLock::new();
    CHARS.get_or_init(|| {
        let mut v = vec![String::new()];
        v.extend(
            REC_DICT
                .split('\n')
                .map(|l| l.trim_end_matches('\r'))
                .filter(|l| !l.is_empty())
                .map(str::to_string),
        );
        v.push(" ".into());
        v
    })
}

/// Lê o texto da foto (JPEG, PNG ou WebP, girada pela EXIF). As linhas saem de cima para baixo,
/// com coordenadas normalizadas e a origem embaixo, como no Apple Vision.
pub fn recognize(bytes: &[u8]) -> Result<Vec<OcrLine>> {
    recognize_with_size(bytes).map(|(lines, _, _)| lines)
}

/// Como [`recognize`], com a largura e a altura da foto já girada (para o `aspect` do leitor de
/// linhas e para o banco de testes).
pub fn recognize_with_size(bytes: &[u8]) -> Result<(Vec<OcrLine>, u32, u32)> {
    let img = decode(bytes)?;
    let (w, h) = img.dimensions();
    let boxes = detect(&img)?;
    let lines = read_boxes(&img, &boxes)?;
    Ok((lines, w, h))
}

/// Abre a foto e aplica a rotação da EXIF.
fn decode(bytes: &[u8]) -> Result<RgbImage> {
    let bad = |_| Error::invalid(t("nucleo-ocr-erro-imagem"));
    let mut dec = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| Error::invalid(t("nucleo-ocr-erro-imagem")))?
        .into_decoder()
        .map_err(bad)?;
    let orientation = dec
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(dec).map_err(bad)?;
    img.apply_orientation(orientation);
    let img = img.into_rgb8();
    if img.width() < 4 || img.height() < 4 {
        return Err(Error::invalid(t("nucleo-ocr-erro-imagem")));
    }
    Ok(img)
}

/// Uma caixa de texto na foto, em pixels: canto de cima à esquerda (`o`), o vetor da borda de
/// cima (`ex`, da esquerda para a direita) e o da borda esquerda (`ey`, de cima para baixo).
#[derive(Debug, Clone, Copy)]
struct TextBox {
    o: [f32; 2],
    ex: [f32; 2],
    ey: [f32; 2],
}

impl TextBox {
    fn corners(&self) -> [[f32; 2]; 4] {
        let [ox, oy] = self.o;
        let (ex, ey) = (self.ex, self.ey);
        [
            [ox, oy],
            [ox + ex[0], oy + ex[1]],
            [ox + ex[0] + ey[0], oy + ex[1] + ey[1]],
            [ox + ey[0], oy + ey[1]],
        ]
    }
}

/// Acha as caixas de texto (DBNet): mapa de probabilidade, limiar, regiões ligadas, caixa
/// orientada de cada região, nota e expansão (`unclip`).
fn detect(img: &RgbImage) -> Result<Vec<TextBox>> {
    let (sw, sh) = (img.width() as f32, img.height() as f32);
    let mut ratio = if sw.min(sh) < DET_MIN_SIDE {
        DET_MIN_SIDE / sw.min(sh)
    } else {
        1.0
    };
    if sw.max(sh) * ratio > DET_MAX_SIDE {
        ratio = DET_MAX_SIDE / sw.max(sh);
    }
    let round32 = |v: f32| (((v * ratio) / 32.0).round() as usize).max(1) * 32;
    let (dw, dh) = (round32(sw), round32(sh));
    let resized = image::imageops::resize(
        img,
        dw as u32,
        dh as u32,
        image::imageops::FilterType::Triangle,
    );
    // BGR, normalizado pela média e desvio do ImageNet, NCHW.
    let mean = [0.485f32, 0.456, 0.406];
    let std = [0.229f32, 0.224, 0.225];
    let plane = dw * dh;
    let mut data = vec![0f32; 3 * plane];
    for (i, p) in resized.pixels().enumerate() {
        for c in 0..3 {
            let v = p.0[2 - c] as f32 / 255.0;
            data[c * plane + i] = (v - mean[c]) / std[c];
        }
    }
    let prob = prob_map(&data, dw, dh, threads())?;
    Ok(boxes_from_map(&prob, dw, dh)
        .into_iter()
        .filter_map(|b| to_image(&b, dw, dh, sw, sh))
        .collect())
}

/// Quantas tarefas em paralelo (até 8).
fn threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, 8)
}

/// O mapa de probabilidade da detecção. Imagem alta vai em faixas horizontais de mesma altura,
/// cada uma com margem de `DET_MARGIN` em cima e embaixo, rodadas em paralelo (no `tract`, uma
/// chamada usa um núcleo só); de cada faixa vale só o miolo.
fn prob_map(data: &[f32], dw: usize, dh: usize, threads: usize) -> Result<Vec<f32>> {
    let plane = dw * dh;
    let n = threads.min(dh / DET_STRIP_MIN).max(1);
    if n == 1 {
        let input = Tensor::from_shape(&[1, 3, dh, dw], data).map_err(engine_err)?;
        let plan = det_net()?.plan(dh / 32, dw / 32)?;
        let out = plan.run(tvec!(input.into())).map_err(engine_err)?;
        let prob = out[0].as_slice::<f32>().map_err(engine_err)?;
        if prob.len() != plane {
            return Err(engine_err("saída inesperada da detecção"));
        }
        return Ok(prob.to_vec());
    }
    let core = dh.div_ceil(n).div_ceil(32) * 32;
    let sh = (core + 2 * DET_MARGIN).min(dh);
    let plan = det_net()?.plan(sh / 32, dw / 32)?;
    let mut prob = vec![0f32; plane];
    let failed: Mutex<Option<Error>> = Mutex::new(None);
    std::thread::scope(|s| {
        for (i, rows) in prob.chunks_mut(core * dw).enumerate() {
            let (plan, failed) = (&plan, &failed);
            s.spawn(move || {
                let c0 = i * core;
                let top = c0.saturating_sub(DET_MARGIN).min(dh - sh);
                let mut strip = vec![0f32; 3 * sh * dw];
                for c in 0..3 {
                    strip[c * sh * dw..][..sh * dw]
                        .copy_from_slice(&data[c * plane + top * dw..][..sh * dw]);
                }
                let run = Tensor::from_shape(&[1, 3, sh, dw], &strip)
                    .and_then(|t| plan.run(tvec!(t.into())))
                    .map_err(engine_err)
                    .and_then(|out| {
                        let p = out[0].as_slice::<f32>().map_err(engine_err)?;
                        if p.len() != sh * dw {
                            return Err(engine_err("saída inesperada da detecção"));
                        }
                        let from = (c0 - top) * dw;
                        rows.copy_from_slice(&p[from..][..rows.len()]);
                        Ok(())
                    });
                if let Err(e) = run {
                    *failed.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                }
            });
        }
    });
    match failed.into_inner().unwrap_or_else(|e| e.into_inner()) {
        Some(e) => Err(e),
        None => Ok(prob),
    }
}

/// Caixas no mapa de probabilidade (coordenadas do mapa).
fn boxes_from_map(prob: &[f32], w: usize, h: usize) -> Vec<TextBox> {
    // Limiar e dilatação 2×2 (o pixel acende se ele, o de cima, o da esquerda ou o da diagonal
    // passam do limiar, como o `cv2.dilate` do RapidOCR).
    let on: Vec<bool> = prob.iter().map(|&p| p > DB_THRESH).collect();
    let mut mask = on.clone();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !on[i]
                && ((x > 0 && on[i - 1])
                    || (y > 0 && on[i - w])
                    || (x > 0 && y > 0 && on[i - w - 1]))
            {
                mask[i] = true;
            }
        }
    }
    let mut seen = vec![false; w * h];
    let mut stack = Vec::new();
    let mut pixels: Vec<(u32, u32)> = Vec::new();
    let mut out = Vec::new();
    let mut candidates = 0;
    for start in 0..w * h {
        if !mask[start] || seen[start] {
            continue;
        }
        candidates += 1;
        if candidates > DB_MAX_CANDIDATES {
            break;
        }
        // Região ligada (vizinhança de 8), por pilha.
        pixels.clear();
        seen[start] = true;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            pixels.push((x as u32, y as u32));
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let j = ny as usize * w + nx as usize;
                    if mask[j] && !seen[j] {
                        seen[j] = true;
                        stack.push(j);
                    }
                }
            }
        }
        if pixels.len() < 4 {
            continue;
        }
        if let Some(b) = region_box(&pixels, prob, w, h) {
            out.push(b);
        }
    }
    out
}

/// A caixa orientada de uma região: eixo pelos momentos de segunda ordem (região redonda fica
/// reta), extensão pelos centros dos pixels, nota pela média da probabilidade dentro da caixa e
/// expansão pela regra do DB (`área × 1,6 / perímetro` de cada lado).
fn region_box(pixels: &[(u32, u32)], prob: &[f32], w: usize, h: usize) -> Option<TextBox> {
    let n = pixels.len() as f64;
    let (mut sx, mut sy) = (0f64, 0f64);
    for &(x, y) in pixels {
        sx += x as f64;
        sy += y as f64;
    }
    let (cx, cy) = (sx / n, sy / n);
    let (mut sxx, mut syy, mut sxy) = (0f64, 0f64, 0f64);
    for &(x, y) in pixels {
        let (dx, dy) = (x as f64 - cx, y as f64 - cy);
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    let (sxx, syy, sxy) = (sxx / n, syy / n, sxy / n);
    let tr = sxx + syy;
    let disc = ((sxx - syy).powi(2) / 4.0 + sxy * sxy).sqrt();
    let (l1, l2) = (tr / 2.0 + disc, tr / 2.0 - disc);
    let mut theta = if l1 > 2.0 * l2.max(1e-9) {
        0.5 * (2.0 * sxy).atan2(sxx - syy)
    } else {
        0.0
    };
    // O eixo `u` fica o mais perto da horizontal (entre −45° e 45°).
    let quarter = std::f64::consts::FRAC_PI_4;
    if theta > quarter {
        theta -= 2.0 * quarter;
    } else if theta <= -quarter {
        theta += 2.0 * quarter;
    }
    let (c, s) = (theta.cos() as f32, theta.sin() as f32);
    let (cx, cy) = (cx as f32, cy as f32);
    let (mut u0, mut u1, mut v0, mut v1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for &(x, y) in pixels {
        let (dx, dy) = (x as f32 - cx, y as f32 - cy);
        let (u, v) = (dx * c + dy * s, -dx * s + dy * c);
        u0 = u0.min(u);
        u1 = u1.max(u);
        v0 = v0.min(v);
        v1 = v1.max(v);
    }
    let (bw, bh) = (u1 - u0, v1 - v0);
    if bw.min(bh) < DB_MIN_SIZE {
        return None;
    }
    // Nota: média da probabilidade dentro da caixa (modo "fast" do DB).
    let at = |u: f32, v: f32| [cx + u * c - v * s, cy + u * s + v * c];
    let corners = [at(u0, v0), at(u1, v0), at(u1, v1), at(u0, v1)];
    let fx = |f: fn(f32, f32) -> f32, k: usize| corners.iter().map(|p| p[k]).fold(corners[0][k], f);
    let x0 = fx(f32::min, 0).floor().max(0.0) as usize;
    let x1 = (fx(f32::max, 0).ceil() as usize).min(w - 1);
    let y0 = fx(f32::min, 1).floor().max(0.0) as usize;
    let y1 = (fx(f32::max, 1).ceil() as usize).min(h - 1);
    let (mut sum, mut cnt) = (0f32, 0u32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            let (u, v) = (dx * c + dy * s, -dx * s + dy * c);
            if u >= u0 - 0.5 && u <= u1 + 0.5 && v >= v0 - 0.5 && v <= v1 + 0.5 {
                sum += prob[y * w + x];
                cnt += 1;
            }
        }
    }
    if cnt == 0 || sum / (cnt as f32) < DB_BOX_THRESH {
        return None;
    }
    let d = bw * bh * DB_UNCLIP / (2.0 * (bw + bh));
    let (u0, u1, v0, v1) = (u0 - d, u1 + d, v0 - d, v1 + d);
    if (u1 - u0).min(v1 - v0) < DB_MIN_SIZE + 2.0 {
        return None;
    }
    let o = at(u0, v0);
    let tr = at(u1, v0);
    let bl = at(u0, v1);
    Some(TextBox {
        o,
        ex: [tr[0] - o[0], tr[1] - o[1]],
        ey: [bl[0] - o[0], bl[1] - o[1]],
    })
}

/// Leva a caixa do mapa para os pixels da foto, presa às bordas; caixa fina demais sai.
fn to_image(b: &TextBox, dw: usize, dh: usize, sw: f32, sh: f32) -> Option<TextBox> {
    let (kx, ky) = (sw / dw as f32, sh / dh as f32);
    let map = |p: [f32; 2]| {
        [
            (p[0] * kx).round().clamp(0.0, sw - 1.0),
            (p[1] * ky).round().clamp(0.0, sh - 1.0),
        ]
    };
    let [o, tr, _, bl] = b.corners().map(map);
    let ex = [tr[0] - o[0], tr[1] - o[1]];
    let ey = [bl[0] - o[0], bl[1] - o[1]];
    let len = |v: [f32; 2]| v[0].hypot(v[1]);
    (len(ex) > 3.0 && len(ey) > 3.0).then_some(TextBox { o, ex, ey })
}

/// A linha recortada e pronta para o reconhecimento: `[3, 48, largura]` (BGR, de −1 a 1).
struct Crop {
    data: Vec<f32>,
    width: usize,
}

/// Recorta a caixa da foto já endireitada e na altura 48, mantendo a proporção. Amostra
/// bilinear, com média de vários pontos quando reduz muito (evita serrilhado na letra).
fn crop(img: &RgbImage, b: &TextBox) -> Crop {
    let len = |v: [f32; 2]| v[0].hypot(v[1]);
    let (mut o, mut ex, mut ey) = (b.o, b.ex, b.ey);
    if len(ey) >= REC_VERTICAL * len(ex) {
        // Texto em pé: lê de cima para baixo (a partir do canto de cima à direita).
        o = [b.o[0] + b.ex[0], b.o[1] + b.ex[1]];
        (ex, ey) = (b.ey, [-b.ex[0], -b.ex[1]]);
    }
    let (cw, ch) = (len(ex).max(1.0), len(ey).max(1.0));
    let max_w = REC_WIDTHS[REC_WIDTHS.len() - 1];
    let width = ((REC_H as f32 * cw / ch).ceil() as usize).clamp(1, max_w);
    let ss = ((ch / REC_H as f32).round() as usize).clamp(1, 4);
    let (iw, ih) = (img.width() as usize, img.height() as usize);
    let raw = img.as_raw();
    let px = |x: usize, y: usize, c: usize| raw[(y * iw + x) * 3 + c] as f32;
    let sample = |x: f32, y: f32, out: &mut [f32; 3]| {
        let x = x.clamp(0.0, (iw - 1) as f32);
        let y = y.clamp(0.0, (ih - 1) as f32);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(iw - 1), (y0 + 1).min(ih - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        for (c, o) in out.iter_mut().enumerate() {
            let top = px(x0, y0, c) * (1.0 - fx) + px(x1, y0, c) * fx;
            let bot = px(x0, y1, c) * (1.0 - fx) + px(x1, y1, c) * fx;
            *o += top * (1.0 - fy) + bot * fy;
        }
    };
    let plane = REC_H * width;
    let mut data = vec![0f32; 3 * plane];
    let k = (ss * ss) as f32;
    for oy in 0..REC_H {
        for ox in 0..width {
            let mut acc = [0f32; 3];
            for sy in 0..ss {
                for sx in 0..ss {
                    let fu = (ox as f32 + (sx as f32 + 0.5) / ss as f32) / width as f32;
                    let fv = (oy as f32 + (sy as f32 + 0.5) / ss as f32) / REC_H as f32;
                    let x = o[0] + fu * ex[0] + fv * ey[0] - 0.5;
                    let y = o[1] + fu * ex[1] + fv * ey[1] - 0.5;
                    sample(x, y, &mut acc);
                }
            }
            let i = oy * width + ox;
            for c in 0..3 {
                // BGR: o canal 0 é o azul.
                let v = acc[2 - c] / k / 255.0;
                data[c * plane + i] = (v - 0.5) / 0.5;
            }
        }
    }
    Crop { data, width }
}

/// A largura da entrada para uma linha: a primeira de `REC_WIDTHS` que cabe.
fn bucket(width: usize) -> usize {
    REC_WIDTHS
        .iter()
        .copied()
        .find(|&b| b >= width)
        .unwrap_or(REC_WIDTHS[REC_WIDTHS.len() - 1])
}

/// Lê uma linha: completa à direita até a largura do plano, roda e decodifica o CTC (o maior
/// de cada passo, sem brancos nem repetidos). A confiança é a média dos caracteres mantidos.
fn read_crop(c: &Crop, plan: &Plan, bw: usize) -> Result<Reading> {
    let plane = REC_H * bw;
    let mut data = vec![0f32; 3 * plane];
    for ch in 0..3 {
        for y in 0..REC_H {
            let src = &c.data[ch * REC_H * c.width + y * c.width..][..c.width];
            data[ch * plane + y * bw..][..c.width].copy_from_slice(src);
        }
    }
    let input = Tensor::from_shape(&[1, 3, REC_H, bw], &data).map_err(engine_err)?;
    let out = plan.run(tvec!(input.into())).map_err(engine_err)?;
    let shape = out[0].shape().to_vec();
    let probs = out[0].as_slice::<f32>().map_err(engine_err)?;
    let chars = charset();
    let classes = *shape.last().unwrap_or(&0);
    if classes == 0 || classes != chars.len() {
        return Err(engine_err("saída inesperada do reconhecimento"));
    }
    let (mut text, mut sum, mut n, mut prev) = (String::new(), 0f32, 0u32, 0usize);
    for step in probs.chunks_exact(classes) {
        let (idx, p) = step
            .iter()
            .copied()
            .enumerate()
            .fold((0, f32::MIN), |a, (i, v)| if v > a.1 { (i, v) } else { a });
        if idx != 0 && idx != prev {
            text.push_str(&chars[idx]);
            sum += p;
            n += 1;
        }
        prev = idx;
    }
    Ok((text, if n > 0 { sum / n as f32 } else { 0.0 }))
}

/// Recorta e lê as caixas em paralelo (uma linha por tarefa) e monta as linhas de saída, de cima
/// para baixo.
fn read_boxes(img: &RgbImage, boxes: &[TextBox]) -> Result<Vec<OcrLine>> {
    if boxes.is_empty() {
        return Ok(vec![]);
    }
    let net = rec_net()?;
    let threads = threads().min(boxes.len());
    let crops: Vec<Crop> = boxes.iter().map(|b| crop(img, b)).collect();
    // Prepara os planos que faltam, também em paralelo.
    let mut widths: Vec<usize> = crops.iter().map(|c| bucket(c.width)).collect();
    widths.sort_unstable();
    widths.dedup();
    let next = AtomicUsize::new(0);
    let plans: Mutex<HashMap<usize, Arc<Plan>>> = Mutex::new(HashMap::new());
    let failed: Mutex<Option<Error>> = Mutex::new(None);
    std::thread::scope(|s| {
        for _ in 0..threads.min(widths.len()) {
            s.spawn(|| {
                while let Some(&bw) = widths.get(next.fetch_add(1, Ordering::Relaxed)) {
                    match net.plan(1, bw) {
                        Ok(p) => {
                            plans
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .insert(bw, p);
                        }
                        Err(e) => {
                            *failed.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                        }
                    }
                }
            });
        }
    });
    if let Some(e) = failed.into_inner().unwrap_or_else(|e| e.into_inner()) {
        return Err(e);
    }
    let plans = plans.into_inner().unwrap_or_else(|e| e.into_inner());
    // As linhas largas primeiro, para as tarefas terminarem juntas.
    let mut order: Vec<usize> = (0..crops.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(crops[i].width));
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<LineRead>>> =
        Mutex::new((0..crops.len()).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                while let Some(&i) = order.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let bw = bucket(crops[i].width);
                    let r = read_crop(&crops[i], &plans[&bw], bw);
                    results.lock().unwrap_or_else(|e| e.into_inner())[i] = Some(r);
                }
            });
        }
    });
    let results = results.into_inner().unwrap_or_else(|e| e.into_inner());
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let mut lines: Vec<(TextBox, OcrLine)> = Vec::new();
    for (b, r) in boxes.iter().zip(results) {
        let (text, conf) = r.unwrap_or_else(|| Err(engine_err("linha sem leitura")))?;
        let text = text.trim().to_string();
        if text.is_empty() || conf < REC_MIN_CONF {
            continue;
        }
        let cs = b.corners();
        let xs = cs.map(|p| p[0]);
        let ys = cs.map(|p| p[1]);
        let (x0, x1) = (
            xs.into_iter().fold(f32::MAX, f32::min),
            xs.into_iter().fold(f32::MIN, f32::max),
        );
        let (y0, y1) = (
            ys.into_iter().fold(f32::MAX, f32::min),
            ys.into_iter().fold(f32::MIN, f32::max),
        );
        let corner = |p: [f32; 2]| [p[0] / iw, 1.0 - p[1] / ih];
        lines.push((
            *b,
            OcrLine {
                text,
                conf,
                x: x0 / iw,
                y: 1.0 - y1 / ih,
                w: (x1 - x0) / iw,
                h: (y1 - y0) / ih,
                tl: Some(corner(cs[0])),
                tr: Some(corner(cs[1])),
            },
        ));
    }
    // De cima para baixo e, na mesma altura (10 px), da esquerda para a direita (como o
    // `sorted_boxes` do RapidOCR).
    lines.sort_by(|a, b| {
        a.0.o[1]
            .total_cmp(&b.0.o[1])
            .then(a.0.o[0].total_cmp(&b.0.o[0]))
    });
    for i in 0..lines.len().saturating_sub(1) {
        let (a, b) = (lines[i].0.o, lines[i + 1].0.o);
        if (b[1] - a[1]).abs() < 10.0 && b[0] < a[0] {
            lines.swap(i, i + 1);
        }
    }
    Ok(lines.into_iter().map(|(_, l)| l).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(img: &RgbImage) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn blank_photo_has_no_lines() {
        let img = RgbImage::from_pixel(400, 600, image::Rgb([255, 255, 255]));
        let (lines, w, h) = recognize_with_size(&png(&img)).unwrap();
        assert!(lines.is_empty(), "{lines:?}");
        assert_eq!((w, h), (400, 600));
    }

    #[test]
    fn unknown_format_and_image_error() {
        let e = recognize(b"isto nao e uma foto").unwrap_err();
        assert_eq!(e.code(), "invalid");
    }

    #[test]
    fn dictionary_matches_the_model() {
        // 836 characters + blank + space = 838 recognition outputs.
        assert_eq!(charset().len(), 838);
        assert_eq!(charset()[1], "0");
        assert_eq!(charset()[837], " ");
    }

    /// Reads the photo from `TREASURY_OCR_SAMPLE` and shows the lines and the time:
    /// `TREASURY_OCR_SAMPLE=photo.jpg cargo test --release -p treasury-core --lib paddle -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn sample_from_env() {
        let Ok(path) = std::env::var("TREASURY_OCR_SAMPLE") else {
            return;
        };
        let bytes = std::fs::read(&path).unwrap();
        for round in 0..2 {
            let t0 = std::time::Instant::now();
            let (lines, w, h) = recognize_with_size(&bytes).unwrap();
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            println!(
                "round {round}: {w}×{h}, {} lines in {ms:.0} ms",
                lines.len()
            );
            if round == 1 {
                for l in &lines {
                    println!("{:.2} y={:.3} x={:.3}  {}", l.conf, l.y, l.x, l.text);
                }
            }
        }
    }

    /// Test bench (docs/ocr-receipts.md §9): reads the photos listed in `TREASURY_OCR_LIST`
    /// (one path per line) and writes to `TREASURY_OCR_OUT` in the `vision_ocr` format
    /// (`[{file, ms, width, height, lines}]`).
    #[test]
    #[ignore]
    fn test_bench() {
        let (Ok(list), Ok(out)) = (
            std::env::var("TREASURY_OCR_LIST"),
            std::env::var("TREASURY_OCR_OUT"),
        ) else {
            return;
        };
        let mut res = Vec::new();
        for path in std::fs::read_to_string(list)
            .unwrap()
            .lines()
            .filter(|l| !l.trim().is_empty())
        {
            let entry = match std::fs::read(path) {
                Ok(bytes) => {
                    let t0 = std::time::Instant::now();
                    match recognize_with_size(&bytes) {
                        Ok((lines, w, h)) => {
                            let ms = t0.elapsed().as_secs_f64() * 1000.0;
                            println!("{ms:7.0} ms  {w}×{h}  {:3} lines  {path}", lines.len());
                            serde_json::json!({"file": path, "ms": ms, "width": w, "height": h, "lines": lines})
                        }
                        Err(e) => serde_json::json!({"file": path, "error": e.to_string()}),
                    }
                }
                Err(e) => serde_json::json!({"file": path, "error": e.to_string()}),
            };
            res.push(entry);
        }
        std::fs::write(out, serde_json::to_string(&res).unwrap()).unwrap();
    }
}
#[cfg(test)]
mod memtmp {
    use super::*;
    fn rss() -> String {
        let o = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .unwrap();
        format!(
            "{} MB",
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u64>()
                .unwrap()
                / 1024
        )
    }
    #[test]
    #[ignore]
    fn memdet() {
        let n: usize = std::env::var("THREADS").unwrap().parse().unwrap();
        let (dw, dh): (usize, usize) = (
            std::env::var("DW").unwrap().parse().unwrap(),
            std::env::var("DH").unwrap().parse().unwrap(),
        );
        let data = vec![0.1f32; 3 * dw * dh];
        det_net().unwrap();
        eprintln!("det net {}", rss());
        let t0 = std::time::Instant::now();
        prob_map(&data, dw, dh, n).unwrap();
        eprintln!("n={n} {} {:?}", rss(), t0.elapsed());
        let t0 = std::time::Instant::now();
        prob_map(&data, dw, dh, n).unwrap();
        eprintln!("n={n} {} {:?}", rss(), t0.elapsed());
    }
    #[test]
    #[ignore]
    fn mem() {
        let path = std::env::var("TREASURY_OCR_SAMPLE").unwrap();
        let bytes = std::fs::read(&path).unwrap();
        eprintln!("start {}", rss());
        det_net().unwrap();
        eprintln!("det net {}", rss());
        rec_net().unwrap();
        eprintln!("rec net {}", rss());
        let img = decode(&bytes).unwrap();
        eprintln!("decoded {}", rss());
        let boxes = detect(&img).unwrap();
        eprintln!("detect {}", rss());
        let _ = detect(&img).unwrap();
        eprintln!("detect2 {}", rss());
        for w in REC_WIDTHS {
            rec_net().unwrap().plan(1, w).unwrap();
        }
        eprintln!("rec plans {}", rss());
        let l = read_boxes(&img, &boxes).unwrap();
        eprintln!("read {} {}", l.len(), rss());
        let l = read_boxes(&img, &boxes).unwrap();
        eprintln!("read2 {} {}", l.len(), rss());
    }
}

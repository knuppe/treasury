//! Gerador de PDF mínimo, sem dependências: páginas A4 com texto nas fontes padrão do PDF
//! (Helvetica e Helvetica-Bold, que todo leitor tem, em WinAnsiEncoding: cobre português,
//! espanhol e inglês), retângulos, linhas e polilinhas. Bastante para o relatório mensal, que
//! é gerado no próprio computador, sem serviço externo.
//!
//! Coordenadas em pontos (1/72 pol.), com a origem no canto de CIMA à esquerda (o writer
//! inverte para o sistema do PDF). Cores em RGB 0–255.

use std::fmt::Write as _;

pub const A4_W: f32 = 595.28;
pub const A4_H: f32 = 841.89;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Regular,
    Bold,
}

#[derive(Clone, Copy)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    fn ops(self, stroke: bool) -> String {
        let f = |c: u8| format!("{:.3}", c as f32 / 255.0);
        format!(
            "{} {} {} {}",
            f(self.0),
            f(self.1),
            f(self.2),
            if stroke { "RG" } else { "rg" }
        )
    }
}

/// Uma página: a sequência de operadores do PDF.
#[derive(Default)]
pub struct Page {
    /// Operadores em bytes: o texto vai em WinAnsi, que não é UTF-8.
    ops: Vec<u8>,
}

impl Page {
    fn y(y: f32) -> f32 {
        A4_H - y
    }

    /// Texto com a linha de base em `y` (a partir do topo).
    pub fn text(&mut self, x: f32, y: f32, size: f32, font: Font, color: Rgb, s: &str) {
        let f = if font == Font::Bold { "F2" } else { "F1" };
        self.ops.extend_from_slice(
            format!(
                "BT {} /{f} {size:.2} Tf {x:.2} {:.2} Td (",
                color.ops(false),
                Self::y(y)
            )
            .as_bytes(),
        );
        self.ops.extend_from_slice(&escape(&encode(s)));
        self.ops.extend_from_slice(b") Tj ET\n");
    }

    /// Texto alinhado à direita em `x`.
    pub fn text_right(&mut self, x: f32, y: f32, size: f32, font: Font, color: Rgb, s: &str) {
        let w = text_width(s, size, font);
        self.text(x - w, y, size, font, color, s);
    }

    /// Texto que quebra em linhas de até `width`; devolve o `y` depois da última linha.
    #[allow(clippy::too_many_arguments)]
    pub fn paragraph(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        size: f32,
        font: Font,
        color: Rgb,
        s: &str,
    ) -> f32 {
        let mut y = y;
        for line in wrap(s, width, size, font) {
            self.text(x, y, size, font, color, &line);
            y += size * 1.35;
        }
        y
    }

    /// Retângulo preenchido; `y` é o topo.
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: Rgb) {
        self.push(&format!(
            "{} {x:.2} {:.2} {w:.2} {h:.2} re f",
            fill.ops(false),
            Self::y(y + h)
        ));
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Rgb) {
        self.push(&format!(
            "{} {width:.2} w {x1:.2} {:.2} m {x2:.2} {:.2} l S",
            color.ops(true),
            Self::y(y1),
            Self::y(y2)
        ));
    }

    /// Linha tracejada (estimativas).
    pub fn dashed_polyline(&mut self, pts: &[(f32, f32)], width: f32, color: Rgb) {
        self.push("[3 2] 0 d");
        self.polyline(pts, width, color);
        self.push("[] 0 d");
    }

    pub fn polyline(&mut self, pts: &[(f32, f32)], width: f32, color: Rgb) {
        if pts.len() < 2 {
            return;
        }
        let mut s = format!("{} {width:.2} w 1 J 1 j ", color.ops(true));
        for (i, (x, y)) in pts.iter().enumerate() {
            let _ = write!(
                s,
                "{x:.2} {:.2} {} ",
                Self::y(*y),
                if i == 0 { "m" } else { "l" }
            );
        }
        s.push('S');
        self.push(&s);
    }

    fn push(&mut self, line: &str) {
        self.ops.extend_from_slice(line.as_bytes());
        self.ops.push(b'\n');
    }
}

/// O documento: páginas na ordem.
#[derive(Default)]
pub struct Document {
    pub pages: Vec<Page>,
    pub title: String,
}

impl Document {
    pub fn new_page(&mut self) -> &mut Page {
        self.pages.push(Page::default());
        self.pages.last_mut().expect("página")
    }

    /// Bytes do PDF 1.4.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets: Vec<usize> = vec![];
        let n = self.pages.len();
        // 1 catálogo, 2 páginas, 3 F1, 4 F2, 5 info, depois (página, conteúdo) por página.
        let page_id = |i: usize| 6 + i * 2;
        let content_id = |i: usize| 7 + i * 2;
        let obj = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, body: Vec<u8>| {
            offsets.push(out.len());
            let id = offsets.len();
            out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
            out.extend_from_slice(&body);
            out.extend_from_slice(b"\nendobj\n");
        };
        obj(
            &mut out,
            &mut offsets,
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        );
        let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", page_id(i))).collect();
        obj(
            &mut out,
            &mut offsets,
            format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")).into_bytes(),
        );
        for name in ["Helvetica", "Helvetica-Bold"] {
            obj(
                &mut out,
                &mut offsets,
                format!(
                    "<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>"
                )
                .into_bytes(),
            );
        }
        let mut info = b"<< /Producer (Treasury) /Title (".to_vec();
        info.extend_from_slice(&escape(&encode(&self.title)));
        info.extend_from_slice(b") >>");
        obj(&mut out, &mut offsets, info);
        for (i, p) in self.pages.iter().enumerate() {
            obj(
                &mut out,
                &mut offsets,
                format!(
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {A4_W} {A4_H}] \
                     /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                    content_id(i)
                )
                .into_bytes(),
            );
            let stream = &p.ops;
            let mut body = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
            body.extend_from_slice(stream);
            body.extend_from_slice(b"\nendstream");
            obj(&mut out, &mut offsets, body);
        }
        let xref = out.len();
        out.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes(),
        );
        for off in &offsets {
            out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                offsets.len() + 1
            )
            .as_bytes(),
        );
        out
    }
}

/// Texto em WinAnsiEncoding (cp1252); o que não existe vira "?".
pub fn encode(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| match c {
            '\u{2212}' | '\u{2013}' => 0x96, // menos e meia-risca: travessão curto
            '\u{2014}' => 0x97,
            '\u{202F}' | '\u{00A0}' | '\u{2009}' => b' ',
            '\u{20AC}' => 0x80,
            '\u{2026}' => 0x85,
            '\u{2018}' => 0x91,
            '\u{2019}' => 0x92,
            '\u{201C}' => 0x93,
            '\u{201D}' => 0x94,
            '\u{2022}' => 0x95,
            '\u{203A}' => 0x9B,
            '\u{2039}' => 0x8B,
            '\u{2248}' => b'~', // ≈ não existe em WinAnsi
            c if (c as u32) < 0x80 => c as u8,
            c if (0xA0..=0xFF).contains(&(c as u32)) => c as u32 as u8,
            _ => b'?',
        })
        .collect()
}

fn escape(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for &b in bytes {
        if b == b'(' || b == b')' || b == b'\\' {
            out.push(b'\\');
        }
        out.push(b);
    }
    out
}

/// Larguras (milésimos do corpo) da Helvetica e da Helvetica-Bold para ASCII 32–126.
const HELV: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667,
    611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500,
    222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];
const HELV_BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667,
    611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556,
    278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

/// Letra-base de uma letra acentuada do Latin-1 (mesma largura na Helvetica).
fn base_letter(b: u8) -> u8 {
    match b {
        0xC0..=0xC5 => b'A',
        0xC7 => b'C',
        0xC8..=0xCB => b'E',
        0xCC..=0xCF => b'I',
        0xD1 => b'N',
        0xD2..=0xD6 | 0xD8 => b'O',
        0xD9..=0xDC => b'U',
        0xDD => b'Y',
        0xE0..=0xE5 => b'a',
        0xE7 => b'c',
        0xE8..=0xEB => b'e',
        0xEC..=0xEF => b'i',
        0xF1 => b'n',
        0xF2..=0xF6 | 0xF8 => b'o',
        0xF9..=0xFC => b'u',
        0xFD | 0xFF => b'y',
        _ => 0,
    }
}

fn byte_width(b: u8, font: Font) -> u16 {
    let table = if font == Font::Bold {
        &HELV_BOLD
    } else {
        &HELV
    };
    match b {
        32..=126 => table[(b - 32) as usize],
        0x80 | 0x96 => 556,
        0x97 => 1000,
        0x95 => 350,
        0x85 => 1000,
        0x91 | 0x92 => 222,
        0x93 | 0x94 => 333,
        0xAA => 370,
        0xBA => 365,
        0xBF => 611,
        0xA1 => 333,
        0xD7 => 584,
        0xB7 => 278,
        0xA0 => 278,
        0xAB | 0xBB => 556,
        _ => match base_letter(b) {
            0 => 556,
            l => table[(l - 32) as usize],
        },
    }
}

/// Largura do texto em pontos.
pub fn text_width(s: &str, size: f32, font: Font) -> f32 {
    encode(s)
        .iter()
        .map(|b| byte_width(*b, font) as f32)
        .sum::<f32>()
        * size
        / 1000.0
}

/// Corta o texto com "…" para caber em `width`.
pub fn fit(s: &str, width: f32, size: f32, font: Font) -> String {
    if text_width(s, size, font) <= width {
        return s.to_string();
    }
    let mut out = String::new();
    for c in s.chars() {
        let next = format!("{out}{c}…");
        if text_width(&next, size, font) > width {
            break;
        }
        out.push(c);
    }
    format!("{}…", out.trim_end())
}

/// Quebra em linhas de até `width`.
pub fn wrap(s: &str, width: f32, size: f32, font: Font) -> Vec<String> {
    let mut lines = vec![];
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let next = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if text_width(&next, size, font) > width && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        } else {
            cur = next;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_portuguese_and_spanish_in_winansi() {
        assert_eq!(encode("ção"), vec![0xE7, 0xE3, b'o']);
        assert_eq!(encode("¿Año?"), vec![0xBF, b'A', 0xF1, b'o', b'?']);
        assert_eq!(
            encode("\u{2212}R$ 1\u{202F}000 €"),
            b"\x96R$ 1 000 \x80".to_vec()
        );
        assert_eq!(encode("日"), b"?".to_vec());
    }

    #[test]
    fn widths_wrap_and_fit() {
        assert!((text_width("AAAA", 10.0, Font::Regular) - 26.68).abs() < 0.01);
        assert_eq!(
            text_width("é", 10.0, Font::Regular),
            text_width("e", 10.0, Font::Regular)
        );
        let lines = wrap("um dois três quatro cinco seis", 60.0, 10.0, Font::Regular);
        assert!(lines.len() > 1);
        assert!(
            lines
                .iter()
                .all(|l| text_width(l, 10.0, Font::Regular) <= 60.0)
        );
        let f = fit(
            "Alimentação e bebidas fora de casa",
            80.0,
            10.0,
            Font::Regular,
        );
        assert!(f.ends_with('…') && text_width(&f, 10.0, Font::Regular) <= 80.0);
    }

    #[test]
    fn a_document_is_a_valid_pdf_that_lopdf_reads() {
        let mut d = Document {
            title: "Relatório (teste)".into(),
            ..Default::default()
        };
        let p = d.new_page();
        p.text(
            40.0,
            60.0,
            18.0,
            Font::Bold,
            Rgb(10, 20, 30),
            "Relatório de outubro",
        );
        p.rect(40.0, 80.0, 100.0, 10.0, Rgb(200, 100, 50));
        p.polyline(
            &[(40.0, 200.0), (100.0, 150.0), (160.0, 180.0)],
            1.5,
            Rgb(0, 0, 0),
        );
        d.new_page()
            .text(40.0, 60.0, 10.0, Font::Regular, Rgb(0, 0, 0), "Página 2");
        let bytes = d.to_bytes();
        let doc = lopdf::Document::load_mem(&bytes).expect("PDF válido");
        assert_eq!(doc.get_pages().len(), 2);
        let text = doc.extract_text(&[1]).expect("texto");
        assert!(text.contains("Relat"), "{text}");
    }
}

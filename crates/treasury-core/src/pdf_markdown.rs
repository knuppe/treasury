//! PDF em Markdown para os agentes (docs/pdf-markdown.md): títulos, parágrafos e tabelas em GFM,
//! montados pela posição de cada caractere que o `pdf-extract` já lê (a mesma leitura dos
//! leitores de extrato e nota). Nenhum caractere é inventado nem perdido: cada página passa por
//! uma conferência (as letras e os números do Markdown são os mesmos da página) e, se não bater,
//! a página sai como texto corrido. Cabeçalho e rodapé repetidos em todas as páginas e números
//! de página saem; cada página começa com `<!-- página N -->`.
//!
//! Tabelas: linhas seguidas com dois ou mais trechos separados por um espaço largo e com o mesmo
//! número de colunas viram tabela; as colunas vêm do alinhamento dos trechos. PDF sem texto (só
//! imagem) vai para o OCR local (`crate::ocr`), em parágrafos.

use std::collections::HashMap;

use pdf_extract::{MediaBox, OutputDev, OutputError, Transform};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Espaço entre caracteres (em tamanhos da fonte) acima do qual entra um espaço entre palavras,
/// quando o PDF não escreve os espaços (só desloca o texto).
const WORD_GAP: f64 = 0.2;
/// Idem quando o PDF escreve os espaços: o deslocamento é só ajuste de letras (kerning), até
/// mais de meia letra ("V alor").
const WORD_GAP_EXPLICIT: f64 = 0.8;
/// Espaço acima do qual o trecho seguinte é outra coluna.
/// Entre números: separa a partir daqui (o ajuste de "11" fica abaixo).
const DIGIT_GAP: f64 = 0.4;
const CELL_GAP: f64 = 1.2;
const CELL_GAP_EXPLICIT: f64 = 1.0;
/// Mínimo de linhas seguidas para virar tabela.
const MIN_TABLE_ROWS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MdEngine {
    /// Pela posição do texto (títulos, parágrafos e tabelas).
    Layout,
    /// Texto corrido (a conversão não conferiu ou falhou).
    Plain,
    /// OCR local (PDF só com imagem): pode ter erros.
    Ocr,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MdDocument {
    /// O Markdown de cada página, já com o marcador `<!-- página N -->`.
    pub pages: Vec<String>,
    pub engine: MdEngine,
    /// Páginas que saíram como texto corrido (a conferência não bateu).
    pub plain_pages: Vec<u32>,
}

impl MdDocument {
    pub fn markdown(&self) -> String {
        self.pages.join("\n\n")
    }
}

#[derive(Debug, Clone)]
struct Glyph {
    x: f64,
    /// De cima para baixo.
    y: f64,
    end: f64,
    size: f64,
    text: String,
}

#[derive(Default)]
struct Collector {
    pages: Vec<Vec<Glyph>>,
    /// Altura de cada página (para contar `y` de cima para baixo e achar as margens).
    heights: Vec<f64>,
    height: f64,
}

impl OutputDev for Collector {
    fn begin_page(
        &mut self,
        _n: u32,
        media_box: &MediaBox,
        _art: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), OutputError> {
        self.height = media_box.ury - media_box.lly;
        self.heights.push(self.height);
        self.pages.push(vec![]);
        Ok(())
    }
    fn end_page(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn output_character(
        &mut self,
        trm: &Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        ch: &str,
    ) -> Result<(), OutputError> {
        // O mesmo cálculo do texto corrido do `pdf-extract` (tamanho pela área do quadrado).
        let vx = font_size * (trm.m11 + trm.m21);
        let vy = font_size * (trm.m12 + trm.m22);
        let size = (vx * vy).abs().sqrt().max(0.1);
        let (x, y) = (trm.m31, self.height - trm.m32);
        if let Some(page) = self.pages.last_mut() {
            page.push(Glyph {
                x,
                y,
                end: x + width * size,
                size,
                text: ch.to_string(),
            });
        }
        Ok(())
    }
    fn begin_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn end_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn end_line(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
}

/// Um trecho de texto contínuo numa linha (uma célula, numa tabela).
#[derive(Debug, Clone)]
struct Segment {
    x0: f64,
    x1: f64,
    text: String,
}

#[derive(Debug, Clone)]
struct Line {
    y: f64,
    size: f64,
    segments: Vec<Segment>,
}

impl Line {
    fn text(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Junta os caracteres em linhas e trechos, pela posição.
fn lines_of(mut glyphs: Vec<Glyph>) -> Vec<Line> {
    glyphs.retain(|g| !g.text.trim().is_empty() || g.text == " ");
    // O PDF escreve os espaços (a maioria dos geradores): as palavras vêm deles.
    // Fonte sem as larguras (alguns PDFs gerados por navegador): a largura média da página.
    let typical = median(
        glyphs
            .iter()
            .filter(|g| g.end - g.x > g.size * 0.05)
            .map(|g| (g.end - g.x) / g.size)
            .collect(),
    );
    let typical = if typical > 0.0 { typical } else { 0.5 };
    for g in &mut glyphs {
        if g.end - g.x <= g.size * 0.05 {
            g.end = g.x + typical * g.size;
        }
    }
    let spaces = glyphs.iter().filter(|g| g.text == " ").count();
    let explicit = spaces * 20 >= glyphs.len().max(1);
    let (word_gap, cell_gap) = if explicit {
        (WORD_GAP_EXPLICIT, CELL_GAP_EXPLICIT)
    } else {
        (WORD_GAP, CELL_GAP)
    };
    // Linhas: a mesma altura (meia fonte de folga), de cima para baixo.
    glyphs.sort_by(|a, b| a.y.total_cmp(&b.y));
    let mut rows: Vec<Vec<Glyph>> = vec![];
    for g in glyphs {
        match rows.last_mut() {
            Some(r) if (r[0].y - g.y).abs() <= r[0].size.max(g.size) * 0.5 => r.push(g),
            _ => rows.push(vec![g]),
        }
    }
    rows.into_iter()
        .map(|mut r| {
            r.sort_by(|a, b| a.x.total_cmp(&b.x));
            let size = median(r.iter().map(|g| g.size).collect());
            let y = r[0].y;
            let mut segments: Vec<Segment> = vec![];
            let mut last_end = f64::NEG_INFINITY;
            for g in r {
                if g.text == " " {
                    // Espaço explícito: só marca a palavra (a largura decide a coluna).
                    if let Some(s) = segments.last_mut()
                        && !s.text.ends_with(' ')
                    {
                        s.text.push(' ');
                    }
                    last_end = last_end.max(g.end);
                    continue;
                }
                let gap = g.x - last_end;
                match segments.last_mut() {
                    Some(s) if gap < size * cell_gap => {
                        // Números nunca grudam: entre dígitos (ou dígito e sinal), qualquer
                        // espaço visível separa ("50 44,00", não "5044,00").
                        let digit = s.text.chars().last().is_some_and(|c| c.is_ascii_digit())
                            || g.text.starts_with(|c: char| c.is_ascii_digit() || c == '-');
                        let limit = if digit { DIGIT_GAP } else { word_gap };
                        if gap > size * limit && !s.text.ends_with(' ') {
                            s.text.push(' ');
                        }
                        s.text.push_str(&g.text);
                        s.x1 = g.end;
                    }
                    _ => segments.push(Segment {
                        x0: g.x,
                        x1: g.end,
                        text: g.text.clone(),
                    }),
                }
                last_end = g.end;
            }
            for s in &mut segments {
                s.text = s.text.split_whitespace().collect::<Vec<_>>().join(" ");
            }
            segments.retain(|s| !s.text.is_empty());
            Line { y, size, segments }
        })
        .filter(|l| !l.segments.is_empty())
        .collect()
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

/// Texto de cabeçalho ou rodapé com os números trocados (para achar o que se repete).
fn shape(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_digit() { '#' } else { c })
        .collect::<String>()
        .to_lowercase()
}

/// Número de página sozinho ("3", "Página 3 de 10", "3/10", "- 3 -").
fn page_number(s: &str) -> bool {
    let t = s.trim().trim_matches('-').trim().to_lowercase();
    let t = t
        .strip_prefix("página")
        .or_else(|| t.strip_prefix("pagina"))
        .or_else(|| t.strip_prefix("page"))
        .or_else(|| t.strip_prefix("pág."))
        .unwrap_or(&t)
        .trim()
        .to_string();
    !t.is_empty()
        && t.chars().all(|c| {
            c.is_ascii_digit()
                || c == '/'
                || c == ' '
                || c == 'd'
                || c == 'e'
                || c == 'o'
                || c == 'f'
        })
        && t.chars().any(|c| c.is_ascii_digit())
        && t.split_whitespace().count() <= 3
}

/// Fração da altura da página que conta como margem de cima ou de baixo.
const MARGIN: f64 = 0.1;

/// Linhas a tirar de cada página: cabeçalho e rodapé repetidos (a partir da segunda página) e
/// números de página, só nas margens de cima e de baixo (o conteúdo repetido fica).
fn running_lines(pages: &[Vec<Line>], heights: &[f64]) -> Vec<Vec<bool>> {
    let in_margin = |pi: usize, l: &Line| {
        let h = heights.get(pi).copied().unwrap_or(0.0);
        h > 0.0 && (l.y < h * MARGIN || l.y > h * (1.0 - MARGIN))
    };
    let mut seen: HashMap<String, usize> = HashMap::new();
    for (pi, p) in pages.iter().enumerate() {
        let mut edges: Vec<String> = vec![];
        for l in p.iter().filter(|l| in_margin(pi, l)) {
            edges.push(shape(&l.text()));
        }
        edges.sort();
        edges.dedup();
        for e in edges {
            *seen.entry(e).or_default() += 1;
        }
    }
    let n = pages.len();
    pages
        .iter()
        .enumerate()
        .map(|(pi, p)| {
            p.iter()
                .map(|l| {
                    let edge = in_margin(pi, l);
                    let text = l.text();
                    if edge && page_number(&text) {
                        return true;
                    }
                    edge && pi > 0
                        && n >= 3
                        && seen.get(&shape(&text)).copied().unwrap_or(0) * 3 >= n * 2
                })
                .collect()
        })
        .collect()
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

/// Colunas pelas bordas dos trechos das linhas com o número de colunas da tabela.
fn table_markdown(lines: &[&Line], cols: usize) -> String {
    let full: Vec<&&Line> = lines.iter().filter(|l| l.segments.len() == cols).collect();
    let anchors: Vec<(f64, f64)> = (0..cols)
        .map(|c| {
            (
                median(full.iter().map(|l| l.segments[c].x0).collect()),
                median(full.iter().map(|l| l.segments[c].x1).collect()),
            )
        })
        .collect();
    let mut rows: Vec<Vec<String>> = vec![];
    for l in lines {
        let mut row = vec![String::new(); cols];
        for s in &l.segments {
            // A coluna que mais se sobrepõe (ou a mais perto).
            let best = (0..cols)
                .max_by(|&a, &b| {
                    let score = |c: usize| {
                        let (a0, a1) = anchors[c];
                        let overlap = s.x1.min(a1) - s.x0.max(a0);
                        if overlap > 0.0 {
                            overlap
                        } else {
                            overlap.min(0.0) - (s.x0 - a0).abs() * 1e-3
                        }
                    };
                    score(a).total_cmp(&score(b))
                })
                .unwrap_or(0);
            if !row[best].is_empty() {
                row[best].push(' ');
            }
            row[best].push_str(&s.text);
        }
        rows.push(row);
    }
    let mut out = String::new();
    for (i, r) in rows.iter().enumerate() {
        out.push_str("| ");
        out.push_str(&r.iter().map(|c| cell(c)).collect::<Vec<_>>().join(" | "));
        out.push_str(" |\n");
        if i == 0 {
            out.push_str(&format!("|{}\n", " --- |".repeat(cols)));
        }
    }
    out.trim_end().to_string()
}

/// A página em Markdown (sem o marcador).
fn page_markdown(lines: &[&Line], body_size: f64) -> String {
    let mut blocks: Vec<String> = vec![];
    let mut i = 0;
    while i < lines.len() {
        // Tabela: linhas seguidas com o mesmo número (≥ 2) de trechos.
        let cols = lines[i].segments.len();
        if cols >= 2 {
            let mut j = i;
            while j < lines.len() && lines[j].segments.len() >= 2 {
                j += 1;
            }
            let block = &lines[i..j];
            let mut counts: HashMap<usize, usize> = HashMap::new();
            for l in block {
                *counts.entry(l.segments.len()).or_default() += 1;
            }
            let (mode, freq) = counts
                .into_iter()
                .max_by_key(|(k, v)| (*v, *k))
                .unwrap_or((cols, 0));
            if block.len() >= MIN_TABLE_ROWS && freq * 2 >= block.len() {
                blocks.push(table_markdown(block, mode));
                i = j;
                continue;
            }
        }
        let l = lines[i];
        let text = l.text();
        // Título: fonte bem maior que a do corpo, linha curta.
        if body_size > 0.0 && l.size >= body_size * 1.25 && text.chars().count() <= 120 {
            let level = if l.size >= body_size * 1.7 { "#" } else { "##" };
            blocks.push(format!("{level} {text}"));
            i += 1;
            continue;
        }
        // Parágrafo: junta as linhas próximas do mesmo tamanho.
        let mut para = text;
        let mut prev = l;
        i += 1;
        while i < lines.len() {
            let n = lines[i];
            let close = (n.y - prev.y) <= prev.size.max(n.size) * 1.8;
            let same = (n.size - prev.size).abs() <= prev.size * 0.15;
            if !close || !same || n.segments.len() >= 2 || n.size >= body_size * 1.25 {
                break;
            }
            para.push(' ');
            para.push_str(&n.text());
            prev = n;
            i += 1;
        }
        blocks.push(para);
    }
    blocks.join("\n\n")
}

/// As letras e os números do texto, sem espaços nem sinais do Markdown (para a conferência).
fn fingerprint(s: &str) -> Vec<char> {
    let mut v: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).collect();
    v.sort_unstable();
    v
}

fn marker(n: usize) -> String {
    format!("<!-- página {n} -->")
}

/// O PDF em Markdown (com a senha, se protegido). Sem texto (só imagem): o OCR local.
pub fn markdown(bytes: &[u8], password: Option<&str>) -> Result<MdDocument, String> {
    markdown_with(bytes, password, true)
}

/// Como [`markdown`]; sem `ocr`, um PDF só com imagem volta sem texto (quem chama decide).
pub fn markdown_with(
    bytes: &[u8],
    password: Option<&str>,
    ocr: bool,
) -> Result<MdDocument, String> {
    let b = crate::pdf::sanitize(bytes).to_vec();
    let pw = password.filter(|p| !p.is_empty()).map(str::to_string);
    let collected = std::panic::catch_unwind(move || -> Result<Collector, String> {
        let mut doc = pdf_extract::Document::load_mem(&b).map_err(|e| e.to_string())?;
        let mut c = Collector::default();
        match pw {
            Some(p) => pdf_extract::output_doc_encrypted(&mut doc, &mut c, &p),
            None => {
                if doc.is_encrypted() {
                    let _ = doc.decrypt("");
                }
                pdf_extract::output_doc(&doc, &mut c)
            }
        }
        .map_err(|e| e.to_string())?;
        Ok(c)
    })
    .map_err(|_| crate::i18n::t("nucleo-base-erro-pdf"))??;
    let alnum: usize = collected
        .pages
        .iter()
        .flatten()
        .map(|g| g.text.chars().filter(|c| c.is_alphanumeric()).count())
        .sum();
    // Pouco ou nenhum texto: é imagem (o OCR lê, se pedido).
    if alnum < 40 && ocr {
        return ocr_markdown(bytes);
    }
    if alnum == 0 {
        return Ok(MdDocument {
            pages: vec![],
            engine: MdEngine::Plain,
            plain_pages: vec![],
        });
    }
    let heights = collected.heights.clone();
    let pages: Vec<Vec<Line>> = collected.pages.into_iter().map(lines_of).collect();
    let body = median(
        pages
            .iter()
            .flatten()
            .flat_map(|l| std::iter::repeat_n(l.size, l.text().len()))
            .collect(),
    );
    let drop = running_lines(&pages, &heights);
    let mut out = MdDocument {
        pages: vec![],
        engine: MdEngine::Layout,
        plain_pages: vec![],
    };
    for (pi, p) in pages.iter().enumerate() {
        let kept: Vec<&Line> = p
            .iter()
            .zip(&drop[pi])
            .filter(|(_, d)| !**d)
            .map(|(l, _)| l)
            .collect();
        let md = page_markdown(&kept, body);
        // Conferência: as mesmas letras e números das linhas mantidas.
        let expected: String = kept.iter().map(|l| l.text()).collect::<Vec<_>>().join(" ");
        let body_md = if fingerprint(&md) == fingerprint(&expected) {
            md
        } else {
            out.plain_pages.push(pi as u32 + 1);
            kept.iter().map(|l| l.text()).collect::<Vec<_>>().join("\n")
        };
        out.pages
            .push(format!("{}\n\n{}", marker(pi + 1), body_md.trim()));
    }
    Ok(out)
}

/// PDF só com imagem: o texto do OCR local, em parágrafos, página a página.
fn ocr_markdown(bytes: &[u8]) -> Result<MdDocument, String> {
    let o = crate::ocr::read_scanned(bytes).map_err(|e| e.to_string())?;
    let text = o.text();
    Ok(MdDocument {
        pages: vec![format!("{}\n\n{}", marker(1), text.trim())],
        engine: MdEngine::Ocr,
        plain_pages: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(x: f64, y: f64, s: &str) -> Vec<Glyph> {
        // Um caractere por letra, 5 pontos cada, fonte 10.
        s.chars()
            .enumerate()
            .map(|(i, c)| Glyph {
                x: x + i as f64 * 5.0,
                y,
                end: x + i as f64 * 5.0 + 5.0,
                size: 10.0,
                text: c.to_string(),
            })
            .collect()
    }

    #[test]
    fn columns_become_a_table_and_text_stays_whole() {
        let mut glyphs = vec![];
        glyphs.extend(g(10.0, 10.0, "Banco Exemplo"));
        for (i, (d, desc, v)) in [
            ("Data", "Descrição", "Valor"),
            ("01/10", "PADARIA BOM PAO", "-23,90"),
            ("02/10", "PIX RECEBIDO", "1.250,00"),
            ("03/10", "LUZ", "-187,44"),
        ]
        .iter()
        .enumerate()
        {
            let y = 40.0 + i as f64 * 12.0;
            glyphs.extend(g(10.0, y, d));
            glyphs.extend(g(80.0, y, desc));
            glyphs.extend(g(200.0, y, v));
        }
        // Espaço de ajuste dentro da palavra (kerning) não quebra a célula.
        let mut kern = g(10.0, 100.0, "Saldo");
        for k in kern.iter_mut().skip(1) {
            k.x += 1.5;
            k.end += 1.5;
        }
        glyphs.extend(kern);
        let lines = lines_of(glyphs);
        let refs: Vec<&Line> = lines.iter().collect();
        let md = page_markdown(&refs, 10.0);
        assert!(md.contains("| Data | Descrição | Valor |"), "{md}");
        assert!(md.contains("| 02/10 | PIX RECEBIDO | 1.250,00 |"), "{md}");
        assert!(md.contains("Banco Exemplo"));
        assert!(md.contains("Saldo"), "{md}");
        let all: String = refs.iter().map(|l| l.text()).collect();
        assert_eq!(fingerprint(&md), fingerprint(&all));
    }

    #[test]
    fn page_numbers_and_repeated_headers_leave() {
        assert!(page_number("Página 3 de 10"));
        assert!(page_number("3/10"));
        assert!(page_number("- 2 -"));
        assert!(!page_number("Total 3"));
        let page = |n: usize| {
            let mut v = vec![];
            v.extend(g(10.0, 5.0, "Fundo Exemplo · Extrato"));
            v.extend(g(10.0, 300.0, &format!("Linha de conteudo {n}")));
            v.extend(g(10.0, 800.0, &format!("{n}")));
            lines_of(v)
        };
        let pages = vec![page(1), page(2), page(3)];
        let drop = running_lines(&pages, &[842.0; 3]);
        assert_eq!(drop[0], [false, false, true]);
        assert_eq!(drop[1], [true, false, true]);
        assert_eq!(drop[2], [true, false, true]);
    }
}

/// Conferência manual: `PDF_MD_DIR=pasta cargo test -p treasury-core --lib -- --ignored
/// pdf_markdown::manual` grava `<arquivo>.md` ao lado de cada PDF da pasta.
#[cfg(test)]
mod manual {
    #[test]
    #[ignore]
    fn converts_the_pdfs_of_a_folder() {
        let Ok(dir) = std::env::var("PDF_MD_DIR") else {
            return;
        };
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().and_then(|x| x.to_str()) != Some("pdf") {
                continue;
            }
            let bytes = std::fs::read(&p).unwrap();
            let t = std::time::Instant::now();
            let d = super::markdown(&bytes, None).unwrap();
            let md = d.markdown();
            println!(
                "{} · {:?} · {} chars · {:?} · texto corrido em {:?}",
                p.display(),
                d.engine,
                md.len(),
                t.elapsed(),
                d.plain_pages
            );
            std::fs::write(p.with_extension("treasury.md"), md).unwrap();
        }
    }
}

/// Um documento lido para o agente, em partes (`import.read_document`).
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ReadDocumentArgs {
    pub filename: String,
    pub content_base64: String,
    /// Senha do PDF protegido (não é guardada).
    #[serde(default)]
    #[ts(optional)]
    pub password: Option<String>,
    /// Caractere de onde começar (padrão 0).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub offset: Option<usize>,
    /// Quantos caracteres (padrão 20 mil, até 40 mil).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentMarkdown {
    pub filename: String,
    /// Sempre `markdown`.
    pub format: String,
    pub engine: MdEngine,
    #[ts(type = "number")]
    pub pages: usize,
    /// Páginas que saíram como texto corrido.
    pub plain_pages: Vec<u32>,
    /// O trecho pedido.
    pub markdown: String,
    #[ts(type = "number")]
    pub offset: usize,
    /// Onde continuar (`None`: acabou).
    #[ts(optional, type = "number")]
    pub next_offset: Option<usize>,
    #[ts(type = "number")]
    pub total_chars: usize,
}

/// `import.read_document`: o PDF (ou texto) em Markdown, em partes.
pub fn read_document(a: &ReadDocumentArgs) -> crate::error::Result<DocumentMarkdown> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(a.content_base64.trim())
        .map_err(|_| {
            crate::error::Error::invalid(crate::i18n::t("nucleo-importacao-erro-base64-curto"))
        })?;
    let (doc, engine, plain) = if bytes.starts_with(b"%PDF") {
        let d = markdown(&bytes, a.password.as_deref()).map_err(crate::error::Error::invalid)?;
        let pages = d.pages.len();
        (d.markdown(), d.engine, (pages, d.plain_pages))
    } else {
        match String::from_utf8(bytes) {
            Ok(text) => (text, MdEngine::Plain, (1, vec![])),
            Err(_) => {
                return Err(crate::error::Error::invalid(crate::i18n::t(
                    "nucleo-documento-nao-pdf",
                )));
            }
        }
    };
    let (piece, next_offset, total_chars) =
        crate::research::doc_read::chunk(&doc, a.offset.unwrap_or(0), a.limit.unwrap_or(20_000));
    Ok(DocumentMarkdown {
        filename: a.filename.clone(),
        format: "markdown".into(),
        engine,
        pages: plain.0,
        plain_pages: plain.1,
        markdown: piece,
        offset: a.offset.unwrap_or(0).min(total_chars),
        next_offset,
        total_chars,
    })
}

# Receipts by photo (OCR)

Photos of receipts with no readable QR, and receipts from any country, are read on the computer: prepare the image, read the QR, local OCR, a line reader with **arithmetic checking** and, only for what did not add up and if the person asks, the AI. The result opens the manual purchase in Consumption already filled in, and nothing is saved without review. Reading the NFC-e (Brazilian electronic retail receipt) through the QR is in [receipts.md](receipts.md); the screen is in [consumption.md](consumption.md). Examples use made-up data.

## Pipeline

```
foto/PDF ─► preparar ─► QR? ─► consulta (BR) ──ok──► prévia
                │         │        └─captcha─► janela do app (pessoa resolve) ─► prévia
                │         └─dados do QR (chave, total, data, IVA) ─┐
                └─► OCR local (motor escolhido) ─► linhas + caixas ─┤
                                                                     ▼
                    perfil do leiaute ─► leitor de linhas ─► conferência ─► fechou? ─► prévia
                                                                     │ não
                                                                     ▼
                                     IA (texto das linhas duvidosas; imagem só se permitido)
                                                                     ▼
                                                  mesma conferência ─► prévia com destaques
```

1. **Prepare the image** (in the core, in Rust, `image` crate): rotate by EXIF, find the paper
   (on the Mac, `VNDetectDocumentSegmentationRequest`; elsewhere, the outline of the largest
   light rectangle), straighten, grayscale, local contrast. Reduce to at most ~2000 px on the
   shorter side of the text column. A very long receipt is cut into overlapping strips, so as
   not to exhaust memory.
2. **QR**: the NFC-e QR reader. In Brazil, the access key gives the state (UF), the CNPJ, the
   model, the series, the number and the month. The contingency (offline) QR also carries the day
   and the total amount. In Portugal (ATCUD), Spain (VeriFactu/TicketBAI) and Germany (TSE) the QR
   carries the total and the tax by rate. This is the OCR's **answer key**: the sum read has to
   match.
3. **OCR**: the configured engine returns lines with text, box (position) and confidence.
4. **Profile and line reader**: sections 5 and 6.
5. **Check**: see "Line reader". Only what added up goes in without review.
6. **AI**: see "AI help".

**Pasted text**: "Paste the items as text" (a receipt that arrived by email, order history,
spreadsheet) goes through the same line reader and the same check, with no OCR
(`receipts.read_text`). One item per line, with the amount at the end.

## OCR engines

| Engine | Where | License | Does the image leave the computer? |
|---|---|---|---|
| **Apple Vision** (`VNRecognizeTextRequest`, accurate level, no language correction) | macOS, in `src-tauri`; the core receives the finished lines, with the boxes | the system's | No |
| **PaddleOCR** (PP-OCRv3 detection and Latin PP-OCRv5 recognition) in `tract`, pure Rust, with the models embedded in `assets/ocr/` (feature `ocr-paddle`, on by default) | All; on the Mac, when Vision does not add up | Apache-2.0 | No |
| **AI with image** (an engine in Settings › AI and agent that reads images) | All | the engine's | Only if the engine is remote, and the app warns |

- One receipt at a time; the image is reduced before OCR and a long receipt goes in strips.
- Cheap order first: QR → lookup → local OCR → AI. The AI only runs on what is left and if the
  person asked.

## Captcha in the SEFAZ lookup

- **What not to do:** solve the captcha by program, use a solving service or change IP to get
  around the limit. It is against the SEFAZ terms (SEFAZ are the state tax authorities) and
  breaks easily.
- **Lookup window in the app.** Tauri opens its own window with the QR URL (only `.gov.br`
  domains, only https). The person solves the captcha there, as they would in the browser. When
  the result page loads, the app reads that page's HTML (read-only, with the same reader as
  today) and closes the window. The "open in the browser, copy and paste" step goes away. The
  window's session is discarded on close.
- **Less captcha.** One lookup at a time, spaced out, without repeating the same key; a receipt
  already read (by key) is not looked up again. Some lookups only ask for a captcha after many
  requests in a row.
- **When the lookup fails** (down, contingency, captcha refused): OCR of the photo, checked
  against what the QR and the key carry (CNPJ, number, month; total and day in the offline QR).
  The receipt is marked to look up again later, in the background, if the person wants.
- **Measure by state.** The test bench records, per state, whether the lookup opened, asked for a
  captcha or failed, to know where it is worth investing.

## Layout profiles (design, not implemented yet)

Receipts vary by country, by chain and, in Brazil, by the **POS software**. The NFC-e DANFE
(the printed receipt) has fixed divisions in the SEFAZ manual (header, items, totals, payment,
key, QR, consumer, messages), but each software breaks the item line in its own way,
abbreviates differently and puts the columns in different places.

A **profile** is a small data file, embedded in the app and tested, in the style of
`income/payslip_specs.rs`:

```toml
id = "br-item-duas-linhas"
pais = "BR"
detecta = ["DANFE NFC-e", "Documento Auxiliar da Nota Fiscal de Consumidor"]
assinatura = ["<nome do software no rodapé>"]       # quando o PDV imprime
item = ["{seq} {codigo} {descricao}", "{qtd} {un} X {unit} {total}"]
numero = "virgula"                                  # 1.234,56
desconto = ["DESCONTO", "DESC."]
total = ["VALOR TOTAL R$", "VALOR A PAGAR R$"]
quantidade_itens = "QTD. TOTAL DE ITENS"
```

- **Detection:** words from the header and footer (many POS systems print the software name), the
  store's CNPJ and the country. With no profile recognized, the country's generic profile
  applies.
- **Small start:** one generic per country (BR, PT, ES, FR, DE, GB, US, CA, AR, CL) and the
  profiles of the most common chains and software.
- **Local learning:** when the person corrects a receipt, the app stores per-store adjustments
  (CNPJ/NIF or name): item on one or two lines, total column, abbreviations. It stays in the
  vault and never leaves it.
- **Tests:** each profile has fixtures with the **text** of made-up receipts (like those in
  `payslip_specs`). An image of a real receipt never goes into the repository.

## Line reader

### Lines

The OCR boxes become lines by height (same y, with tolerance for tilt) and columns by x position.
A two-line item is joined by the profile.

### Normalization

- Only in numeric fields: `O→0`, `o→0`, `l/I/|→1`, `S→5`, `B→8`, `Z→2`.
- Decimal separator by country and profile (BR, PT, ES, FR, DE, AR: comma; US, GB, CA: period;
  CL: peso with no cents, with a thousands period).
- Units: `UN`, `KG`, `G`, `L`, `ML`, `PC`, `lb`, `oz`, `Stk`; variable weight
  (`0,734 KG X 39,90`).
- Lines that are not items: change, payment, tax, "Pfand", loyalty coupon.

### Check

| Rule | Where |
|---|---|
| `quantity × unit price ≈ total` of the line (rounding tolerance; 3 decimal places on weight) | All |
| `Σ items − discounts + surcharges = total` | All |
| `Σ items − discounts = subtotal` and `subtotal + tax = total` (the tax is added at the end; both sums working out count as the second check; a tax line the OCR missed comes from total − subtotal) | US, CA |
| `Σ payments − change = total` | All |
| Number of items = "Qtd. total de itens" | BR |
| Total and date = those of the QR (offline) and the key checks out (check digit) | BR |
| Total and VAT by rate = those of the QR | PT, ES, DE |
| Tax by rate (A/B in Germany, VAT/TVA by rate) matches the sum of the lines | DE, FR, PT, ES |

Each line ends up **checked**, **doubtful** or **not read**. When only the sum does not add up,
the reader tries likely swaps in the low-confidence digits (8↔6↔0, 1↔7, 5↔6) until it adds up,
with a limit on attempts. If a swap works, the line is marked "corrected by the sum", and the
preview shows the highlight. A receipt where everything adds up goes in without review; the rest
opens the preview with the doubtful lines highlighted next to the crop of the photo.

## AI help

- **Text first.** The AI receives the OCR lines with positions, the detected profile and what did
  not add up, with CPF/NIF masked. JSON in the `ReceiptPreview` format is requested. That is
  hundreds of tokens, so it fits a free engine (Groq) or a local one (Qwen 3 4B through an
  endpoint).
- **Image only if allowed**, cropped to the items area, with the consumer area blurred. A local
  vision model (Ollama/LM Studio) when the machine can handle it; otherwise the chosen remote
  engine.
- **The same check.** What the AI returns goes through the same check as the line reader. If it
  does not add up, it stays doubtful. The AI never saves: it becomes a proposal.
- **No local OCR** (machine with no engine at all): the AI with image does everything, but the
  result goes through the check.

## By country

| Country | Source of the items | Extra check |
|---|---|---|
| Brazil | SEFAZ lookup, XML, PDF; OCR when the lookup fails | Key, offline QR, number of items |
| Portugal | OCR | ATCUD QR (total, VAT by rate, NIF) |
| Spain | OCR | VeriFactu/TicketBAI QR (total) |
| Germany | OCR | QR/TSE and A/B tax |
| France | OCR | TVA by rate |
| United Kingdom, US, Canada | OCR | Subtotal, tax (sales tax/HST) and total |
| Argentina, Chile | OCR | Total; itemized VAT when present |

## Test bench

The real photos used to measure the reader stay outside the repository, in a private folder, with
each one's answer key written by hand; the repository keeps only fixtures with the **text** of
made-up receipts. Measured with accurate Apple Vision, the item text was found in 84% of the
legible receipts, the total came out right in 68% and the sum worked out by itself in 34% (with
the right total in 91% of those that worked out), at ~74 ms per photo; Vision's fast mode is not
suitable for receipts. Latin PaddleOCR reached 75% right totals on the same bench.

## How it is implemented

**Core** (`receipts/ocr/`, no new migration):

- `lines.rs`: the prototype's line reader, in Rust, with the same result on the test bench
  (Vision 68% right total, Latin PaddleOCR 75%). It returns the items, the discount, the total,
  the printed item count and the band: `checked`, `balanced` (added up) or `unbalanced` (did not
  add up).
- `paddle.rs` (feature `ocr-paddle`, on by default): PP-OCRv3 detection and Latin PP-OCRv5
  recognition in `tract` (pure Rust), with the models embedded in `assets/ocr/` (Apache-2.0).
  A single binary, no native library.
- `mod.rs`: `receipts.read_photo` (outside the database lock). It uses Vision's lines when the
  app sends them; PaddleOCR reads again when Vision does not add up (and alone outside the Mac);
  the AI only if the person asked and the sum did not add up, through the "PDF reading" route
  (falls back to the chat's), only with an engine that reads images, with no tools, and the
  answer goes through the same check. It returns the finished purchase (`ManualReceipt` with
  `from_photo`, saved with origin `text`), the date and store found in the text and the warnings.

**App** (`src-tauri`):

- `read_text_native`: Apple Vision (`VNRecognizeTextRequest`, accurate, no language correction),
  with the boxes and corners of each line.
- `sefaz_open` / `sefaz_capture`: the NFC-e lookup in an app window (only `https` on `.gov.br`,
  anonymous session that disappears on close, with no permissions). The person solves the captcha
  there; when the page looks like a receipt ("Valor a pagar", "Qtd. total de itens"), the text
  comes back through the intercepted navigation `treasury-sefaz://` and goes through the usual
  text reader. The "Read the page now" button forces the reading.

  The frames of the captcha services (Cloudflare Turnstile, which the SC SEFAZ uses; reCAPTCHA;
  hCaptcha) and empty frames (`about:blank`) also load: WKWebView passes each frame's navigation
  through the `.gov.br` rule, and without this exception the captcha did not appear and the SEFAZ
  answered "Não foi possível validar o CAPTCHA" (2026-10-22). The main page stays restricted to
  `.gov.br`.

**UI** (Consumption › Purchases › Read receipt):

- Brazil: the photo tries the QR first; without a QR, it reads the text. The "Open the lookup
  here" button opens the SEFAZ window.
- Other countries: the button opens only the reading by photo.
- The photo read opens the manual purchase already filled in (date, store, items, discount), with
  the confidence band, the printed total and how much is missing or extra, and the "Ask the AI
  for help" button when the sum did not add up and there is an engine with vision. Nothing is
  saved without the person reviewing.

## Limitations

- The per-store layout profiles do not exist yet; the country's generic profile applies.
- HEIC photos are only converted on the Mac.
- PaddleOCR was not measured end to end on a modest machine.
- Even when the sum adds up, the total may be wrong (a non-fiscal receipt with a service charge
  outside the sum, for example): that is why review remains.

## Sources

- NFC-e DANFE and QR Code manual (SEFAZ; copy at
  https://www.sefaz.rr.gov.br/Manual_do_DANFE_NFC-e_QR_Code.pdf; current version on the NF-e
  National Portal, nfe.fazenda.gov.br › Documentos › Manuais).
- Technical Note 2025.001 (NFC-e QR Code version 3 layout):
  https://netcpa.com.br/noticias/nf-e-publicada-nota-tecnica-2025001-divulgando-o-leiaute-do-qr-code-versao-3-da-nfc-e/21325
- NFC-e lookup by QR Code (SEFAZ-ES): https://sefaz.es.gov.br/qr-code
- Wikimedia Commons, Category:Receipts by country:
  https://commons.wikimedia.org/wiki/Category:Receipts_by_country
- Engenere/BrazilFiscalReport (generated NFC-e DANFE, LGPL-3.0):
  https://github.com/Engenere/BrazilFiscalReport

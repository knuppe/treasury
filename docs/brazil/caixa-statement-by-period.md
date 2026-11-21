# Caixa: "Extrato por período" (Internet Banking PDF)

The Caixa checking account statement downloaded from Internet Banking ("Extrato por período", statement by period) is a PDF
with no text layer: each page is an image. Treasury extracts the page images, runs OCR
on the computer and reads the statement with a deterministic reader, checked against the balances, without AI. This
document is the specification of the layout. All examples are invented; no data from a real
file (name, account, CPF/CNPJ, amounts, descriptions) enters the repository.

## The file

- It is the Caixa **checking account statement**. The name comes as
  `Comprovante_AAAA-MM-DD_HHMMSS.pdf`, but it is **not a receipt**: the file name is no use for
  classifying.
- PDF 1.3 (`pdfmake`), A4, **no font and no text layer**: the whole page is **one JPEG
  image** (`/DCTDecode`, RGB, ~1957×3230 px, ~268 ppi). `pdftotext` returns empty.

## Path (no AI)

1. `sniff`: a PDF whose text (normalized) is empty or almost empty → **extract the page images**
   with `lopdf` (already a dependency). For `/DCTDecode` the stream bytes **are** the JPEG (no decoding).
   `/FlateDecode` and others: outside v1 (warning "image-only PDF in a format we do not read yet").
2. Local OCR of the images, in page order: Apple Vision through the app on the Mac (the interface receives the
   JPEGs through a new command, runs Vision and returns the lines, as it already does with photos), the core's PaddleOCR
   in other cases (`crate::ocr::read_text`). Join the lines with `receipts::ocr::lines::row_texts`.
3. The OCR text (or the PDF text, if one day it comes with a text layer) goes through
   `layouts::identify` with a new layout **`bank.caixa.extrato-periodo.v1`**
   (`subject: BankStatement`, `format: Pdf`, issuer "Caixa").
4. `FileKind::Statement`, label "Caixa statement (PDF, read on the computer)" and the normal statement
   import flow (choose account, preview, dedupe, save). `engine.rs` needs to
   route `ImportFormat::Pdf` by the layout subject (card → `card_pdf`, account → new reader).
5. Also classify this way if the user sends the **image** (PNG/JPEG/screenshot) of the same statement:
   `import::image::classify` must recognize the markers below before "receipt"
   (the text contains "PIX", which today would pull it toward receipt).

## Detection markers (folded text: lowercase, no accents)

Require **at least 3**, with the first OR the second mandatory:

| marker | where it appears |
|---|---|
| `extrato por periodo` | title, below the logo |
| `periodo dos lancamentos` | header |
| `saldo anterior` | line before the table |
| table header with `nr. doc` + `historico/complemento` + `favorecido` + `cpf/cnpj` + `valor` + `saldo` (tolerate OCR: `nr doc`, `historico complemento`) | start of the table |
| `alo caixa` or `sac caixa` or `ouvidoria: 0800 725 7474` | footer |
| `saldo dia` (daily balance lines) | body |

The "CAIXA" logo is an image and the OCR may or may not read it; do not depend on it. In the upper right corner there is a
free label (e.g. `#PESSOAL`), a nickname for the account chosen by the customer; ignore it (at most suggest
it as the account name).

## Header (label → value pairs, one per line)

```
Cliente                  FULANA DE TAL
Conta                    0001 / 3701.000012345678-9
Data                     15/03/2025 - 09:30
Mês                      Março
Período dos lançamentos  01/03/2025 até 15/03/2025
                                   SALDO ANTERIOR                     R$ 1.250,00 C
```

- `Conta`: `AAAA / OOOO.NNNNNNNNNNNN-D` (branch / operation.number-digit). Use only to **suggest**
  the Treasury account (compare with the number saved on the account, if any); never write it to the transaction.
- `Data`: statement issue date (`DD/MM/AAAA - HH:MM`). It is not a transaction date.
- `Período dos lançamentos`: `DD/MM/AAAA até DD/MM/AAAA` → import period (for the overlap warning and
  to know the year). All dates in the table must fall within it.
- `SALDO ANTERIOR`: opening balance of the period, `R$ ` + value + `C`/`D`.
- `Cliente` and `Mês`: ignore.

## "Lançamentos" table

Columns, in this order: **Date/time · Nr. Doc · History/Complement · Payee · CPF/CNPJ · Amount · Balance**.

```
Lançamentos            Nr. Doc  Histórico/Complemento  Favorecido          CPF/CNPJ          Valor        Saldo
12/03/2025 - 00:00:00  000000   SALDO DIA                                                    0,00 C       150,40 D
12/03/2025 - 09:00:00  000000   DEBITO PRESTACAO HAB                                         3.200,00 D   150,40 D
07/03/2025 - 00:00:00  000000   SALDO DIA                                                    0,00 C       3.049,60 C
07/03/2025 - 18:40:00  004417   COMPRA ELO              Loja Exemplo Ltda   **12.345/0***    79,90 D      3.049,60 C
07/03/2025 - 10:02:11  120333   PAG BOLETO              Empresa Ficticia S                   120,50 D     3.049,60 C
05/03/2025 - 00:00:00  000000   SALDO DIA                                                    0,00 C       3.250,00 C
05/03/2025 - 08:14:02  011926   CRED PIX CHAVE          Fulano Beltrano Si  ***.123.456***   2.000,00 C   3.250,00 C
```

Observed rules:

- **Order: most recent to oldest.** Within a day, the `SALDO DIA` line comes first,
  followed by the day's transactions (also most recent to oldest).
- **Date**: `DD/MM/AAAA - HH:MM:SS` (4-digit year, always present). Keep the date; the time only
  goes into the external id (order and dedupe).
- **Nr. Doc**: 6 digits, often `000000` (not unique).
- **History**: uppercase, no accents (`CRED PIX CHAVE`, `PREV CAIXA`, `DEBITO PRESTACAO HAB`,
  `PAG BOLETO`, `COMPRA ELO`, `ENVIO PIX`, `TARIFA`...). It becomes the description.
- **Payee**: optional, **cut at ~18 characters** (e.g. "Fulano Beltrano Si"). It becomes `payee`.
- **CPF/CNPJ**: optional, **masked** (`***.123.456***` CPF; `**12.345/0***` CNPJ). **Discard**:
  do not save it or send it to the agent.
- **Amount**: `1.234,56` + space + `C` (credit, positive) or `D` (debit, negative). No `R$`, no sign.
  In whole cents.
- **Balance**: same format; `D` = negative balance. **In the real example the balance on the transaction line is the
  day's closing balance** (equal to the `SALDO DIA` of that date), not the running balance line by line.
- **`SALDO DIA`** (time `00:00:00`, doc `000000`, amount `0,00 C`): **it is not a transaction**. It is the
  day's closing balance → check point.
- There is no separate final balance line: the final balance of the period is the most recent `SALDO DIA`.

## Check (the statement's "3-band check", no AI)

With the transactions in chronological order (reversed from the printed one):

1. Per day: `closing(previous day)` (or `SALDO ANTERIOR` on the first day) + sum of the day's amounts
   = the day's `SALDO DIA`. Invented example: 1.250,00 + 2.000,00 = 3.250,00; 3.250,00 − 120,50 − 79,90 =
   3.049,60; 3.049,60 − 3.200,00 = −150,40 ("150,40 D").
2. Total: `SALDO ANTERIOR` + sum of everything = most recent `SALDO DIA`.
3. If the balances of the lines of the same day differ from each other, treat it as a running balance and
   check line by line (`previous line balance + amount = line balance`). The reader must accept both
   variants (only one line per day was seen in the real example).

If the check fails: do not discard; mark the line/day with a warning ("the balance does not add up, check the
value read") in the preview so the user can correct it. An OCR error in one digit shows up here.

## OCR tolerance (the image is clean, generated, 268 ppi — the OCR should do well)

- A table line starts with `\d{2}/\d{2}/\d{4}\s*-\s*\d{2}:\d{2}:\d{2}`. A line without a date at the start
  = continuation of the previous one (broken Payee) or garbage.
- Take **the last two monetary values** on the line (`[\d.]+,\d{2}\s*[CD]`) as Amount and Balance;
  what is between doc and amount is history + payee + CPF/CNPJ. Separate the history from the payee
  by the large gap (`row_texts` separates columns with two spaces); if that fails, history = everything
  up to the first mixed-case run.
- The masked CPF/CNPJ (`*` and digits) is recognizable by the asterisks: remove it before building the
  description.
- Accept `C`/`D` stuck to the value (`389,65D`), `O` in place of `0` in Nr. Doc, a missing `.` in the thousands.
- Ignore the footer (`Alô CAIXA`, `Pessoas com deficiência`, `SAC CAIXA`, `Ouvidoria`) and the header.

## Page breaks

The real example has 1 page. For long periods several pages are expected, probably one image
per page (hypothesis, **confirm with a statement of 2+ pages**). The reader must: OCR each image
in order; ignore the repeated customer header, title and table header; ignore the footer; and
join the lines. If a page has more than one image (strips), sort by the `cm` position in the
page content (top to bottom).

## Dedupe and external id

`external_id = sha256("caixa-periodo|" + data + hora + nr_doc + valor_centavos + historico)`.
Nr. Doc alone does not work (`000000` repeats). Reimporting the same period or overlapping periods does not
duplicate. The existing content-based dedupe still applies to statements coming from OFX of the same account.

## Synthetic fixture (all invented)

Two pieces, to test the reader without depending on the OCR engine:

1. **`crates/treasury-core/src/import/fixtures/caixa-extrato-periodo.ocr.txt`** — the lines as
   `row_texts` returns them (runs separated by two spaces), with the header from section 4, the table
   from section 5 and the footer. Variants in the test (in code, from the same text):
   - `C`/`D` stuck to the value and `O` in Nr. Doc (dirty OCR) → same result;
   - one digit swapped in an amount → the check flags the day;
   - two "pages" (header and table repeated in the middle) → same result, without duplicating;
   - running balance line by line (variant of §6.3) → accepted;
   - negative final balance (`150,40 D`).
   Expected: 4 transactions (2.000,00 C; 120,50 D; 79,90 D; 3.200,00 D), dates 05/03, 07/03, 07/03,
   12/03/2025, opening balance 1.250,00, closing −150,40, check ok, CPF/CNPJ absent from
   every field.
2. **Image-only PDF**, generated in the test itself with `lopdf` (as the note tests already do):
   `Creator (pdfmake)`, one A4 page, a small JPEG XObject (any valid JPEG generated in
   memory by the `image` crate, or minimal JPEG bytes embedded in the test). Tests: `sniff` detects "PDF
   with no text with image", the extraction returns exactly the JPEG bytes, and without OCR available the
   result is a clear warning (not a generic error nor "brokerage note").
3. Optional (manual, outside CI and the repository): a generator that draws the "Lançamentos" table fixture
   on a 1957×3230 image (Verdana/DejaVu font, white background) and wraps it in a PDF like the real one, to
   measure Vision and PaddleOCR end to end.

## Out of scope

Savings account statement, Caixa card statement, statement with a text layer from other channels (the text reader
works the same if one shows up), and automatic categorization by history (stays with the rules that
already exist).

## In the code

- `pdf::page_jpegs` extracts the JPEGs (`/DCTDecode`) from the pages with `lopdf` (which already came with
  `pdf-extract`; it went from a test dependency to a normal dependency, with no new crate).
- `ocr::read_scanned`: OCR of the pages in order (the core's PaddleOCR), cached by sha256 so
  sniff, detection, preview and save do not read again. `import.sniff` and `import.detect`
  run outside the database lock. The Mac's Vision does not enter here yet (PaddleOCR read the
  synthetic image completely right, in 1.7 s).
- `import::caixa`: markers, header, table, `SALDO DIA`, the two balance variants and the
  check; CPF/CNPJ discarded; `external_id` as in "Dedupe and external id" (prefix `caixa:`).
- Layout `bank.caixa.extrato-periodo.v1` in `layouts.rs`; `import.preview` routes the PDF by account
  (checking account → statement, card → card statement); the photo or screenshot enters by the same path
  (`ImageKind::Statement`, before "receipt").
- Tests: `.ocr.txt` fixture with the variants of "Synthetic fixture", PDF with text and image-only PDF generated
  with `lopdf`, and an ignored test with a synthetic image rendered outside the repository
  (`TREASURY_CAIXA_SAMPLE`). Multi-page is still a hypothesis (tested only on text).

## Limitations

- The Mac's Vision does not enter this reading; only the core's PaddleOCR.
- A statement with several pages was tested only on text, not with real images.
- Page images in a format other than JPEG (`/FlateDecode` and others) are not read.

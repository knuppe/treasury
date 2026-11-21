# PDF to Markdown for the agents

The agents read PDFs (attachments, inbox files, CVM documents) as Markdown, with
tables, headings and paragraphs, instead of running text. The converter belongs to the app itself, on top of the
PDF reading it already uses.

## Why a custom converter

Two libraries were evaluated first. `pdf_oxide` (MIT/Apache), in the test with four invented PDFs (statement,
brokerage note, material fact, a 120-line pension statement), cut the text
of cells where the font has spacing adjustment ("PADARIA BOM PAO" became "P", "Valor (R$)"
became "V", "MENSAL 110" became "MENSAL 1") and sent the rest to the end of the page. EdgeParse
(Apache) got the statement tables right, but erased "CONTRIBUICAO MENSAL" in 100 of the 120 lines
(repeated text treated as a header) and the title of the material fact. For financial values,
losing or moving a character is worse than having no table. Hence: a custom converter
on top of the reading the app already uses (`pdf-extract`), with no new dependency and no larger app.
The test used four invented PDFs; no real document entered the repository.

## How it works (`pdf_markdown.rs`)

- **Characters with position**: a `pdf-extract` `OutputDev` keeps each character with x, y,
  width and size. Font without widths (some browser PDFs): the average width of the page.
- **Lines and runs**: characters at the same height become a line; within it, a wide space
  (1 to 1.2 times the font size) starts another run (cell). Space between words: the
  spaces the PDF writes; without them, an offset above 0.2. Letter adjustment (kerning, up to
  0.8 when the PDF writes the spaces) does not separate words; numbers never stick together (0.4).
- **Tables**: 3 or more consecutive lines with 2 or more runs, most with the same number of
  runs; the columns come from the run edges (median) and each run goes to the column it
  overlaps most. Output in GFM, with `|` escaped.
- **Headings** by a font larger than the body (1.25×: `##`; 1.7×: `#`); **paragraphs** join
  nearby lines of the same size.
- **Header and footer**: only in the margins (top and bottom 10%), text that repeats (with the
  numbers swapped) on 2/3 of the pages, from 3 pages on, is removed from the second page onward;
  page numbers are removed. Repeated content in the middle of the page stays.
- **Check**: the letters and numbers of each page's Markdown are the same as in the lines
  kept; if not, the page comes out as running text (`plain_pages`).
- **Image-only PDF**: the local OCR that already exists (Vision or PaddleOCR), `engine: ocr`.
- Each page starts with `<!-- página N -->`.

Measured on the four invented PDFs: everything in under 20 ms; the Markdown is 1–15% larger than the
running text (the table bars), but the agent does not need to rebuild the columns.

## Where it is used

- MCP `read_document` (read, `import.read_document`): `inbox_id`, `path` (Downloads and the inbox
  folder), `attachments` (in-app agent) or `files`; in chunks of up to 40 thousand characters.
- `read_attachment` (in-app agent attachments) and `read_corporate_document` (CVM) return Markdown.
- The readers with rules (statement, note, payslip, pension, FGTS, receipt) did not change.
- Inbox: "View as text" on PDFs shows the Markdown the agents read (to check and
  copy).
- Manual check with other PDFs: `PDF_MD_DIR=pasta cargo test -p treasury-core --lib --
  --ignored pdf_markdown::manual` writes `<arquivo>.treasury.md` next to each PDF.

## Limitations

- A table header with very close cells can merge two columns ("Quantidade Preço").
- A cell with text on two lines becomes two table rows.
- A table in a scanned PDF comes out as text; the columns still do not come from the position of the OCR boxes.

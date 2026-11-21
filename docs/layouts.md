# Imported file layouts

Banks, brokers and funds change file formats without notice. To avoid importing wrongly,
every import starts by identifying the file (`crates/treasury-core/src/layouts.rs`):

- **Subject**: account statement, card statement, meal and food voucher statement, brokerage
  note, pension statement.
- **Layout**: the issuer and version that the app knows how to read (`pension.quanta.v1-colunas`,
  `pension.quanta.v1-colado`, `notes.sinacor`, `ofx`, `csv.nubank_conta`, `csv.flash`…). The
  list comes from `import.layouts`.

## A file that does not match

If the file has the markers of a subject (for example, "Extrato Financeiro do
Participante") but matches no known layout, the import is refused with the reason:
"looks like a pension statement, but matches no known layout (…). It may be a new
version: nothing was imported". A file of another subject is also refused with the reason.
Once recognized, each reader still checks the result (pension compares units and
balance with the footer and warns in the preview if they do not match).

## Where it shows up

- Universal import (`import.sniff`): the type and the recognized `layout` of each file.
- Agent: the text of the attached PDF starts with the recognized layout and the tools that read it (or
  the unknown-layout warning).
- Previews: the pension statement shows the layout; brokerage notes say why a
  PDF was not read.

## Adding a new version

1. Keep a sample file (without personal data) in the importer's `fixtures/`.
2. Write the reader (or adapt the current one) and a `matches_*` function that recognizes the new
   version and not the others.
3. Add a `Def` in `layouts.rs` (id, subject, format, issuer, version, description).
4. Test: the new file is recognized by the new layout, the old ones stay with theirs and the
   reader checks the totals.

Old versions remain valid for old files. CSV presets become layouts on their own
(`csv.<preset>`), by the header.

## PDF card statement

`card.generico.v1` reads any card statement with one line per transaction (date, description and amount) and recognizes the bank by name. See [card-statement-pdf.md](card-statement-pdf.md). A bank-specific layout (`card.<bank>.vN`) goes in ahead of the generic one when the generic one gets that statement wrong.

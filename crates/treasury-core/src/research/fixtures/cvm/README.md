# CVM fixtures (real data)

Rows cut from the CVM open data files (dados.cvm.gov.br, ITR of 2026-09-30 and DFP of 2025,
ODbL), only for the five companies the tests use (Klabin, Taesa, CVC, Itaú Unibanco and WEG) and
only the accounts the reader uses. Columns the reader ignores were dropped and values keep the
original scale (`ESCALA_MOEDA`).

They pin down real cases found in the indicator study of November 2026: net income attributable
to owners left at zero in the consolidated statements (Klabin, Taesa, CVC), a unit with ticker
digits in its composition (Klabin), shares reported in thousands (Taesa, Itaú), a bank whose
figures follow the Central Bank standard in the parent-only statements (Itaú) and a clean case
(WEG). The expected values were recomputed independently from the full files
(`scripts/indicators-audit.py`) and match the public figures where those exist.

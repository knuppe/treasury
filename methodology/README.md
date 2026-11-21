# Methodology of the per-asset health score

Each version (`v<MAJOR.MINOR.PATCH>/`) has `weights.json` (profiles, dimensions, weights and critical metrics), `thresholds.json` (metric catalog, anchors, normalization, missing data, confidence, bands, red flags) and `segments.json` (classes, segments, sector map). The core embeds the version in use (`score::config::METHODOLOGY_VERSION`); each snapshot stores the version. Explanation in [docs/asset-score.md](../docs/asset-score.md).

SemVer: MAJOR changes dimensions or scale; MINOR adds indicators/segments or recalibrates weights; PATCH fixes a bug or data.

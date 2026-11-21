# Third-party crates with a local fix

Copies of third-party crates that carry a fix Treasury needs before the crate that pulls them in
can move to a fixed version. Each one is the published crate (source and license only, without its
tests and benchmarks), with the fix marked by a `Treasury:` comment, and is wired in through
`[patch.crates-io]` in the root `Cargo.toml`. Remove the copy as soon as the dependency tree reaches
a fixed version.

## glib 0.18.5

- **Why:** GHSA-wrw7-89jp-8q8g. `glib::VariantStrIter::impl_get` passed an out-argument to
  `g_variant_get_child` as `&p` instead of `&mut p`, which is undefined behaviour; the fix is in
  glib 0.20.0.
- **Where it comes from:** only the Linux build, through the GTK 3 stack of Tauri 2 (gtk, wry, tao,
  webkit2gtk), which is pinned to gtk-rs 0.18. Treasury does not use glib directly.
- **The change:** `src/variant_iter.rs`, the same one-line fix as glib 0.20 (`let mut p` and
  `&mut p`).
- **Remove when:** Tauri moves to a glib at or above 0.20.
- **License:** MIT (`LICENSE`, `COPYRIGHT`), as published.

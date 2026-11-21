# Bundled PaddleOCR models

Used by `src/receipts/ocr/paddle.rs` (feature `ocr-paddle`, on by default) to read a receipt from a
photo without a QR code, on the computer itself (docs/ocr-receipts.md §12.2). They go inside the
binary with `include_bytes!` and run on `tract` (pure Rust, no ONNX Runtime).

- ``det.onnx``
  - **What it is:** PP-OCRv3 text detection (DBNet, `ch_PP-OCRv3_det_infer`), 2.4 MB
  - **Origin:** PaddleOCR (PaddlePaddle), converted to ONNX by RapidOCR: file `models/ch_PP-OCRv3_det_infer.onnx` of the `rapidocr_onnxruntime` 1.2.3 package (https://github.com/RapidAI/RapidOCR)
  - **License:** Apache-2.0
- ``rec_latin.onnx``
  - **What it is:** PP-OCRv5 *mobile* Latin recognition (`latin_PP-OCRv5_mobile_rec`), 8 MB
  - **Origin:** Hugging Face, `PaddlePaddle/latin_PP-OCRv5_mobile_rec_onnx` (https://huggingface.co/PaddlePaddle/latin_PP-OCRv5_mobile_rec_onnx)
  - **License:** Apache-2.0
- ``rec_latin_dict.txt``
  - **What it is:** Dictionary of the Latin recognition (836 characters, one per line)
  - **Origin:** Same repository (the `character_dict` list from the model's `.yml` configuration file)
  - **License:** Apache-2.0

## Modification

`rec_latin.onnx` differs from the original only in the names of the dynamic dimensions
(`dim_param`): `DynamicDimension.0`, `.1` and `.2` became `DynamicDimension_0`, `_1` and `_2`,
because `tract` does not accept a dot in the symbol name. The weights and the graph are the same
(4 bytes changed).

SHA-256:

```
3439588c030faea393a54515f51e983d8e155b19a2e8aba7891934c1cf0de526  det.onnx
e687a0e3104ea72019731c7ed83a362d2d4f70981ebf4ffe6b6904d3f4b53568  rec_latin.onnx
ccbcc45730b3fbbd9050c5bc74db6a99067141ef1035e3d14889a84a6b9b1aff  rec_latin_dict.txt
```

## CTC output

The recognition returns 838 classes per step: 0 is the CTC blank, 1 to 836 are the dictionary
lines, in order, and 837 is the space (like RapidOCR's `CTCLabelDecode`). Changing the model
requires its dictionary and an updated `dicionario_bate_com_o_modelo` test.

## License

Apache License 2.0 (https://www.apache.org/licenses/LICENSE-2.0). The models keep their own license
inside Treasury (AGPL-3.0-only); the attribution is also in `DATA_LICENSES.md`.

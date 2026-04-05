# tlsr82xx

Monorepo for Telink TLSR82xx Rust support.

## Layout

- `tlsr82xx-pac`: peripheral access crate, SVD files, and generation scripts.
- `tlsr82xx-hal`: hardware abstraction layer built on top of the PAC.

## SVD Generation

```bash
python3 tlsr82xx-pac/scripts/gen_svd.py \
  --sdk ../tl_zigbee_sdk \
  --chip 8258 \
  --output tlsr82xx-pac/svd/tlsr8258.svd
```

## PAC Generation

The repository also contains a PAC generation wrapper for `svd2rust`.

```bash
python3 tlsr82xx-pac/scripts/gen_pac.py \
  --chip 8258 \
  --svd tlsr82xx-pac/svd/tlsr8258.svd \
  --out-dir tlsr82xx-pac/generated
```

This command expects `svd2rust` and `form` to be installed locally.

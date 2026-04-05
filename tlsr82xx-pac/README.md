# tlsr82xx-pac

Peripheral access crate workspace member for Telink TLSR82xx chips.

This crate currently hosts:

- `scripts/gen_svd.py`: generator for `CMSIS-SVD` files from the Telink SDK.
- `scripts/gen_pac.py`: wrapper around `svd2rust` for generating PAC sources.
- `svd/`: generated SVD inputs for future PAC generation.
- `generated/`: output tree for per-chip generated PAC sources.

`gen_pac.py` defaults to the local binary `../svd2rust-aarch64-apple-darwin`
from the workspace root.

The top-level `tlsr82xx-pac` crate is a facade that reexports one generated
per-chip PAC selected by feature.

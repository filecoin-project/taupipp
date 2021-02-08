# Taupipp: Powers of Tau aggregator for Inner Pairing Product

Taupipp is a tool that allows to aggregate two powers of tau into a combined one
to be used as a CRS for Groth16 aggregation protocol (see [here](https://github.com/filecoin-project/bellperson/tree/feat-ipp2/src/groth16/aggregate) for more info). Specifically, it takes as
input:
```
g,g^alpha, g^(alpha^2), ... in G1
h,h^alpha, h^(alpha^2), ... in G2
```
and
```
g,g^beta, g^(beta^2), ... in G1
h,h^beta, h^(beta^2), ... in G2
```
and then outputs
```
g,g^alpha, g^(alpha^2), ... in G1
h,h^alpha, h^(alpha^2), ... in G2
g,g^beta, g^(beta^2), ... in G1
h,h^beta, h^(beta^2), ... in G2
```

By default, this tool will download (or read on file) Filecoin and Zcash's last
power of tau partcipation and returns the combination of the two.

## Aggregate

Simply run 
```
cargo run --release --bin assemble
```
By default it tries to look for the files `zcash_powers` and `filecoin_powers`
that corresponds to the releveant files of the powers of tau. 
It outputs the combined power of tau to `ipp_srs` as well as **the hash of both
inputs and output**.

## Verify

If you rebuild the aggregated CRS yourself and want to check consistency with
one already published, you can run
```
cargo run --release --bin verify
```
It looks for the file `ipp_srs` or downloads the default filecoin-zcash one and
outputs the hash of it.

# musl_cdylib_demo

A Rust `cdylib` with no libc of its own. A musl-linked C host `dlopen()`s it, and the library's
`open`/`read`/`close`/`write` bind straight into the musl already mapped in the host. One libc in
the process.

Regular `std` code, `println!`, `String`, `CString` all work.

## Build

Needs `musl-gcc`. `rust-toolchain.toml` pins nightly, so rustup fetches the rest.

```sh
make run        # build + run
make release
make clean
```

```
cdylib loaded, calling into musl...
read from /etc/hostname: xps
```

## Check it

```sh
readelf -d target/x86_64-unknown-linux-musl/debug/libmusl_cdylib_demo.so | grep NEEDED
```

One line, `ld-musl-x86_64.so.1`. The libc functions stay `UND` and resolve at `dlopen()`. The link
uses `-z defs`, so a symbol musl doesn't define fails the build instead of the load.

## Why nightly

musl has no unwinder. Every `_Unwind_*` reference `std` leaves behind has to be satisfied by
`libgcc_s.so.1`, which is linked against glibc, so the only way to keep the loader happy is to emit
none of them.

Default `panic=unwind` is the worst case. `std` references the whole unwinder, those symbols bind
against the real `libgcc_s.so.1`, and the `.so` comes out needing it by name plus versions
`GCC_3.0`/`GCC_3.3`/`GCC_4.2.0`. The musl loader can't load it and `dlopen()` fails.

`panic=abort` gets close but not all the way. The unwinder proper is gone, yet `std`'s backtrace
support still calls `_Unwind_Backtrace` and `_Unwind_GetIP`. They stay `UND`, nothing in the process
defines them, and `RTLD_NOW` fails:

```
Error relocating ./libmusl_cdylib_demo.so: _Unwind_Backtrace: symbol not found
```

So remove the panic machinery entirely: `build-std` plus `-Cpanic=immediate-abort` rebuilds `std`
with panics as a trap. No `_Unwind_*` references survive. Both need nightly.

Cost: a panic is now `SIGILL` with no message. `.unwrap()` failures crash silently. Keeping the
message costs about 70 KB, `panic=abort`, `libgcc_eh.a` linked statically to satisfy the two
backtrace symbols, a stub `_dl_find_object` returning `-1` because that archive
member wants a glibc 2.35 symbol, and `-Zbuild-std-features=` to drop the `gimli`/`addr2line`
symbolization that would otherwise add 200 KB more to produce an always-empty backtrace. Three extra
moving parts for a code path that shouldn't run.

## Dropping libgcc_s

With the unwinder gone, `libgcc_s.so.1` *still* showed up in `NEEDED`.

std's `unwind` crate carries `#[link(name = "gcc_s")]` for musl whenever `crt-static` is off, so
rustc passes `-lgcc_s` regardless of whether anything uses it. On Fedora that resolves to an ld
script:

```
GROUP ( /lib64/libgcc_s.so.1 libgcc.a )
```

and ld records the dependency even though nothing references a symbol from it.

`-Z link-native-libraries=no` makes rustc ignore every crate's `#[link]` and `rustc-link-lib`
dynamic libraries, which removes `-lgcc_s` along with the `libc` crate's `-lc`. `-C link-arg=-lc`
puts musl back, so it's the only library on the link line.

## Dependencies

The same flags keep any crate you add musl-only:

- Pure Rust crates go through `std`/`libc` and resolve to musl.
- Crates that build and bundle their own C as a static archive (`rustc-link-lib=static=...`, most
  `-sys` crates' `vendored`/`bundled`/`static` features) still link in. Their code lands inside the
  `.so`. Compile that C with `musl-gcc`, not glibc's `gcc`.
- Crates that want a system shared library (`#[link(name = "z")]`) have that library dropped, and
  `-z defs` turns the leftover undefined symbols into a link error:

```
undefined reference to `zlibVersion'
```

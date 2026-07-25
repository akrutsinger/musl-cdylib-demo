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

Build through `make`; one linker flag uses a path relative to the project root.

## Check it

```sh
readelf -d target/x86_64-unknown-linux-musl/debug/libmusl_cdylib_demo.so | grep NEEDED
```

One line, `ld-musl-x86_64.so.1`. The libc functions stay `UND` and resolve at `dlopen()`. The host
passes `RTLD_NOW`, so a missing symbol fails there instead of crashing later.

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
message costs about 70 KB, `panic=abort`, `libgcc_eh.a` added to the shim's `GROUP` to satisfy the
two backtrace symbols statically, a stub `_dl_find_object` returning `-1` because that archive
member wants a glibc 2.35 symbol, and `-Zbuild-std-features=` to drop the `gimli`/`addr2line`
symbolization that would otherwise add 200 KB more to produce an always-empty backtrace. Three extra
moving parts for a code path that shouldn't run.

## The Fedora shim

With the unwinder gone, `libgcc_s.so.1` *still* showed up in `NEEDED`.

`rustc` always passes `-lgcc_s` for a non-static `cdylib`. On Fedora that resolves to an ld script:

```
GROUP ( /lib64/libgcc_s.so.1 libgcc.a )
```

`GROUP` members ignore `--as-needed` or other link time optimization, so ld records the dependency
even though nothing references a symbol from it. `--gc-sections` doesn't help either.

`linker-shim/libgcc_s.so` is an ld script that pulls in only `libgcc.a`, and `-L native=linker-shim`
in `.cargo/config.toml` makes `-lgcc_s` find it first. Archive members load only when referenced, so
it contributes nothing.

## Other distros

Fedora packaging quirk. Where `libgcc_s.so` is a plain symlink, the `--as-needed` rustc already
passes should drop it and the shim is dead weight.

Check `NEEDED` first. If musl is alone, delete the `-L native=linker-shim` line. If not, `file`
whatever `-lgcc_s` resolves to; it may be another ld script.

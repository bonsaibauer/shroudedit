# Validated build baseline

The in-game test on 2026-09-18 successfully copied and placed a mixed selection containing voxel data and five props (window, furniture and small decorations).

The tested DLL was compared byte-for-byte with the local release output before this repository was prepared:

```text
SHA-256  A7200653C13D8665797CDE690B1ACFD43C0E06E6DB87B024F840E2FA691197B6
File     mod.shroudedit.dll
Size     216064 bytes
```

This hash identifies the validated baseline only. A different compiler, source revision or build metadata can legitimately produce a different DLL. Every published archive receives its own `.sha256` file.

The first clean repository rebuild had the same size and differed from the validated DLL in only four bytes: the PE timestamp and its duplicate in the debug directory. The `.text`, `.data`, `.pdata`, `.rsrc` and `.reloc` sections were byte-identical. `/Brepro` is enabled for subsequent release builds so that this timestamp is deterministic.

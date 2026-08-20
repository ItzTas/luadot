# Benchmarks

`cargo bench` measures what every command pays for, over repositories of 16,
128 and 1024 files, so a result reads as a cost per file instead of a single
number.

| Bench | Measures |
| --- | --- |
| `files` | Walking the repository, the status of each file against the system copy, and placing every file as a hard link, as a symlink, or over one that differs. |
| `config` | `link_mode`, `conflict_policy` and `is_ignored` against 4, 32 and 256 rules, plus the path math each managed file goes through. |
| `lua` | Resolving a template, which pays for the Lua runtime, the `ld` interface and the script's own work. |

Comparing two runs is what the numbers are for:

```
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

The second run reports the change against the saved one. A single area is
reached by target, and a single measurement by name:

```
cargo bench --bench files
cargo bench --bench files -- walk/collect_entries/1024
```

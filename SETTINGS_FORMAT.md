# settings.bin format

Location: `%LOCALAPPDATA%\TreeAutoClicker\settings.bin` (or `settings.bin` in the working directory if `LOCALAPPDATA` is unset).

All multi-byte integers are little-endian. The first byte is always the format version.

Current writers emit **v3**. Loaders accept **v1**, **v2**, and **v3** (older files are rewritten as v3 on load).

---

## Version 1

Header length: **18** bytes, then optional process list.

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 1 | `u8` | Version (`1`) |
| 1 | 1 | `u8` | Flags |
| 2 | 4 | `i32` | `current_hotkey` |
| 6 | 4 | `i32` | `interval_ms` |
| 10 | 4 | `i32` | `click_type_index` |
| 14 | 4 | `i32` | `filter_mode` |
| 18… | — | UTF-8 | Processes (optional) |

### Flags (byte 1)

| Bit | Field |
|-----|-------|
| 0 | `always_on_top` |
| 1 | `minimize_to_tray` |
| 2 | `pause_on_window_change` |

### Processes

Null-separated (`\0`) process **names** only. No action field.

```
name1\0name2\0name3
```

---

## Version 2

Same header as v1, plus process entries that include an action.

Header length: **18** bytes, then optional process list.

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 1 | `u8` | Version (`2`) |
| 1 | 1 | `u8` | Flags (same as v1) |
| 2 | 4 | `i32` | `current_hotkey` |
| 6 | 4 | `i32` | `interval_ms` |
| 10 | 4 | `i32` | `click_type_index` |
| 14 | 4 | `i32` | `filter_mode` |
| 18… | — | UTF-8 | Processes (optional) |

### Processes

Null-separated entries. Each entry is `name` + `\x01` + decimal `action`.

```
name1\x01action1\0name2\x01action2
```

Legacy name-only entries (no `\x01`) are treated as action `1`.

Only enabled processes are written; `enabled` is not stored on disk.

---

## Version 3 (current)

`click_type_index` and `filter_mode` move into the flags byte. Eight `u16` corner/edge stop values sit between the core settings and the process list.

Header length: **26** bytes, then optional process list.

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 1 | `u8` | Version (`3`) |
| 1 | 1 | `u8` | Flags |
| 2 | 4 | `i32` | `current_hotkey` |
| 6 | 4 | `i32` | `interval_ms` |
| 10 | 2 | `u16` | `corner_stop_tl` |
| 12 | 2 | `u16` | `corner_stop_tr` |
| 14 | 2 | `u16` | `corner_stop_bl` |
| 16 | 2 | `u16` | `corner_stop_br` |
| 18 | 2 | `u16` | `edge_stop_top` |
| 20 | 2 | `u16` | `edge_stop_right` |
| 22 | 2 | `u16` | `edge_stop_bottom` |
| 24 | 2 | `u16` | `edge_stop_left` |
| 26… | — | UTF-8 | Processes (optional; same as v2) |

### Flags (byte 1)

| Bits | Field |
|------|-------|
| 0 | `always_on_top` |
| 1 | `minimize_to_tray` |
| 2 | `pause_on_window_change` |
| 3–4 | `click_type_index` (`& 0b11`) |
| 5 | `filter_mode` (`0` or `1`) |

---

## Field evolution

| Field | v1 | v2 | v3 |
|-------|----|----|----|
| `always_on_top` / `minimize_to_tray` / `pause_on_window_change` | flags | flags | flags |
| `current_hotkey` / `interval_ms` | `i32` | `i32` | `i32` |
| `click_type_index` / `filter_mode` | `i32` each | `i32` each | packed in flags |
| Corner / edge stops | — | — | 8 × `u16` |
| Processes | names only | `name\x01action` | `name\x01action` |

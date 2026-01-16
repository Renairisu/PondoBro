# PondoBro Rust Frontend (Design Port)

This frontend mirrors the visual layout from the legacy UI using Yew + Tailwind CDN.

## Quick start (Trunk)

If you use Trunk for Yew apps:

```powershell
trunk serve
```

## Build check

```powershell
cargo build
```

## Notes

- Tailwind is loaded via CDN in [index.html](index.html).
- Design tokens are defined in [styles.css](styles.css).
- The sidebar and header match the old layout, with a dashboard sample and placeholder pages.

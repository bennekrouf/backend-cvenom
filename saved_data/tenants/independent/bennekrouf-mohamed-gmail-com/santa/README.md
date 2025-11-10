# CV Data for santa

This directory contains CV data for santa.

## Files

- `cv_params.toml` - Personal information and configuration
- `experiences_en.typ` - Work experience and content in English
- `experiences_fr.typ` - Work experience and content in French
- `profile.png` - Profile image (add your own)

## Usage

Generate CV in English:
```bash
cargo run -- generate santa en
```

Generate CV in French:
```bash
cargo run -- generate santa fr
```

## Customization

1. Edit `cv_params.toml` to update personal information
2. Modify `experiences_*.typ` files to add your experience
3. Replace `profile.png` with your profile image

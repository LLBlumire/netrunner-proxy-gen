# Netrunner Proxy Gen (NSG Only)

## Dependencies

- poppler 
- imagemagick

You need to have both of these installed and in your path, such that `pdfimages` and `imagemagick convert` work.

This is possible in all major operating systems.

## Usage

```
cargo run -- dl -d <deckid>
```

You can also pass `--include-basic-actions` if you want to add in the NSG basic action cards.

You can also pass `--include-marks` if you want to add in the mark cards.
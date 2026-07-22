#!/usr/bin/env bash
# Generate monogram project assets under assets/ (next to this script, or --out DIR).
#
# Pipeline:
#   1. Write logo.svg  -- rounded square + letter (pure SVG text)
#   2. magick: SVG -> logo-1024.png + icon-{16,32,48,64,128,256,512}.png + icon.png
#   3. magick: multi-size icon.ico (16/32/48/256)
#   4. magick: social-banner.png (1280x640) + README-header.png (1280x320)
#      solid BG, left accent bar, monogram mark, name + tagline lettering
#
# Deps: ImageMagick 7 (`magick`). Font: Noto Sans Bold (or override --font).
#
# Usage:
#   ./generate-assets.sh --letter R --name rust_template --tagline "Rust project template"
#   ./generate-assets.sh -l C -n chapterize -t "YouTube-style chapters via ffmpeg" --accent "#EF4444"
#   ./generate-assets.sh --out /path/to/other/assets ...
#
# Defaults match rust_template current branding.
set -euo pipefail

BG="#2D2D2D"
ACCENT="#00D9A5"
LETTER="R"
NAME="rust_template"
TAGLINE="Rust project template"
OUT=""
FONT=""
SOURCE_PNG=""

usage() {
  cat <<'USAGE'
generate-assets.sh - monogram SVG/PNG/ICO + social banners via ImageMagick

Options:
  -l, --letter CHAR       Monogram letter (default: R)
  -n, --name NAME         Project name for banners (default: rust_template)
  -t, --tagline TEXT      Banner tagline (default: Rust project template)
  -a, --accent HEX        Accent color (default: #00D9A5)
  -b, --bg HEX            Background color (default: #2D2D2D)
  -o, --out DIR           Output directory (default: dir of this script)
  -f, --font PATH         Bold TTF/OTF for raster lettering + banners
      --source-png PATH   Use existing PNG as master art instead of monogram
  -h, --help              Show this help
USAGE
}

while [ $# -gt 0 ]; do
  case "$1" in
    -l|--letter) LETTER="${2:?}"; shift 2 ;;
    -n|--name) NAME="${2:?}"; shift 2 ;;
    -t|--tagline) TAGLINE="${2:?}"; shift 2 ;;
    -a|--accent) ACCENT="${2:?}"; shift 2 ;;
    -b|--bg) BG="${2:?}"; shift 2 ;;
    -o|--out) OUT="${2:?}"; shift 2 ;;
    -f|--font) FONT="${2:?}"; shift 2 ;;
    --source-png) SOURCE_PNG="${2:?}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -z "${OUT}" ]; then
  OUT="${SCRIPT_DIR}"
fi
mkdir -p "${OUT}"
OUT="$(cd "${OUT}" && pwd)"

if ! command -v magick >/dev/null 2>&1; then
  echo "error: ImageMagick 7 'magick' not found in PATH" >&2
  exit 1
fi

# Prefer a bold sans if --font not set
if [ -z "${FONT}" ]; then
  for candidate in \
    /usr/share/fonts/noto/NotoSans-Bold.ttf \
    /usr/share/fonts/TTF/DejaVuSans-Bold.ttf \
    /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf \
    /usr/share/fonts/liberation/LiberationSans-Bold.ttf \
    /usr/share/fonts/noto/NotoSans-Regular.ttf
  do
    if [ -f "${candidate}" ]; then
      FONT="${candidate}"
      break
    fi
  done
fi
if [ -z "${FONT}" ] || [ ! -f "${FONT}" ]; then
  echo "error: no usable bold font found; pass --font /path/to/Bold.ttf" >&2
  exit 1
fi

# Escape XML special chars in letter (keep it to one grapheme ideally)
xml_escape() {
  printf '%s' "$1" | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g'
}
LETTER_XML="$(xml_escape "${LETTER}")"

write_svg() {
  local path="$1"
  # Pure ASCII SVG monogram -- rounded square + centered letter
  cat > "${path}" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <rect width="512" height="512" rx="96" fill="${BG}"/>
  <text x="256" y="340" font-family="DejaVu Sans, Liberation Sans, Noto Sans, Arial, sans-serif"
        font-size="280" font-weight="700" fill="${ACCENT}" text-anchor="middle">${LETTER_XML}</text>
</svg>
SVG
}

echo "==> out: ${OUT}"
echo "    letter=${LETTER} accent=${ACCENT} bg=${BG}"
echo "    name=${NAME}"
echo "    tagline=${TAGLINE}"
echo "    font=${FONT}"

SVG_PATH="${OUT}/logo.svg"
if [ -n "${SOURCE_PNG}" ]; then
  if [ ! -f "${SOURCE_PNG}" ]; then
    echo "error: --source-png not found: ${SOURCE_PNG}" >&2
    exit 1
  fi
  echo "==> master from source png: ${SOURCE_PNG}"
  # Fit into square on BG, then 1024
  magick "${SOURCE_PNG}" -background "${BG}" -alpha remove -alpha off \
    -gravity center -extent "%[fx:max(w,h)]x%[fx:max(w,h)]" \
    -resize 1024x1024 \
    "${OUT}/logo-1024.png"
  # Still write monogram SVG as the vector brand mark for README
  write_svg "${SVG_PATH}"
else
  echo "==> write logo.svg"
  write_svg "${SVG_PATH}"
  echo "==> rasterize logo-1024.png from SVG"
  # denser raster for clean letterforms
  magick -background none -density 512 "${SVG_PATH}" -resize 1024x1024 "${OUT}/logo-1024.png"
fi

MASTER="${OUT}/logo-1024.png"
SIZES=(16 32 48 64 128 256 512)

echo "==> icon-*.png"
for s in "${SIZES[@]}"; do
  magick "${MASTER}" -resize "${s}x${s}" "${OUT}/icon-${s}.png"
done
# convenience 256 copy
cp -f "${OUT}/icon-256.png" "${OUT}/icon.png"

echo "==> icon.ico (16/32/48/256)"
# ImageMagick packs multi-resolution ICO from the listed PNGs
magick \
  "${OUT}/icon-16.png" \
  "${OUT}/icon-32.png" \
  "${OUT}/icon-48.png" \
  "${OUT}/icon-256.png" \
  "${OUT}/icon.ico"

echo "==> social-banner.png 1280x640"
# Canvas + left accent bar + monogram + name/tagline
# annotate uses the bold font; gravity west after placing mark on left
MARK_SZ=280
MARK_X=80
MARK_Y=$(( (640 - MARK_SZ) / 2 ))
TEXT_X=$(( MARK_X + MARK_SZ + 48 ))

magick -size 1280x640 "xc:${BG}" \
  -fill "${ACCENT}" -draw "rectangle 0,0 16,640" \
  \( "${MASTER}" -resize "${MARK_SZ}x${MARK_SZ}" \) \
  -geometry "+${MARK_X}+${MARK_Y}" -composite \
  -font "${FONT}" -fill "#F0F0F0" -pointsize 72 \
  -gravity Northwest -annotate "+${TEXT_X}+240" "${NAME}" \
  -font "${FONT}" -fill "${ACCENT}" -pointsize 36 \
  -gravity Northwest -annotate "+${TEXT_X}+340" "${TAGLINE}" \
  "${OUT}/social-banner.png"

echo "==> README-header.png 1280x320"
magick "${OUT}/social-banner.png" -resize 1280x320! "${OUT}/README-header.png"

echo "==> done -> ${OUT}"
ls -la "${OUT}/logo.svg" "${OUT}/logo-1024.png" "${OUT}/icon.ico" \
  "${OUT}/icon.png" "${OUT}/social-banner.png" "${OUT}/README-header.png"

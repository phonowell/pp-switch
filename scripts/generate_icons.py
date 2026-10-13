"""Regenerate packaged icons from the SVG; CairoSVG/Pillow are build tools only."""
import io
from pathlib import Path
import struct

import cairosvg
from PIL import Image

assets = Path(__file__).resolve().parents[1] / "assets"
source = (assets / "icon.svg").read_bytes()


def image(size):
    # Supersample small sizes so all formats share smooth edges and real alpha.
    png = cairosvg.svg2png(bytestring=source, output_width=size * 2, output_height=size * 2)
    return Image.open(io.BytesIO(png)).convert("RGBA").resize(
        (size, size), Image.Resampling.LANCZOS
    )


def png(size):
    output = io.BytesIO()
    # Two flat colors need only a small indexed palette, including alpha edges.
    raster = image(size)
    indexed = raster.quantize(
        colors=256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE
    )
    # Quantization averages edge pixels into solid palette entries. Restore
    # the SVG's exact opaque brand blue and white, while keeping alpha edges.
    palette = indexed.getpalette("RGBA")
    for color in ((54, 93, 216, 255), (255, 255, 255, 255)):
        # Tiny representations may have no perfectly solid source pixel.
        index = min(
            range(len(palette) // 4),
            key=lambda i: sum((palette[i * 4 + channel] - color[channel]) ** 2
                              for channel in range(4)),
        )
        palette[index * 4:index * 4 + 4] = color
    indexed.putpalette(palette, rawmode="RGBA")
    indexed.save(output, format="PNG", optimize=True)
    return output.getvalue()


(assets / "icon.png").write_bytes(png(1024))
(assets / "icon-64.rgba").write_bytes(image(64).tobytes())
image(256).save(
    assets / "icon.ico", sizes=[(size, size) for size in (16, 24, 32, 48, 64, 128, 256)]
)

# PNG-backed ICNS entries, including the standard Retina representations.
representations = (
    (b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024),
    (b"ic11", 32), (b"ic12", 64), (b"ic13", 256), (b"ic14", 512),
)
chunks = [tag + struct.pack(">I", len(data) + 8) + data for tag, size in representations for data in [png(size)]]
toc = b"".join(chunk[:8] for chunk in chunks)
payload = b"TOC " + struct.pack(">I", len(toc) + 8) + toc + b"".join(chunks)
(assets / "icon.icns").write_bytes(b"icns" + struct.pack(">I", len(payload) + 8) + payload)
for name in ("icon.svg", "icon.png", "icon-64.rgba", "icon.ico", "icon.icns"):
    print(f"{name}: {(assets / name).stat().st_size} bytes")

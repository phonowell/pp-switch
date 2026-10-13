"""Check captured pixels for stable icons, row actions and complete localized badges.

Uses only the standard library. Native title-icon and interaction properties are checked
by input_smoke; this gate checks rendered output rather than source constants.
"""
import argparse
import json
from pathlib import Path


def ppm(path):
    magic, dimensions, maximum, pixels = path.read_bytes().split(b"\n", 3)
    width, height = map(int, dimensions.split())
    assert magic == b"P6" and maximum == b"255"
    assert len(pixels) == width * height * 3, f"Incomplete capture: {path}"
    return width, height, pixels


def pixel(image, x, y):
    width, _, pixels = image
    start = (y * width + x) * 3
    return tuple(pixels[start:start + 3])


def icon(image, left, top, width, height):
    return tuple(pixel(image, x, y) for y in range(top, top+height)
                 for x in range(left, left+width))


def borderless(image, background):
    width, height, _ = image
    return all(pixel(image,x,y) == background
               for x,y in ([(x,y) for x in range(width) for y in (0,height-1)]
                           + [(x,y) for y in range(height) for x in (0,width-1)]))


def clean_margin(image):
    background = pixel(image, 0, 0)
    width, height, _ = image
    return all(pixel(image, x, y) == background for y in range(480, height)
               for x in range(width))


def badge_mask(image, bounds=None):
    width, height, _ = image
    # Cocoa text rasterization applies color conversion/antialiasing. Compare
    # the captured blue glyph silhouette against a reference from this run,
    # rather than assuming its pixels equal the requested RGB paint color.
    left, top, right, bottom = bounds or (0, 0, width, height)
    background = pixel(image, 0, 0)
    background_chroma = background[2] - background[0]
    colored = {}
    for y in range(top, bottom):
        for x in range(left, right):
            red, green, blue = pixel(image, x, y)
            if blue > red + 40 and blue > green + 25:
                colored[x, y] = blue - red - background_chroma
    assert colored, "Active-provider badge has no rendered ink"
    # Normalize glyph coverage against the captured background. Thin English
    # strokes can contain no solid pixels; their most frequent RGB changes
    # between the reference and the plain row even with identical silhouettes.
    threshold = max(colored.values()) * 0.5
    points = {point for point, strength in colored.items() if strength > threshold}
    left = min(x for x, _ in points)
    top = min(y for _, y in points)
    return {(x - left, y - top) for x, y in points}


def validate(directory):
    results = json.loads((directory / "results.json").read_text())
    assert len(results) == 4 and all(item.get("captured") for item in results), "Incomplete language style evidence"
    baseline = None
    for locale in ("zh", "en", "zh-TW", "ja"):
        empty = ppm(directory / f"{locale}-empty.ppm")
        background = pixel(empty, 0, 0)
        gear = icon(empty, 388, 31, 24, 24)
        assert len(set(gear)) > 1, f"{locale}: missing Settings icon"
        if baseline is None: baseline = gear
        assert gear == baseline, f"{locale}: Settings icon changed with language"
        assert borderless(ppm(directory / f"{locale}-language-popup.ppm"), background), f"{locale}: language popup border remains"
        assert clean_margin(empty), f"{locale}: bottom status/Switch area remains"
        rows = ppm(directory / f"{locale}-rows.ppm")
        active = ppm(directory / f"{locale}-active.ppm")
        for name, image in (("rows", rows), ("active", active)):
            for left in (341, 377):
                pixels = icon(image, left+5, 222, 20, 20)
                assert len(set(pixels)) > 1, f"{locale}/{name}: missing row action at {left}"
            assert clean_margin(image), f"{locale}/{name}: removed footer remains"
        reference = badge_mask(ppm(directory / f"{locale}-badge-reference.ppm"))
        badge_width = max(x for x, _ in reference) + 1
        for state in ("active", "overflow", "scrolled"):
            actual = ppm(directory / f"{locale}-{state}.ppm")
            # Only the badge: names and addresses now share its blue ink.
            # Overflow reserves 12px for the scrollbar; scroll moves this row.
            right = actual[0] - 24 - 75 - 8 - (12 if state != "active" else 0)
            top = 198 - (24 if state == "scrolled" else 0)
            bounds = (right - badge_width - 8, top, right + 2, top + 69)
            assert badge_mask(actual, bounds) == reference, f"{locale}/{state}: badge clipped or covered"
        settings = ppm(directory / f"{locale}-settings.ppm")
        assert all(pixel(settings,x,y) == background for x,y in ((196,84),(415,84),(196,119),(415,119))), f"{locale}: selector corners retain stale ink"
        for state in ("settings", "update-busy", "update-error", "add", "edit"):
            assert (directory / f"{locale}-{state}.ppm").exists(), f"Missing {locale}/{state}"
        print(f"STYLE {locale}: stable Settings icon, Edit/Switch visible, no footer, complete badge with overflow passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    validate(parser.parse_args().directory)

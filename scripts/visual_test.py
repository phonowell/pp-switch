"""Regression detector sensitivity, independent of production drawing code."""
import unittest
from visual_check import badge_mask, borderless, clean_margin, icon


def image(width, height, points=(), color=(57, 121, 95)):
    pixels = bytearray(bytes((247, 248, 250)) * width * height)
    for x, y in points:
        start = (y * width + x) * 3
        pixels[start:start + 3] = bytes(color)
    return width, height, bytes(pixels)


class VisualDetectorTests(unittest.TestCase):
    def test_added_popup_border_is_rejected(self):
        self.assertTrue(borderless(image(100,140), (247,248,250)))
        self.assertFalse(borderless(image(100,140,[(50,0)]), (247,248,250)))

    def test_removed_footer_ink_is_rejected(self):
        self.assertTrue(clean_margin(image(440, 502)))
        self.assertFalse(clean_margin(image(440, 502, [(40, 483)], (97,106,121))))

    def test_locale_icon_changes_are_detected(self):
        original = image(440, 502, [(393, 36)], (97, 106, 121))
        changed = image(440, 502, [(394, 36)], (97, 106, 121))
        self.assertNotEqual(icon(original,388,31,24,24), icon(changed,388,31,24,24))

    def test_missing_action_has_no_ink(self):
        self.assertEqual(len(set(icon(image(440,502),346,222,20,20))), 1)
        self.assertGreater(len(set(icon(image(440,502,[(355,232)]),346,222,20,20))), 1)

    def test_badge_position_is_allowed_but_missing_ink_is_rejected(self):
        original = badge_mask(image(20, 20, [(3, 4), (5, 4), (5, 5)]))
        moved = badge_mask(image(20, 20, [(8, 9), (10, 9), (10, 10)]))
        covered = badge_mask(image(20, 20, [(3, 4), (5, 4)]))
        self.assertEqual(original, moved)
        self.assertNotEqual(original, covered)

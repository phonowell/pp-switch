"""Regression detector sensitivity, independent of production drawing code."""
import unittest
from visual_check import badge_mask, borderless, clean_margin, icon


def image(width, height, points=(), color=(69, 118, 224)):
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

    def test_retired_green_and_neutral_ink_cannot_pass_blue_badge_check(self):
        for color in [(57, 121, 95), (97, 106, 121)]:
            with self.subTest(color=color), self.assertRaisesRegex(AssertionError, "no rendered ink"):
                badge_mask(image(20, 20, [(3, 4)], color))

    def test_blue_text_outside_badge_cannot_hide_missing_badge_ink(self):
        actual = image(20, 20, [(1, 1), (12, 4), (14, 4), (14, 5)])
        expected = badge_mask(image(20, 20, [(3, 4), (5, 4), (5, 5)]))
        self.assertEqual(badge_mask(actual, (10, 0, 20, 20)), expected)
        with self.assertRaisesRegex(AssertionError, "no rendered ink"):
            badge_mask(image(20, 20, [(1, 1)]), (10, 0, 20, 20))

    def test_antialiased_badge_survives_background_change(self):
        masks = []
        for background in [(247, 248, 250), (233, 239, 252)]:
            pixels = bytearray(bytes(background) * 20 * 20)
            for (x, y), alpha in [((3, 4), 0.6), ((5, 4), 1), ((5, 5), 0.35)]:
                color = bytes(round(bg + alpha * (fg - bg))
                              for bg, fg in zip(background, (69, 118, 224)))
                start = (y * 20 + x) * 3
                pixels[start:start + 3] = color
            masks.append(badge_mask((20, 20, bytes(pixels))))
        self.assertEqual(masks[0], masks[1])
        self.assertEqual(masks[0], {(0, 0), (2, 0)})

# Tests of the landing page builder: python3 -m unittest discover -s site/tests -v
import json
import re
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / "site"))
import build  # noqa: E402


def release():
    return json.loads((HERE / "release.json").read_text(encoding="utf-8"))


def theme():
    with open(ROOT / "assets/theme.toml", "rb") as f:
        return tomllib.load(f)


class ChangelogTest(unittest.TestCase):
    def test_sections_from_real_changelog(self):
        md = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        versions = [s["version"] for s in build.changelog_sections(md, 50)]
        self.assertIn("0.1.0", versions)

    def test_md_list_with_continuation(self):
        out = build.md_to_html("### Added\n\n- The **Reset** button: one\n  two\n- three\n")
        self.assertEqual(out, "<h4>Added</h4>\n<ul><li>The <strong>Reset</strong> button: one two</li><li>three</li></ul>")

    def test_md_escapes_html_in_code(self):
        self.assertIn("<code>a &lt; b &amp;</code>", build.md_to_html("- `a < b &`\n"))

    def test_unreleased_versions_are_not_news(self):
        md = "## [0.2.0] — 2026-04-01\n\n- later\n\n## [0.1.0] — 2026-03-12\n\n- first\n"
        t = json.loads((ROOT / "site/i18n/en.json").read_text(encoding="utf-8"))
        out = build.news_html(md, t, "0.1.0")
        self.assertIn("first", out)
        self.assertNotIn("later", out)


class ReleaseTest(unittest.TestCase):
    def test_all_kinds_found(self):
        a = build.release_assets(release())
        self.assertEqual(set(a), set(build.ASSET_KINDS))
        self.assertEqual(a["win-msi"]["name"], "keyprobe-0.1.0-windows-x64.msi")

    def test_missing_asset_fails(self):
        r = release()
        r["assets"] = [a for a in r["assets"] if not a["name"].endswith(".dmg")]
        with self.assertRaises(SystemExit) as e:
            build.release_assets(r)
        self.assertIn("macos-universal.dmg", str(e.exception))

    def test_sizes(self):
        self.assertEqual(build.format_size(712), "1 KB")
        self.assertEqual(build.format_size(11534336), "11.0 MB")


class PageTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name) / "site"
        build.build(release(), self.out)
        self.page = (self.out / "index.html").read_text(encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def test_no_placeholder_left(self):
        self.assertNotRegex(self.page, r"\{\{")

    def test_links_come_from_the_release(self):
        for a in release()["assets"]:
            self.assertIn(a["url"], self.page)
        self.assertIn("Version 0.1.0, released March 12, 2026", self.page)

    def test_no_third_party_requests(self):
        for url in re.findall(r'(?:src|href)="(https?://[^"]+)"', self.page):
            self.assertTrue(
                url.startswith(("https://github.com/nazdravan/keyprobe", build.SITE_URL)),
                url,
            )

    def test_every_local_file_exists(self):
        for ref in re.findall(r'(?:src|href)="(assets/[^"#]+)"', self.page):
            self.assertTrue((self.out / ref).is_file(), ref)
        css = (self.out / "assets/theme.css").read_text(encoding="utf-8")
        for ref in re.findall(r'url\("([^"]+)"\)', css):
            self.assertTrue((self.out / "assets" / ref).is_file(), ref)

    def test_social_and_canonical(self):
        self.assertIn('rel="canonical" href="https://nazdravan.github.io/keyprobe/"', self.page)
        self.assertIn('property="og:image" content="https://nazdravan.github.io/keyprobe/assets/og.png"', self.page)
        self.assertIn('hreflang="en"', self.page)


class ThemeTest(unittest.TestCase):
    def test_css_variables_match_theme_toml(self):
        t = theme()
        css = build.theme_css(t)
        light, dark = css.split("@media (prefers-color-scheme: dark)")
        for var, (lkey, dkey) in build.THEME_VARS.items():
            self.assertIn(f"{var}: {t[lkey]};", light)
            self.assertIn(f"{var}: {t[dkey]};", dark)
        self.assertIn(t["font"], css)

    def test_style_has_no_colours_of_its_own(self):
        css = (ROOT / "site/assets/style.css").read_text(encoding="utf-8")
        self.assertNotRegex(css, r"#[0-9a-fA-F]{3,8}\b|\brgba?\(|\bhsla?\(")


if __name__ == "__main__":
    unittest.main()

from pathlib import Path
import re
import unittest
from html.parser import HTMLParser

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"


class PageParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.ids = set()
        self.links = []
        self.assets = []
        self.langs = []
        self.csp = []

    def handle_starttag(self, tag, attrs):
        data = dict(attrs)
        if "id" in data:
            self.ids.add(data["id"])
        if tag == "a" and data.get("href"):
            self.links.append(data["href"])
        if tag in {"script", "link"}:
            src = data.get("src") or (data.get("href") if tag == "link" else None)
            if src:
                self.assets.append(src)
        if tag == "html":
            self.langs.append((data.get("lang"), data.get("dir")))
        if tag == "meta" and data.get("http-equiv", "").lower() == "content-security-policy":
            self.csp.append(data.get("content", ""))


class PublicSiteTests(unittest.TestCase):
    def test_required_static_files_exist(self):
        for name in ("index.html", "404.html", "styles.css", "app.js"):
            self.assertTrue((SITE / name).is_file(), name)

    def test_pages_artifact_has_only_curated_site_folder(self):
        workflow = (ROOT / ".github/workflows/pages.yml").read_text()
        self.assertRegex(workflow, r"upload-pages-artifact[^\n]*\n\s+with:\n\s+path: site/")
        self.assertNotIn("deploy/docker-compose", workflow)

    def test_only_curated_workflow_deploys_to_pages(self):
        workflow_files = list((ROOT / ".github/workflows").glob("*.yml"))
        workflow_files += list((ROOT / ".github/workflows").glob("*.yaml"))
        deployers = sorted(
            path.name for path in workflow_files
            if "actions/deploy-pages@" in path.read_text()
        )
        self.assertEqual(deployers, ["pages.yml"])

    def test_html_languages_and_csp(self):
        for filename in ("index.html", "404.html"):
            parser = PageParser()
            parser.feed((SITE / filename).read_text())
            self.assertEqual(parser.langs, [("ar", "rtl")], filename)
            self.assertTrue(parser.csp, filename)
            self.assertIn("connect-src 'none'", parser.csp[0], filename)
            self.assertIn("default-src 'self'", parser.csp[0], filename)

    def test_assets_are_local_and_local_links_resolve(self):
        parser = PageParser()
        parser.feed((SITE / "index.html").read_text())
        for asset in parser.assets:
            self.assertFalse(asset.startswith(("http:", "https:", "//")), asset)
            self.assertTrue((SITE / asset.removeprefix("./")).is_file(), asset)
        for link in parser.links:
            if link.startswith("#"):
                self.assertIn(link[1:], parser.ids, link)
            elif not link.startswith(("https://", "mailto:", "tel:")):
                target = link.split("#", 1)[0] or "index.html"
                self.assertTrue((SITE / target.removeprefix("./")).exists(), link)

    def test_no_backend_connections_or_credentials(self):
        source = "\n".join(p.read_text() for p in SITE.rglob("*") if p.is_file())
        self.assertNotRegex(source, r"(?i)(postgres(?:ql)?://|nats://|GULA_ACCESS_TOKEN|JWT_SECRET|BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY)")
        parser = PageParser()
        parser.feed((SITE / "index.html").read_text())
        self.assertIn("connect-src 'none'", parser.csp[0])

    def test_admin_console_is_a_synthetic_static_demo(self):
        page = SITE / "console/index.html"
        js = (SITE / "console/app.js").read_text()
        html = page.read_text()
        parser = PageParser()
        parser.feed(html)
        self.assertEqual(parser.langs, [("ar", "rtl")])
        self.assertIn("connect-src 'none'", parser.csp[0])
        self.assertIn('name="robots" content="noindex, nofollow"', html)
        self.assertIn("EMP-DEMO-001", js)
        self.assertIn("بيانات اصطناعية", html)
        self.assertNotRegex(js, r"\b(fetch|XMLHttpRequest|WebSocket)\b")
        for asset in parser.assets:
            self.assertFalse(asset.startswith(("http:", "https:", "//")), asset)
            self.assertTrue((page.parent / asset.removeprefix("./")).is_file(), asset)

    def test_public_portal_links_to_demo_console(self):
        html = (SITE / "index.html").read_text()
        self.assertIn('href="./console/"', html)
        self.assertIn("translations.en.consolePreview", (SITE / "app.js").read_text())

    def test_bilingual_controls_and_synthetic_demo_present(self):
        html = (SITE / "index.html").read_text()
        js = (SITE / "app.js").read_text()
        self.assertIn('id="language-toggle"', html)
        self.assertIn('id="theme-toggle"', html)
        self.assertIn('id="sync-demo"', html)
        self.assertIn("const translations={", js)
        self.assertIn("DEMO-001", js)
        self.assertIn("localStorage", js)

    def test_crawler_metadata_and_robots_are_present(self):
        html = (SITE / "index.html").read_text()
        self.assertIn('<meta name="description"', html)
        self.assertIn('<meta property="og:title"', html)
        self.assertIn('<meta name="twitter:card"', html)
        self.assertIn("translations[next].pageTitle", (SITE / "app.js").read_text())
        self.assertIn("User-agent: *\nAllow: /", (SITE / "robots.txt").read_text())
        self.assertFalse(re.search(r'<link rel="canonical"', html, re.I))


if __name__ == "__main__":
    unittest.main()

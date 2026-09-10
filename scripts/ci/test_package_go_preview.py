"""Validate preview provenance gates and cache separation without publishing."""

import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "package_go_preview", Path(__file__).with_name("package-go-preview.py")
)
preview = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preview)


class GoPreviewTests(unittest.TestCase):
    def test_rejects_unchecked_or_unrelated_artifacts(self):
        checked = {
            "head_sha": "a" * 40,
            "conclusion": "success",
            "path": ".github/workflows/check.yml",
            "head_repository": {"full_name": preview.REPOSITORY},
            "head_branch": "releases/build-pr-1537",
            "event": "push",
        }
        self.assertEqual(preview.validate_check(checked, checked["head_branch"]), "a" * 40)
        for key, value in (
            ("head_sha", "main"),
            ("conclusion", "failure"),
            ("conclusion", None),
            ("path", ".github/workflows/release.yml"),
            ("head_repository", {"full_name": "someone/fork"}),
            ("head_branch", "main"),
            ("event", "pull_request"),
        ):
            with self.subTest(key=key, value=value):
                with self.assertRaises(ValueError):
                    preview.validate_check({**checked, key: value}, checked["head_branch"])

    def test_go_qualification_only_relaxes_the_overall_conclusion(self):
        checked = {
            "head_sha": "a" * 40,
            "conclusion": "failure",
            "path": ".github/workflows/check.yml",
            "head_repository": {"full_name": preview.REPOSITORY},
            "head_branch": "releases/build-pr-1537",
            "event": "push",
        }
        with self.assertRaises(ValueError):
            preview.validate_check(checked, checked["head_branch"])
        self.assertEqual(preview.validate_check(
            checked, checked["head_branch"], go_qualified=True), "a" * 40)
        for key, value in (("event", "pull_request"), ("head_sha", "main"),
                           ("head_repository", {"full_name": "someone/fork"})):
            with self.subTest(key=key):
                with self.assertRaises(ValueError):
                    preview.validate_check({**checked, key: value}, checked["head_branch"],
                                           go_qualified=True)

    def test_preview_revisions_use_distinct_cache_paths(self):
        source = (Path(__file__).parents[2] / "sdk/go/setup.go").read_text()
        first = preview.preview_setup(source, "a" * 40)
        second = preview.preview_setup(first, "b" * 40)
        self.assertIn('"preview-' + "a" * 40 + '"', first)
        self.assertNotIn('"preview-' + "a" * 40 + '"', second)
        self.assertIn('"preview-' + "b" * 40 + '"', second)
        # Preview packaging does not repoint the release downloader.
        self.assertEqual(
            source.split('const sdkVersion = ')[1].splitlines()[0],
            second.split('const sdkVersion = ')[1].splitlines()[0],
        )

    def test_unknown_or_ambiguous_cache_layout_requires_review(self):
        with self.assertRaises(ValueError):
            preview.preview_setup("new cache implementation", "a" * 40)
        line = 'libDir := filepath.Join(dir, "lib", "v"+sdkVersion)\n'
        with self.assertRaises(ValueError):
            preview.preview_setup(line * 2, "a" * 40)


if __name__ == "__main__":
    unittest.main()

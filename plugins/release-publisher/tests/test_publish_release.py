from __future__ import annotations

import importlib.util
import pathlib
import unittest


MODULE_PATH = pathlib.Path(__file__).resolve().parents[1] / "scripts" / "publish_release.py"
SPEC = importlib.util.spec_from_file_location("publish_release", MODULE_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class PublishReleaseTests(unittest.TestCase):
    def test_next_patch_version_for_stable_release(self) -> None:
        self.assertEqual(MODULE.next_patch_version("0.1.5"), "0.1.6")

    def test_next_patch_version_rejects_prerelease(self) -> None:
        with self.assertRaises(MODULE.ReleaseError):
            MODULE.next_patch_version("0.2.0-rc.1")

    def test_parse_github_repository_https(self) -> None:
        self.assertEqual(
            MODULE.parse_github_repository("https://github.com/xjeway/mcp-manager.git"),
            ("xjeway", "mcp-manager"),
        )

    def test_parse_github_repository_ssh(self) -> None:
        self.assertEqual(
            MODULE.parse_github_repository("git@github.com:xjeway/mcp-manager.git"),
            ("xjeway", "mcp-manager"),
        )

    def test_release_run_matches_tag_push_release_workflow(self) -> None:
        run = {
            "name": "Release",
            "path": ".github/workflows/release.yml",
            "event": "push",
            "head_branch": "v0.1.6",
        }
        self.assertTrue(MODULE.release_run_matches(run, "v0.1.6"))
        self.assertFalse(MODULE.release_run_matches(run, "v0.1.7"))

    def test_validate_latest_manifest_requires_tagged_urls(self) -> None:
        manifest_text = """
        {
          "version": "0.1.6",
          "platforms": {
            "darwin-x86_64": {
              "url": "https://github.com/xjeway/mcp-manager/releases/download/v0.1.6/MCP.Manager_x64.app.tar.gz"
            }
          }
        }
        """
        manifest = MODULE.validate_latest_manifest(manifest_text, "0.1.6", "v0.1.6")
        self.assertEqual(manifest["version"], "0.1.6")

    def test_validate_latest_manifest_rejects_untagged_urls(self) -> None:
        manifest_text = """
        {
          "version": "0.1.6",
          "platforms": {
            "darwin-x86_64": {
              "url": "https://github.com/xjeway/mcp-manager/releases/download/untagged-123/latest.json"
            }
          }
        }
        """
        with self.assertRaises(MODULE.ReleaseError):
            MODULE.validate_latest_manifest(manifest_text, "0.1.6", "v0.1.6")


if __name__ == "__main__":
    unittest.main()

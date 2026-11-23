import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


GENERATOR = Path(__file__).with_name("generate_inventory.py")


def export_document():
    return {
        "meta": {"commit": "test-commit", "commitDate": "2026-01-01T00:00:00Z"},
        "data": {
            "actions": {"action": {}},
            "values": {
                "second": {"en-US": "Second", "zh-CN": "第二"},
                "first": {"en-US": "First", "zh-CN": "第一"},
            },
            "gamemodes": {},
            "maps": {},
            "heroes": {},
            "customGameSettings": {
                "main": {
                    "en-US": "Main",
                    "zh-CN": "主选项",
                    "values": {
                        "one": {"en-US": "%1$s", "zh-CN": "%1$s"},
                        "two": {"en-US": "%1$s", "zh-CN": "%1$s"},
                    },
                },
                "lobby": {
                    "en-US": "Lobby",
                    "zh-CN": "大厅",
                    "values": {"one": {"en-US": "%1$s", "zh-CN": "%1$s"}},
                },
            },
        },
    }


def generate(document):
    raw = json.dumps(document, ensure_ascii=False).encode()
    with tempfile.TemporaryDirectory() as directory:
        export = Path(directory) / "export.json"
        output = Path(directory) / "inventory.json"
        export.write_bytes(raw)
        result = subprocess.run(
            [
                sys.executable,
                "-I",
                "-B",
                str(GENERATOR),
                "--export",
                str(export),
                "--output",
                str(output),
                "--expected-commit",
                "test-commit",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        return result, output.read_bytes() if output.exists() else None, raw


class InventoryGenerationTests(unittest.TestCase):
    def test_regenerates_groups_without_collecting_nested_labels(self):
        result, output, raw = generate(export_document())
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = {
            "schemaVersion": 1,
            "source": {
                "commit": "test-commit",
                "commitDate": "2026-01-01T00:00:00Z",
                "sha256": hashlib.sha256(raw).hexdigest(),
            },
            "counts": {
                "actions": 1,
                "values": 2,
                "gamemodes": 0,
                "maps": 0,
                "heroes": 0,
                "settingsGroups": 2,
            },
            "entries": {
                "values": [
                    {
                        "id": "first",
                        "en-US": "First",
                        "source": "data.values.first",
                        "zh-CN": "第一",
                    },
                    {
                        "id": "second",
                        "en-US": "Second",
                        "source": "data.values.second",
                        "zh-CN": "第二",
                    },
                ],
                "gamemodes": [],
                "maps": [],
                "heroes": [],
            },
            "settingsGroups": [
                {
                    "en-US": "Lobby",
                    "source": "data.customGameSettings.lobby",
                    "zh-CN": "大厅",
                },
                {
                    "en-US": "Main",
                    "source": "data.customGameSettings.main",
                    "zh-CN": "主选项",
                },
            ],
        }
        self.assertEqual(
            output, (json.dumps(expected, ensure_ascii=False, indent=2) + "\n").encode()
        )

    def test_conflicting_settings_group_aliases_are_rejected(self):
        for locale in ("en-US", "zh-CN"):
            with self.subTest(locale=locale):
                document = export_document()
                groups = document["data"]["customGameSettings"]
                groups["lobby"][locale] = groups["main"][locale]
                result, output, _ = generate(document)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"settingsGroups: duplicate localized aliases: {{'{locale}:",
                    result.stderr,
                )
                self.assertIn("data.customGameSettings.main", result.stderr)
                self.assertIn("data.customGameSettings.lobby", result.stderr)
                self.assertIsNone(output)

    def test_conflicting_category_aliases_are_rejected(self):
        for category in ("values", "gamemodes", "maps", "heroes"):
            for locale in ("en-US", "zh-CN"):
                with self.subTest(category=category, locale=locale):
                    document = export_document()
                    entries = {
                        "first": {"en-US": "First", "zh-CN": "第一"},
                        "second": {"en-US": "Second", "zh-CN": "第二"},
                    }
                    entries["second"][locale] = entries["first"][locale]
                    document["data"][category] = entries
                    result, output, _ = generate(document)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(
                        f"{category}: duplicate localized aliases: {{'{locale}:",
                        result.stderr,
                    )
                    self.assertIn("'first'", result.stderr)
                    self.assertIn("'second'", result.stderr)
                    self.assertIsNone(output)


if __name__ == "__main__":
    unittest.main()

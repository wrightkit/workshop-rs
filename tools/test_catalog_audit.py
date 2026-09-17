import importlib.util
import json
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("catalog-audit.py")
SPEC = importlib.util.spec_from_file_location("catalog_audit", MODULE_PATH)
catalog_audit = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(catalog_audit)


def catalog_with(*, value_kind="values", arity=2, return_type="Array", enum_members=None):
    entry = {
        "id": "removeFromArray",
        "aliases": {"en-US": "Remove From Array"},
        "params": ["Array", "Value"][:arity],
        "paramTypes": ["Array", "Object | Array"][:arity],
    }
    if return_type is not None:
        entry["returnType"] = return_type
    domain = {
        "domain": "Color",
        "aliases": {"en-US": "Color"},
        "members": enum_members or [{"id": "RED", "aliases": {"en-US": "Red"}}],
    }
    return {
        "structural": [],
        "actions": [entry.copy()] if value_kind == "actions" else [],
        "values": [entry.copy()] if value_kind == "values" else [],
        "operators": [entry.copy()] if value_kind == "operators" else [],
        "events": [],
        "enums": [domain],
        "localizedStrings": [],
    }


def reference(**overrides):
    fact = {
        "kind": "value",
        "id": "removeFromArray",
        "title": "Remove From Array",
        "params": ["Array", "Value"],
        "paramTypes": ["Array", "Object | Array"],
        "returnType": "Array",
        "sourceUrl": "https://md.wrightkit.dev/wiki/articles/remove-from-array.md",
    }
    fact.update(overrides)
    return {"schemaVersion": 1, "source": {"baseUrl": "https://md.wrightkit.dev"}, "facts": [fact]}


class AuditTests(unittest.TestCase):
    def settings(self):
        return {"entries": {}}

    def test_clean_reference_and_catalog_are_independently_matchable(self):
        result = catalog_audit.audit(catalog_with(), reference(), self.settings())
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["coverage"]["surfaces"]["value"]["evidenced"], 1)

    def test_kind_signature_return_and_enum_membership_are_distinct_findings(self):
        bad_catalog = catalog_with(value_kind="operators", arity=1, return_type="Boolean", enum_members=[])
        result = catalog_audit.audit(bad_catalog, reference(), self.settings())
        kinds = {finding["mismatch"] for finding in result["findings"]}
        self.assertIn("wrong catalog kind/classification", kinds)
        self.assertIn("signature/arity mismatch", kinds)
        self.assertIn("return-type mismatch", kinds)

        enum_reference = {"schemaVersion": 1, "facts": [{
            "kind": "enum", "id": "Color", "domain": "Color", "title": "Color",
            "members": [{"id": "RED", "title": "Red"}, {"id": "BLUE", "title": "Blue"}],
            "sourceUrl": "https://md.wrightkit.dev/wiki/articles/color.md",
        }]}
        enum_result = catalog_audit.audit(catalog_with(), enum_reference, self.settings())
        self.assertIn("enum/operation membership mismatch", {f["mismatch"] for f in enum_result["findings"]})

    def test_operation_membership_detects_remove_from_array_value_leak(self):
        catalog = catalog_with()
        catalog["operators"] = [{"id": "removeFromArray", "aliases": {"en-US": "Remove From Array"}}]
        operation = {"schemaVersion": 1, "facts": [{
            "kind": "operation", "id": "Operation", "members": ["Remove From Array By Value"],
            "sourceUrl": "https://md.wrightkit.dev/wiki/articles/operation.md",
        }]}
        result = catalog_audit.audit(catalog, operation, self.settings())
        self.assertEqual(result["status"], "mismatch")
        self.assertIn("enum/operation membership mismatch", {f["mismatch"] for f in result["findings"]})

    def test_every_catalog_entry_gets_a_status_and_reference_only_is_explicit(self):
        catalog = catalog_with()
        reference_data = {"schemaVersion": 1, "facts": [{
            "kind": "value", "id": "missingValue", "title": "Missing Value",
            "sourceUrl": "https://md.wrightkit.dev/wiki/articles/missing-value.md",
        }]}
        result = catalog_audit.audit(catalog, reference_data, self.settings())
        self.assertEqual(result["coverage"]["catalogEntries"], 3)
        self.assertEqual(len([f for f in result["findings"] if f["status"] == "unverified"]), 3)
        self.assertEqual(len([f for f in result["findings"] if f["status"] == "reference-only"]), 1)

    def test_snapshot_is_valid_json_and_has_provenance_for_known_facts(self):
        snapshot = json.loads(Path(__file__).with_name("catalog-reference.json").read_text())
        self.assertEqual(snapshot["schemaVersion"], 1)
        self.assertTrue(snapshot["source"]["manifestUrl"].startswith("https://md.wrightkit.dev/"))
        self.assertTrue(snapshot["facts"])
        self.assertTrue(all(fact.get("sourceUrl", "").startswith("https://md.wrightkit.dev/") for fact in snapshot["facts"]))


if __name__ == "__main__":
    unittest.main()

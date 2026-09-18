import copy
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).parents[1]
spec = importlib.util.spec_from_file_location("catalog_tool", ROOT / "tools/catalog.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.pack = module.load(ROOT / "data/core.json")

    def test_seed_pack(self):
        self.assertEqual(len(self.pack["dishes"]), 72)

    def test_merge_identical_deduplicates(self):
        result = module.merge([self.pack, self.pack], "merged", "1")
        self.assertEqual(len(result["dishes"]), 72)

    def test_merge_conflict_rejected(self):
        other = copy.deepcopy(self.pack)
        other["dishes"][0]["name"] = "Changed"
        with self.assertRaises(ValueError):
            module.merge([self.pack, other], "merged", "1")

    def test_alias_conflict_rejected(self):
        self.pack["dishes"][0]["aliases"].append(self.pack["dishes"][1]["name"])
        with self.assertRaises(ValueError): module.validate(self.pack)

    def test_invalid_source_rejected(self):
        self.pack["dishes"][0]["sources"] = ["nonexistent"]
        with self.assertRaises(ValueError): module.validate(self.pack)

    def test_missing_knowledge_is_valid(self):
        self.pack["dishes"][0]["minutes_max"] = None
        self.pack["dishes"][0]["ingredients"] = []
        module.validate(self.pack)

    def test_invalid_numeric_fields(self):
        for value in [-1, True, 1.5, 2**32]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                pack = copy.deepcopy(self.pack)
                pack["dishes"][0]["minutes_max"] = value
                module.validate(pack)


if __name__ == "__main__": unittest.main()

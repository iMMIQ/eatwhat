import json
from pathlib import Path
import unittest

from eatwhat import recommend


class BindingTests(unittest.TestCase):
    def test_default_and_determinism(self):
        self.assertEqual(recommend(seed=42, count=5), recommend(seed=42, count=5))
        self.assertEqual(len(recommend()["items"]), 1)

    def test_constraints(self):
        result = recommend(count=100, mode="cook", max_minutes=25, budget_max_fen=1200)
        self.assertTrue(result["items"])
        for item in result["items"]:
            self.assertLessEqual(item["dish"]["minutes_max"], 25)
            self.assertLessEqual(item["dish"]["cost_max_fen"], 1200)

    def test_recent_alias(self):
        result = recommend(count=100, recent=["西红柿炒鸡蛋"])
        self.assertNotIn("番茄炒蛋", [i["dish"]["name"] for i in result["items"]])

    def test_input_errors(self):
        for args in [{"count": 0}, {"count": True}, {"seed": -1}, {"strategy": "bad"},
                     {"preferences": "酸辣"}, {"budget_max_fen": 1000},
                     {"preferences": [12]}, {"max_minutes": 1.5}]:
            with self.subTest(args=args), self.assertRaises(ValueError):
                recommend(**args)

    def test_no_match(self):
        self.assertEqual(recommend(categories=["missing"])["status"], "no_match")

    def test_custom_pack_path_and_dict(self):
        path = Path(__file__).parents[1] / "data" / "core.json"
        pack = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(recommend(catalog=path, seed=42), recommend(catalog=pack, seed=42))

    def test_incomplete_ingredients(self):
        self.assertEqual(recommend(exclude_ingredients=["花生"])["status"], "no_match")
        result = recommend(exclude_ingredients=["花生"], require_complete_ingredients=False)
        self.assertTrue(result["warnings"])


if __name__ == "__main__":
    unittest.main()

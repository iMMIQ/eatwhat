from eatwhat import recommend
import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

result = recommend(seed=42, count=3, mode="cook", max_minutes=30, preferences=["酸鲜"])
for item in result["items"]:
    print(item["dish"]["name"], item["reason_codes"])

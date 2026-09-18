from eatwhat import recommend

result = recommend(seed=42, count=3, mode="cook", max_minutes=30, preferences=["酸鲜"])
for item in result["items"]:
    print(item["dish"]["name"], item["reason_codes"])

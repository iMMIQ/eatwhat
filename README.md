# eatwhat · 今天吃什么

离线 Rust 推荐库，通过 Python 的一个 `recommend()` 接口嵌入聊天机器人、命令行、食堂工具等程序。运行时不调用 LLM、不联网、不保存用户历史。

当前版本：0.1.0。内置 **72 道、9 类**编辑种子菜品，支持替换和合并数据包。它不是全量菜谱库；批量采集、语义实体消歧、完整配料审核是后续数据建设工作。

## 安装与使用

从源码安装需要 Rust >= 1.82、C 编译/链接工具链和 Python >= 3.10。项目尚未发布到 PyPI，不要直接从 PyPI 安装同名包。

```bash
python -m venv .venv
# Linux/macOS: source .venv/bin/activate
# Windows: .venv\Scripts\activate
python -m pip install .
```

```python
from eatwhat import recommend

print(recommend()["items"][0]["dish"]["name"])

result = recommend(
    seed=42,
    count=3,
    mode="cook",
    max_minutes=30,
    budget_max_fen=2000,  # 每人食材成本上限估计，单位：人民币分
    preferences=["酸鲜", "清淡"],
    recent=["西红柿炒鸡蛋", "core_0001"],
)
for item in result["items"]:
    print(item["dish"]["name"], item["reason_codes"])

# 完整数据包，替换内置包；也接受 dict
print(recommend(catalog="data/core.json", seed=42))
```

Rust:

```rust
use eatwhat::{recommend, Options};
let result = recommend(&Options {
    count: 3,
    seed: Some(42),
    ..Default::default()
}).unwrap();
assert_eq!(result.items.len(), 3);
```

`recommend(&Options)` 是 Rust 推荐入口。`Catalog::from_json` 和 `Catalog::validate` 为数据辅助 API。

## 参数语义

所有参数可选。Python 使用关键字参数；拼错参数名会报错。

| 参数 | 默认值 | 语义 |
|---|---|---|
| `count` | 1 | 1–100，同一批无放回抽样 |
| `seed` | 随机生成 | u64；结果返回实际 seed |
| `strategy` | balanced | 见下方策略表 |
| `categories` / `regions` | 空 | 各列表内部 OR，不同条件之间 AND |
| `meal` | 无 | breakfast / lunch / dinner 或自定义数据标签 |
| `mode` | 无 | cook / eat_out 或自定义标签 |
| `theme` | 无 | quick_meal / rainy_day / comfort_food / weekend |
| `max_minutes` | 无 | 烹饪时长上限估计；未知时长排除，不是餐馆等待时间 |
| `budget_max_fen` | 无 | 每人食材成本上限估计，只接受 `mode="cook"`；未知成本排除 |
| `required_tags` | 空 | 全部满足的硬约束 |
| `preferences` | 空 | 匹配风味/食材标签时增加权重，uniform 忽略它 |
| `exclude_ingredients` | 空 | 排除明确记录的原料标签 |
| `require_complete_ingredients` | true | 有原料排除条件时，排除配料记录不完整的菜品 |
| `recent` | 空 | 菜品 ID、名称或别名，按从新到旧排列 |
| `exclude_recent` | true | 完全重复从候选中排除；false 时加权策略仅降权 |
| `catalog` | 内置包 | dict / JSON 路径，完整替换而非隐式追加 |

字符串匹配采用 trim + lowercase；不做任意模糊匹配、繁简转换或原料上位词推断。例如“坚果”不会自动匹配“花生”。内置 `regions` 暂为空，传地区会得到无匹配；可通过扩展包补齐。

**配料信息边界：**内置包只有常见主料，所有 `ingredients_complete=false`。因此默认的严格原料排除会返回 `no_match`。普通口味排除可显式传 `require_complete_ingredients=False`，结果会包含提示。此库不提供过敏安全承诺，不推断复合调料或交叉接触风险。

## 抽样方式

| 策略 | 行为 |
|---|---|
| uniform | 满足硬条件的菜品等概率，忽略软偏好和相似度 |
| balanced | 先选主类别，再在类别内加权选择 |
| comfort | balanced 基础上提高编辑熟悉度高的菜品权重 |
| explore | 提高熟悉度较低的权重，并加强近期相似度惩罚 |
| surprise | 更偏向不常见菜品，仍遵循全部硬约束 |

加权策略：偏好权重 × 熟悉度因子 × 近期相似度因子 × 本批相似度因子。
类别权重取组内菜品权重的**平均值**，不取总和，避免类别单纯因记录多而占优势。
在没有偏好、历史和批内结果时，balanced 的类别概率相等。
相似度基于类别、主料和标签；这是可解释的初版启发式，不是训练模型。
`familiarity` 是编辑先验，不是用户画像或实测热度。

相同数据内容、参数、算法版本和种子可复现。菜品先按稳定 ID 排序，输入记录顺序不影响结果。不保证不同数据版本或不同算法版本仍返回同一结果。

## 返回值和错误

返回 dict 包含 `items`、`status`、`candidate_count`、`requested_count`、`seed`、`dataset_id`、`dataset_version`、`algorithm_version`、`warnings`。
每项包含完整 `dish` 与规则产生的 `reason_codes`，不虚构生成式推荐理由。

- `ok`：满足数量。
- `partial`：候选不足，返回所有可选项，绝不补重复项。
- `no_match`：硬条件下为空，不放宽条件。
- 无效参数、数据冲突：Rust `Error` / Python `ValueError`；文件访问保留 Python `OSError`。
- 未知 recent 记录会产生 `UNKNOWN_RECENT` 警告。

## 数据工作流

版本化数据包是普通 JSON。`schema_version=1`，包包含 `id/version/sources/dishes`。
每道菜必须有稳定 ID、主类别、熟悉度和来源。允许缺失知识，不允许未知来源、重复 ID 或跨菜品的别名冲突。

```bash
python tools/catalog.py check data/core.json
python tools/catalog.py merge data/core.json my-region.json \
  --id combined --version 0.1.0 --output combined.json
```

合并只去除完全相同的 ID 记录；冲突明确报错。不会因名称相似就擅自合并地方变体。输出使用独占创建，避免覆盖源数据。
导入外部数据时先通过独立适配器转换成此结构，保留来源与许可，执行校验，再发布。维护数据与 Rust 代码可以分开迭代。
当前没有捆绑 RecipeNLG、Wikidata 或其他批量抓取数据；不代表已完成这些来源的许可审核。

数据完整性和编辑估计口径见 [data/PROVENANCE.md](data/PROVENANCE.md)。

## 开发验证

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --example basic
python -m pip install .
python -m unittest discover -s tests -p 'test_*.py' -v
python tools/catalog.py check data/core.json
```

CI 覆盖 Rust 引擎，以及 Linux/macOS/Windows 的 Python 3.10、3.13 原生扩展安装与调用。
纯 Rust 测试不启用 `python` feature，避免 PyO3 extension-module 的测试链接冲突。
发布前应运行 CI 并保留依赖锁定文件；本项目不自动发布到 crates.io 或 PyPI。

## 初版范围

已实现推荐引擎、PyO3 绑定、数据验证/合并、种子数据和测试。
后续可增加：有来源依据的地域数据包、批量采集适配器、人工审核队列、菜品/做法/套餐的独立实体、数据库索引。当前返回单道菜品；荤菜或汤不是自动组成的一整顿饭。

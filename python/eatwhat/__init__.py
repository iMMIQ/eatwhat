"""Single-entry offline food recommendation API, backed exclusively by Rust."""
from __future__ import annotations

import json as _json
from os import PathLike as _PathLike
from pathlib import Path as _Path
from typing import Any as _Any, Literal as _Literal, Sequence as _Sequence
from ._native import recommend_json as _recommend_json

__all__ = ["recommend"]


def recommend(
    *,
    count: int = 1,
    seed: int | None = None,
    strategy: _Literal["uniform", "balanced", "comfort", "explore", "surprise"] = "balanced",
    categories: _Sequence[str] = (),
    regions: _Sequence[str] = (),
    meal: str | None = None,
    mode: str | None = None,
    theme: str | None = None,
    max_minutes: int | None = None,
    budget_max_fen: int | None = None,
    exclude_ingredients: _Sequence[str] = (),
    require_complete_ingredients: bool = True,
    required_tags: _Sequence[str] = (),
    preferences: _Sequence[str] = (),
    recent: _Sequence[str] = (),
    exclude_recent: bool = True,
    catalog: dict[str, _Any] | str | _PathLike[str] | None = None,
) -> dict[str, _Any]:
    """Return a structured recommendation response.

    ``catalog`` is a complete pack dict or UTF-8 JSON file path and replaces the
    bundled catalog. No downloads, global user state or LLM calls are performed.
    ``budget_max_fen`` is ingredient cost per serving, only valid with mode='cook'.
    Invalid inputs raise ValueError; valid but unsatisfiable filters return no_match.
    Ingredient matching uses exact normalized labels, not an allergy ontology.
    """
    values = locals().copy()
    for key in ("categories", "regions", "exclude_ingredients", "required_tags", "preferences", "recent"):
        value = values[key]
        if isinstance(value, (str, bytes)) or not isinstance(value, _Sequence):
            raise ValueError(f"{key} must be a sequence of strings")
        if not all(isinstance(item, str) for item in value):
            raise ValueError(f"{key} must contain only strings")
        values[key] = list(value)
    if isinstance(catalog, (str, _PathLike)):
        values["catalog"] = _json.loads(_Path(catalog).read_text(encoding="utf-8"))
    return _json.loads(_recommend_json(_json.dumps(values, ensure_ascii=False, allow_nan=False)))

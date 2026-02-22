#!/usr/bin/env python3
"""Friendly CSV browser for `snaphu_translation_order.csv` using Polars."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Iterable

import polars as pl

STATUS_CHOICES = ["DONE", "NOT_PLANNED", "EMPTY"]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Filter and sort snaphu_translation_order.csv using Polars. "
            "Use --status to show only DONE/NOT PLANNED/EMPTY rows and --desc to "
            "see highest priority numbers last."
        )
    )
    parser.add_argument(
        "--csv",
        type=Path,
        default=Path("snaphu_translation_order.csv"),
        help="Path to the translation order CSV (default: snaphu_translation_order.csv)",
    )
    parser.add_argument(
        "--status",
        nargs="+",
        choices=STATUS_CHOICES,
        help=(
            "Filter by status. Repeatable; use EMPTY for blank status entries. "
            "Omit to show every row."
        ),
    )
    parser.add_argument(
        "--desc",
        action="store_true",
        help="Sort priorities in descending order (default ascending).",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Optional maximum number of rows to display.",
    )
    return parser.parse_args()


def filter_status(df: pl.DataFrame, statuses: Iterable[str] | None) -> pl.DataFrame:
    if not statuses:
        return df

    clean_status = pl.col("Status").fill_null("")
    expr = None
    for status in statuses:
        if status == "EMPTY":
            candidate = clean_status == ""
        else:
            candidate = clean_status == status
        expr = candidate if expr is None else (expr | candidate)

    return df.filter(expr)


def main() -> None:
    args = parse_args()
    if not args.csv.exists():
        raise SystemExit(f"CSV file not found: {args.csv}")

    df = pl.read_csv(args.csv)
    pl.Config.set_tbl_rows(-1)
    df = filter_status(df, args.status)
    df = df.sort("Priority", descending=args.desc)
    if args.limit is not None:
        df = df.head(args.limit)

    if df.is_empty():
        print("No rows match the requested filters.")
    else:
        print(df)


if __name__ == "__main__":
    main()

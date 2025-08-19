from __future__ import annotations

from typing import TYPE_CHECKING

import fancy_polars._reexport as pl

if TYPE_CHECKING:
    from fancy_polars import DataFrame, Expr, LazyFrame, Series
    from fancy_polars.fancy_polars import PyDataFrame, PyExpr, PyLazyFrame, PySeries


def wrap_df(df: PyDataFrame) -> DataFrame:
    return pl.DataFrame._from_pydf(df)


def wrap_ldf(ldf: PyLazyFrame) -> LazyFrame:
    return pl.LazyFrame._from_pyldf(ldf)


def wrap_s(s: PySeries) -> Series:
    return pl.Series._from_pyseries(s)


def wrap_expr(pyexpr: PyExpr) -> Expr:
    return pl.Expr._from_pyexpr(pyexpr)

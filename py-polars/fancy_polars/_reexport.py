"""Re-export Polars functionality to avoid cyclical imports."""

from fancy_polars.dataframe import DataFrame
from fancy_polars.expr import Expr, When
from fancy_polars.lazyframe import LazyFrame
from fancy_polars.schema import Schema
from fancy_polars.series import Series

__all__ = [
    "DataFrame",
    "Expr",
    "LazyFrame",
    "Schema",
    "Series",
    "When",
]

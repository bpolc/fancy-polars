from fancy_polars.dependencies import _HYPOTHESIS_AVAILABLE

if not _HYPOTHESIS_AVAILABLE:
    msg = (
        "fancy_polars.testing.parametric requires the 'hypothesis' module\n"
        "Please install it using the command: pip install hypothesis"
    )
    raise ModuleNotFoundError(msg)

from fancy_polars.testing.parametric.profiles import load_profile, set_profile
from fancy_polars.testing.parametric.strategies import (
    column,
    columns,
    create_list_strategy,
    dataframes,
    dtypes,
    lists,
    series,
)

__all__ = [
    # strategies
    "dataframes",
    "series",
    "column",
    "columns",
    "dtypes",
    "lists",
    "create_list_strategy",
    # profiles
    "load_profile",
    "set_profile",
]

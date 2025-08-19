//! Define the Polars exception hierarchy.

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyWarning};

// Errors
create_exception!(fancy_polars.exceptions, PolarsError, PyException);
create_exception!(fancy_polars.exceptions, ColumnNotFoundError, PolarsError);
create_exception!(fancy_polars.exceptions, ComputeError, PolarsError);
create_exception!(fancy_polars.exceptions, DuplicateError, PolarsError);
create_exception!(fancy_polars.exceptions, InvalidOperationError, PolarsError);
create_exception!(fancy_polars.exceptions, NoDataError, PolarsError);
create_exception!(fancy_polars.exceptions, OutOfBoundsError, PolarsError);
create_exception!(fancy_polars.exceptions, SQLInterfaceError, PolarsError);
create_exception!(fancy_polars.exceptions, SQLSyntaxError, PolarsError);
create_exception!(fancy_polars.exceptions, SchemaError, PolarsError);
create_exception!(
    fancy_polars.exceptions,
    SchemaFieldNotFoundError,
    PolarsError
);
create_exception!(fancy_polars.exceptions, ShapeError, PolarsError);
create_exception!(
    fancy_polars.exceptions,
    StringCacheMismatchError,
    PolarsError
);
create_exception!(
    fancy_polars.exceptions,
    StructFieldNotFoundError,
    PolarsError
);

// Warnings
create_exception!(fancy_polars.exceptions, PolarsWarning, PyWarning);
create_exception!(fancy_polars.exceptions, PerformanceWarning, PolarsWarning);
create_exception!(
    fancy_polars.exceptions,
    CategoricalRemappingWarning,
    PerformanceWarning
);
create_exception!(
    fancy_polars.exceptions,
    MapWithoutReturnDtypeWarning,
    PolarsWarning
);

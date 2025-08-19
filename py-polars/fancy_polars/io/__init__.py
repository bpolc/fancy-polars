"""Functions for reading data."""

from fancy_polars.io.avro import read_avro
from fancy_polars.io.clipboard import read_clipboard
from fancy_polars.io.csv import read_csv, read_csv_batched, scan_csv
from fancy_polars.io.database import read_database, read_database_uri
from fancy_polars.io.delta import read_delta, scan_delta
from fancy_polars.io.iceberg import scan_iceberg
from fancy_polars.io.ipc import read_ipc, read_ipc_schema, read_ipc_stream, scan_ipc
from fancy_polars.io.json import read_json
from fancy_polars.io.ndjson import read_ndjson, scan_ndjson
from fancy_polars.io.parquet import read_parquet, read_parquet_schema, scan_parquet
from fancy_polars.io.partition import (
    BasePartitionContext,
    KeyedPartition,
    KeyedPartitionContext,
    PartitionByKey,
    PartitionMaxSize,
    PartitionParted,
)
from fancy_polars.io.plugins import _defer as defer
from fancy_polars.io.pyarrow_dataset import scan_pyarrow_dataset
from fancy_polars.io.spreadsheet import read_excel, read_ods

__all__ = [
    "defer",
    "PartitionByKey",
    "PartitionMaxSize",
    "PartitionParted",
    "KeyedPartition",
    "BasePartitionContext",
    "KeyedPartitionContext",
    "read_avro",
    "read_clipboard",
    "read_csv",
    "read_csv_batched",
    "read_database",
    "read_database_uri",
    "read_delta",
    "read_excel",
    "read_ipc",
    "read_ipc_schema",
    "read_ipc_stream",
    "read_json",
    "read_ndjson",
    "read_ods",
    "read_parquet",
    "read_parquet_schema",
    "scan_csv",
    "scan_delta",
    "scan_iceberg",
    "scan_ipc",
    "scan_ndjson",
    "scan_parquet",
    "scan_pyarrow_dataset",
]

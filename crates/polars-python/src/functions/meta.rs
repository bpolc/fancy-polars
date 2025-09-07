use polars_core::POOL;
use polars_core::fmt::FloatFmt;
use polars_core::prelude::IDX_DTYPE;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::conversion::Wrap;

#[pyfunction]
pub fn get_index_type(py: Python) -> PyResult<Bound<PyAny>> {
    Wrap(IDX_DTYPE).into_pyobject(py)
}

#[pyfunction]
pub fn thread_pool_size() -> usize {
    POOL.current_num_threads()
}

#[pyfunction]
pub fn set_float_fmt(fmt: &str) -> PyResult<()> {
    let fmt = match fmt {
        "full" => FloatFmt::Full,
        "mixed" => FloatFmt::Mixed,
        e => {
            return Err(PyValueError::new_err(format!(
                "fmt must be one of {{'full', 'mixed'}}, got {e}",
            )));
        },
    };
    polars_core::fmt::set_float_fmt(fmt);
    Ok(())
}

#[pyfunction]
pub fn get_float_fmt() -> PyResult<String> {
    let strfmt = match polars_core::fmt::get_float_fmt() {
        FloatFmt::Full => "full",
        FloatFmt::Mixed => "mixed",
    };
    Ok(strfmt.to_string())
}

#[pyfunction]
#[pyo3(signature = (precision=None))]
pub fn set_float_precision(precision: Option<usize>) -> PyResult<()> {
    use polars_core::fmt::set_float_precision;
    set_float_precision(precision);
    Ok(())
}

#[pyfunction]
pub fn get_float_precision() -> PyResult<Option<usize>> {
    use polars_core::fmt::get_float_precision;
    Ok(get_float_precision())
}

#[pyfunction]
#[pyo3(signature = (sep=None))]
pub fn set_thousands_separator(sep: Option<char>) -> PyResult<()> {
    use polars_core::fmt::set_thousands_separator;
    set_thousands_separator(sep);
    Ok(())
}

#[pyfunction]
pub fn get_thousands_separator() -> PyResult<Option<String>> {
    use polars_core::fmt::get_thousands_separator;
    Ok(Some(get_thousands_separator()))
}

#[pyfunction]
#[pyo3(signature = (sep=None))]
pub fn set_decimal_separator(sep: Option<char>) -> PyResult<()> {
    use polars_core::fmt::set_decimal_separator;
    set_decimal_separator(sep);
    Ok(())
}

#[pyfunction]
pub fn get_decimal_separator() -> PyResult<Option<char>> {
    use polars_core::fmt::get_decimal_separator;
    Ok(Some(get_decimal_separator()))
}

#[pyfunction]
#[pyo3(signature = (trim=None))]
pub fn set_trim_decimal_zeros(trim: Option<bool>) -> PyResult<()> {
    use polars_core::fmt::set_trim_decimal_zeros;
    set_trim_decimal_zeros(trim);
    Ok(())
}

#[pyfunction]
pub fn get_trim_decimal_zeros() -> PyResult<Option<bool>> {
    use polars_core::fmt::get_trim_decimal_zeros;
    Ok(Some(get_trim_decimal_zeros()))
}

#[pyfunction]
pub fn set_default_regex_engine(engine: Option<&str>) -> PyResult<()> {
    use std::str::FromStr;

    use polars_utils::regex_adapter::{RegexEngine, set_default_regex_engine};

    let regex_engine = match engine {
        Some(s) => RegexEngine::from_str(s)
            .map_err(|_| PyValueError::new_err(format!("Regex engine '{s}' not supported.")))?,
        None => RegexEngine::default(),
    };
    set_default_regex_engine(regex_engine);
    Ok(())
}

#[pyfunction]
pub fn get_default_regex_engine() -> PyResult<String> {
    use polars_utils::regex_adapter::get_default_regex_engine;
    let regex_engine = get_default_regex_engine();
    Ok(regex_engine.as_ref().to_lowercase())
}

#[pyfunction]
#[pyo3(signature = (ttl_ms=None))]
pub fn set_regex_cache_ttl(ttl_ms: Option<usize>) -> PyResult<()> {
    use polars_utils::regex_cache::set_regex_cache_ttl_ms;
    set_regex_cache_ttl_ms(ttl_ms);
    Ok(())
}

#[pyfunction]
pub fn get_regex_cache_ttl() -> PyResult<usize> {
    use polars_utils::regex_cache::get_regex_cache_ttl_ms;
    Ok(get_regex_cache_ttl_ms())
}

#[pyfunction]
#[pyo3(signature = (capacity=None))]
pub fn set_regex_cache_capacity(capacity: Option<usize>) -> PyResult<()> {
    use polars_utils::regex_cache::set_global_regex_cache_capacity;
    set_global_regex_cache_capacity(capacity);
    Ok(())
}

#[pyfunction]
pub fn get_regex_cache_capacity() -> PyResult<usize> {
    use polars_utils::regex_cache::get_global_regex_cache_capacity;
    Ok(get_global_regex_cache_capacity())
}

#[pyfunction]
#[pyo3(signature = (capacity=None))]
pub fn set_local_regex_cache_capacity(capacity: Option<usize>) -> PyResult<()> {
    use polars_utils::regex_cache::set_local_regex_cache_capacity;
    set_local_regex_cache_capacity(capacity);
    Ok(())
}

#[pyfunction]
pub fn get_local_regex_cache_capacity() -> PyResult<usize> {
    use polars_utils::regex_cache::get_local_regex_cache_capacity;
    Ok(get_local_regex_cache_capacity())
}

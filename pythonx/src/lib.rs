mod navigation;
pub mod punc;

use pyo3::prelude::*;

// flate!(static DICT: str from "src/data/unionwords.txt");
static DICT: &str = include_str!("data/unionwords.txt");

#[pyfunction]
fn wordmotion_b(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_prev_start_of_word,
        navigation::index_last_start_of_word,
        true,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
#[allow(non_snake_case)]
fn wordmotion_B(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_prev_start_of_WORD,
        navigation::index_last_start_of_WORD,
        true,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
fn wordmotion_ge(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_prev_end_of_word,
        navigation::index_last_end_of_word,
        true,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
#[allow(non_snake_case)]
fn wordmotion_gE(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_prev_end_of_WORD,
        navigation::index_last_end_of_WORD,
        true,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
fn wordmotion_w(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_next_start_of_word,
        navigation::index_first_start_of_word,
        false,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
#[allow(non_snake_case)]
fn wordmotion_W(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_next_start_of_WORD,
        navigation::index_first_start_of_WORD,
        false,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
fn wordmotion_e(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_next_end_of_word,
        navigation::index_first_end_of_word,
        false,
        buffer,
        cursor_pos,
    )
}

#[pyfunction]
#[allow(non_snake_case)]
fn wordmotion_E(
    buffer: &Bound<'_, PyAny>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)> {
    navigation::navigate(
        navigation::index_next_end_of_WORD,
        navigation::index_first_end_of_WORD,
        false,
        buffer,
        cursor_pos,
    )
}

/// A Python module implemented in Rust.
#[pymodule]
fn jieba_navi_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wordmotion_b, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_B, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_ge, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_gE, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_w, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_W, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_e, m)?)?;
    m.add_function(wrap_pyfunction!(wordmotion_E, m)?)?;

    Ok(())
}

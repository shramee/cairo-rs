#![deny(warnings)]
use std::path::Path;

use cairo_rs::cairo_run;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use pyo3::prelude::*;

#[pyfunction]
fn execute_cairo(
    filename: String,
    trace_file: String,
    print_output: bool,
    entrypoint: String,
    memory_file: String,
) -> PyResult<()> {
    let hint_processor = BuiltinHintProcessor::new_empty();

    let trace_enabled = true;
    let mut cairo_runner = match cairo_run::cairo_run(
        &Path::new(&filename),
        &entrypoint,
        trace_enabled,
        &hint_processor,
    ) {
        Ok(runner) => runner,
        Err(_error) => todo!(),
    };

    let relocated_trace = cairo_runner.relocated_trace.as_ref().unwrap();
    match cairo_run::write_binary_trace(relocated_trace, Path::new(&trace_file)) {
        Ok(()) => (),
        Err(_e) => todo!(),
    }

    if print_output {
        cairo_run::write_output(&mut cairo_runner).unwrap();
    }

    match cairo_run::write_binary_memory(&cairo_runner.relocated_memory, &Path::new(&memory_file)) {
        Ok(()) => (),
        Err(_e) => todo!(),
    }

    Ok(())
}

#[pymodule]
fn python_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(execute_cairo, m)?)?;
    Ok(())
}

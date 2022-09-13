#![deny(warnings)]
use std::path::PathBuf;

use cairo_rs::cairo_run;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "cairo_run")]
fn py_cairo_run(
    filename: PathBuf,
    print_output: bool,
    entrypoint: Option<String>,
    trace_file: Option<PathBuf>,
    memory_file: Option<PathBuf>,
) -> PyResult<()> {
    let hint_processor = BuiltinHintProcessor::new_empty();

    let trace_enabled = trace_file.is_some();

    let entrypoint = entrypoint.unwrap_or(String::from("main"));

    let mut cairo_runner =
        match cairo_run::cairo_run(&filename, &entrypoint, trace_enabled, &hint_processor) {
            Ok(runner) => runner,
            Err(_error) => todo!(),
        };

    if let Some(trace_path) = trace_file {
        let relocated_trace = cairo_runner.relocated_trace.as_ref().unwrap();
        match cairo_run::write_binary_trace(relocated_trace, &trace_path) {
            Ok(()) => (),
            Err(_e) => todo!(),
        }
    }

    if print_output {
        cairo_run::write_output(&mut cairo_runner).unwrap();
    }

    if let Some(memory_path) = memory_file {
        match cairo_run::write_binary_memory(&cairo_runner.relocated_memory, &memory_path) {
            Ok(()) => (),
            Err(_e) => todo!(),
        }
    }

    Ok(())
}

#[pymodule]
fn cairo_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_cairo_run, m)?)?;
    Ok(())
}

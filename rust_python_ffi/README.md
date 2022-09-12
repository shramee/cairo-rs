# Building

Go to the `vm_poc` folder:
`cd vm_poc`

Create virtual environment and activate it:
`python -m venv .env`
`source .env/bin/activate`

Install maturin
`pip install maturin`

Build weels for CPython
`maturin develop`

Go to `python_executor` folder
`cd ../python_executor`

Set the dynamic library path
`export DYLD_LIBRARY_PATH="/Users/lambda/cairo-rs/rust_python_ffi/vm_poc/.env/lib"`

Run the project
`cargo r`


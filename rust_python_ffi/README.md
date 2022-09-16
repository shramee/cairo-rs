# Building
Create a Python virtual environment and activate it

```
cd ffi
python -m venv .env
source .env/bin/activate
```

Install maturin
```
pip install maturin
```

Set the dynamic library path
```
export DYLD_LIBRARY_PATH=".env/lib"
```

# Running the Python server
```
cd ../python_server
make run
```


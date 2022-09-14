.PHONY: default build run

default: 
	run 
	
build:
	cd ../ffi && maturin develop

run: build
	python3 src/main.py

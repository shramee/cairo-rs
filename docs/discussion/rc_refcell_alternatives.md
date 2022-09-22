# Rc<RefCell> alternatives discussion

## Memory cloning alternative: Benchmark results
Benchmarks were done with the Rust-Python integration PoC.

```
# PoC with Rc-RefCell

           1mill elements     10mill elements     100mill elements
            
1 hints    0.486s             0.702s              3.028s
2 hints    0.926s             1.142s              3.309s
3 hints    1.384s             1.675s              3.972s
```

```
# PoC with memory cloning

           1mill elements     10mill element      100mill elements 
            
1 hints    1.491s             2.052s              18.665s 
2 hints    3.217s             4.318s              37.053s 
3 hints    5.114s             6.498s              58.469s
```
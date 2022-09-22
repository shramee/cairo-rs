import ffi

def fibonacci(n):
    a = 1
    b = 1
    while (n-2):
        c = a + b
        a, b = b, c
        n = n-1
    return c

def main():
    vm = ffi.PyVM()
    vm.load("11112")
    vm.run()

if __name__ == '__main__':
    main()


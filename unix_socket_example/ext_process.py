import socket
import os
import sys

def print_memory(byte_array):
    for i, byte in enumerate(byte_array):
        if i % 32 == 0 and i != 0:
            idx_0 = i - 32
            print("Number", int.from_bytes(byte_array[idx_0:i], "little"))



socket_path = "ipc.sock"

try:
    os.unlink(socket_path)
except OSError:
    pass

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.bind(socket_path)

s.listen()

while 1:
    conn, addr = s.accept()
    b_array = bytearray()
    try:
        while 1:
            data = conn.recv(8192)
            if data:
                #print("received ", data)
                #print("Receive number: ", j)
                #j += 1
                b_array += bytearray(data)
                #print_memory(b_array)
    finally:
        conn.close()
        print_memory(b_array)

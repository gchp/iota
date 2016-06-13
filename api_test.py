import socket
import json


s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(("localhost", 10000))

open_file = json.dumps({
    "command": "create_buffer",
    "args": {
        "path": "/home/gchp/src/github.com/gchp/iota/README.md"
    }    
})

print(len(open_file))

print("Sending open_file")
s.send(open_file)
print("Receiving open_file")
resp = s.recv(2048)

print(resp)

open_file = json.dumps({
    "command": "list_buffers",
    "args": {}    
})
print(len(open_file))
print("Sending list_buffers")
s.send(open_file)
print("Receiving list_buffers")
resp = s.recv(2048)

print(resp)

#s.close()

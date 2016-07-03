from argparse import ArgumentParser

import socket
import json


def get_socket():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect(("localhost", 10000))
    return s

def list_workspaces(s, *args, **kwargs):
    command = json.dumps({
        "command": "list_workspaces",
        "args": {}
    })
    s.send(command)


def create_buffer(s, *args, **kwargs):
    command = json.dumps({
        "command": "create_buffer",
        "args": {
            "path": "/home/gchp/src/github.com/gchp/iota/README.md"
        }
    })
    s.send(command)


def main():
    parser = ArgumentParser()
    parser.add_argument("api_command")
    parser.add_argument("create_buffer", action="store_true")

    args = parser.parse_args()

    print(args.api_command)

    s = get_socket()

    method = globals()[args.api_command]

    method(s)
    response = s.recv(2048)
    print(response)
    s.close()


if __name__ == "__main__":
    main()

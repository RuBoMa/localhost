import os
import sys

def read_request_body():
    length = os.environ.get("CONTENT_LENGTH")
    if not length:
        # "b" for binary (converts "" into bytes, not an empty string)
        return b""
    try:
        to_read = int(length)
    except ValueError:
        to_read = 0
    return sys.stdin.buffer.read(to_read)

def main():
    script_name = os.environ.get("SCRIPT_NAME", "")
    server_name = os.environ.get("SERVER_NAME", "")
    server_port = os.environ.get("SERVER_PORT", "")
    method = os.environ.get("REQUEST_METHOD", "")
    query = os.environ.get("QUERY_STRING", "")
    body = read_request_body()

    sys.stdout.write("Content-Type: text/plain\r\n")
    sys.stdout.write(
        f"Hello from hello.py using CGI!\r\n\r\n"
        f"Some request info:\r\n"
        f"SERVER_NAME={server_name}\nSERVER_PORT={server_port}\nMETHOD={method}\nSCRIPT_NAME={script_name}\nQUERY={query}\n"
        f"BODY_LEN={len(body)}\n"
    )

if __name__ == "__main__":
    main()
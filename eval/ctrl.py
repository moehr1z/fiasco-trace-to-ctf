#!/usr/bin/env python3

import socket
import subprocess

HOST = "0.0.0.0"
PORT = 9999
CONVERTER_CMD = ["./run.sh", "-l error"]  # NOTE add -r here if running real benchmark


def start_converter():
    try:
        print("Starting converter...")
        subprocess.run(CONVERTER_CMD, check=True)
        print("Converter finished.")
    except subprocess.CalledProcessError as e:
        print(f"Converter failed with return code {e.returncode}")
    except Exception as e:
        print(f"Unexpected error: {e}")


def main():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((HOST, PORT))
        s.listen(1)
        print(f"Listening on {HOST}:{PORT}...")

        while True:
            conn, addr = s.accept()
            with conn:
                print(f"Connection from {addr}")
                data = conn.recv(1024)
                print(f"Received raw data: {data}")
                if data and data[0] == 1:
                    start_converter()
                elif data and data[0] == 2:
                    print("Signaled to stop")
                    exit()
                else:
                    print("Unexpected or malformed message received.")


if __name__ == "__main__":
    main()

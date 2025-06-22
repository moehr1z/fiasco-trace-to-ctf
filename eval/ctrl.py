#!/usr/bin/env -S python3 -u

import socket
import subprocess
import time
import psutil
import struct

HOST = "0.0.0.0"
PORT = 9999
CONVERTER_CMD = [
    "./run.sh",
    "-l error",
    "-r",
]  # NOTE add -r here if running real benchmark


def start_converter():
    proc = subprocess.Popen(CONVERTER_CMD)
    p = psutil.Process(proc.pid)

    try:
        while True:
            if not p.is_running() or p.status() == psutil.STATUS_ZOMBIE:
                print("Exit")
                break

            rss = p.memory_info().rss  # resident memory

            print(f"RSS={rss} BYTES")

            time.sleep(0.5)
    except psutil.NoSuchProcess:
        print("Exit")
    finally:
        cpu_times = p.cpu_times()
        total_cpu_time = cpu_times.user + cpu_times.system
        wall_time = time.time() - p.create_time()

        num_cores = 1
        avg_cpu_percent = (total_cpu_time / wall_time) * 100 / num_cores

        print(f"AVG CPU: {avg_cpu_percent:.2f}%")

        return_code = proc.wait()

        print(f"Process exited with code {return_code}")


def main():
    run = 0
    rate = 0

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((HOST, PORT))
        s.listen(1)
        print(f"Listening on {HOST}:{PORT}...")

        while True:
            conn, addr = s.accept()
            print(f"Connection from {addr}")
            with conn:
                data = conn.recv(100)
                if data:
                    print(f"Received raw data: {data}")
                    opcode = data[0]
                    print(f"Opcode: {opcode}")
                    if opcode == 1:
                        print(f"RUN {run}")
                        print(f"RATE {rate}")
                        start_converter()
                        print(f"END {run}")
                    elif opcode == 2:
                        print("Signaled to stop")
                        exit()
                    elif opcode == 3:
                        rate = struct.unpack("I", data[1:5])[0]
                    elif opcode == 4:
                        run = struct.unpack("I", data[1:5])[0]
                    else:
                        print("Unexpected or malformed message received.")
                else:
                    print("Client disconnected.")


if __name__ == "__main__":
    main()

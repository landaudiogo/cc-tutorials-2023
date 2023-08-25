import time
import signal
import sys

def signal_handler(sig, frame):
    print('EXITING SAFELY!')
    exit(0)

signal.signal(signal.SIGTERM, signal_handler)

print("Starting")
print(sys.argv)
i = 0
while True:
    print(i)
    time.sleep(1)
    i += 1

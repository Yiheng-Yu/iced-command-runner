import time
import sys

print("Streaming stdout")
for i in range(5):
    print(f"This is {i}th data into stdout", file=sys.stdout, flush=True)
    print(f"This is {i}th data into stderr", file=sys.stderr, flush=True)
    time.sleep(0.5)
    
print("Raising error")
print(not_exist)
from tqdm import tqdm
from time import sleep

for i in tqdm(range(10), desc="Dummy progress"):
    sleep(0.5)
import multiprocessing
import string
import shutil
from pathlib import Path
import subprocess


alpha = {
    0: 'a',
    1: 'b',
    2: 'c',
    3: 'd',
    4: 'e',
    5: 'f',
    6: 'g',
    7: 'h',
    8: 'i',
    9: 'j',
    10: 'k',
    11: 'l',
    12: 'm',
    13: 'n',
    14: 'o',
    15: 'p',
    16: 'q',
    17: 'r',
    18: 's',
    19: 't',
    20: 'u',
    21: 'v',
    22: 'w',
    23: 'x',
    24: 'y',
    25: 'z',
}
    




def clean_data():
    shutil.rmtree("data")
    
def creat_files():
    d = Path('data')
    d.mkdir(exist_ok=True)
    for c in string.ascii_lowercase:
        with open(f'data/{c}','w') as f:
            f.write(c)

    
def upload_to_minio(x):
    def dfs(n,x,tmp,ans):
        if n == 4:
            ans.append("/".join(tmp))
            return
        for i in range(x):
            tmp.append(alpha[i])
            dfs(n+1,x,tmp,ans)
            tmp.pop()
    targets = []
    dfs(0,x,[],targets)
    return targets

class DataProcessor(multiprocessing.Process):
    def __init__(self, chunk):
        super().__init__()
        self.chunk = chunk
    
    def run(self):
        for t in self.chunk:
            subprocess.run(["mcli","cp",f"data/{t[-1]}",f"myio/jiax/{t}"],check=True,stdout=subprocess.DEVNULL)


def main():
    clean_data()
    creat_files()
    targets = upload_to_minio(5)
    num_processes = multiprocessing.cpu_count()
    chunk_size = len(targets) // num_processes
    chunks = [targets[i*chunk_size : (i+1)*chunk_size] for i in range(num_processes)]
    
    processes = [DataProcessor(chunk) for chunk in chunks]
    for p in processes:
        p.start()
    for p in processes:
        p.join()
    print("finish")

    
if __name__== "__main__":
    main() 




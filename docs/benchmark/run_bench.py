from contextlib import suppress
import os
import subprocess

import matplotlib

matplotlib.use("Agg")
from matplotlib import pyplot as plt
import numpy as np


def run_bench(vim):
    with suppress(FileNotFoundError):
        os.remove("bench_output_std.txt")
    with suppress(FileNotFoundError):
        os.remove("bench_output_custom.txt")
    if vim == "vim":
        cmd = f"{vim} -es -u vimrc -S bench.vim data_zh.txt"
    else:
        cmd = f"{vim} --headless -u init.lua -S bench.vim data_zh.txt"
    subprocess.run(cmd.split(), check=True)
    return np.loadtxt("bench_output_custom.txt")


subprocess.run("bash download_dataset.sh".split(), check=True)
vim_bench_zh = run_bench("vim")
nvim_bench_zh = run_bench("nvim")

fig, ax = plt.subplots()
ax.hist(nvim_bench_zh[1:], bins=50, density=True, label="nvim", alpha=0.7)
ax.hist(vim_bench_zh[1:], bins=50, density=True, label="vim", alpha=0.7)
ax.set_title("warm start wall time histogram")
ax.legend()
ax.set_xlabel("sec")
fig.savefig("bench_output_hist.jpg")
plt.close(fig)

fig, ax = plt.subplots()
ax.bar(["vim", "nvim"], [vim_bench_zh[0], nvim_bench_zh[0]], width=0.3)
ax.set_title("warmup wall time")
ax.set_ylabel("sec")
fig.savefig("bench_output_warmup.jpg")
plt.close(fig)

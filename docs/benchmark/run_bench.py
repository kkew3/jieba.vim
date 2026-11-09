import argparse
import json
import os
import subprocess
import time
from contextlib import suppress
from pathlib import Path

VERSION = 1
BENCH_FILE = "bench_result.json"
BENCH_REPORT = "bench_result.md"


def download_dataset():
    subprocess.run(["bash", "download_dataset.sh"], check=True)


def run_build(vim):
    plugin_root_dir = Path(__file__).resolve().parents[2]
    os.environ["JIEBA_VIM_BUILD_FROM_SOURCE"] = "1"
    os.environ["JIEBA_VIM_INSTALL_NVIM"] = "0" if vim == "vim" else "1"
    subprocess.run(["./build.sh"], cwd=plugin_root_dir, check=True)
    time.sleep(10)


def run_bench(vim, repeat_warmup):
    import numpy as np

    if vim == "vim":
        cmd = f"{vim} -es -u vimrc -S bench.vim data_zh.txt"
    else:
        cmd = f"{vim} --headless -u vimrc -S bench.vim data_zh.txt"

    # Run builtin motion.
    with suppress(FileNotFoundError):
        os.remove("bench_output_std.txt")
    os.environ["RUN_BUILTIN"] = "1"
    subprocess.run(cmd.split(), check=True)
    data = np.loadtxt("bench_output_std.txt")
    stdrun_mean = np.mean(data[1:])
    stdrun_std = np.std(data[1:])
    stdrun_q90 = np.quantile(data[1:], 0.9)

    # Run jieba motion (warmup).
    del os.environ["RUN_BUILTIN"]
    os.environ["MEASURE_WARMUP"] = "1"
    warmup = []
    for _ in range(repeat_warmup + 1):
        time.sleep(2)
        with suppress(FileNotFoundError):
            os.remove("bench_output_custom.txt")
        subprocess.run(cmd.split(), check=True)
        data = np.loadtxt("bench_output_custom.txt")
        warmup.append(data[0])
    del warmup[0]  # the first run loads cdylib into system cache
    warmup = np.stack(warmup)

    # Run jieba motion.
    del os.environ["MEASURE_WARMUP"]
    time.sleep(2)
    subprocess.run(cmd.split(), check=True)
    data = np.loadtxt("bench_output_custom.txt")
    warmed = data[1:]

    vim_version = (
        subprocess.run(
            [vim, "--version"], check=True, capture_output=True, text=True
        )
        .stdout.splitlines()[0]
        .strip()
    )

    return {
        "vim_version": vim_version,
        "builtin": {
            "mean": float(stdrun_mean),
            "std": float(stdrun_std),
            "q90": float(stdrun_q90),
        },
        "jieba": {
            "warmup": {
                "mean": float(np.mean(warmup)),
                "std": float(np.std(warmup)),
                "q90": float(np.quantile(warmup, 0.9)),
            },
            "warmed": {
                "mean": float(np.mean(warmed)),
                "std": float(np.std(warmed)),
                "q90": float(np.quantile(warmed, 0.9)),
            },
        },
    }


def builtin_adjusted_change_warmed(base, head, field):
    return (head["jieba"]["warmed"][field] / base["jieba"]["warmed"][field]) / (
        head["builtin"][field] / base["builtin"][field]
    ) - 1


def change_builtin(base, head, field):
    return head["builtin"][field] / base["builtin"][field] - 1


def change_warmup(base, head, field):
    return head["jieba"]["warmup"][field] / base["jieba"]["warmup"][field] - 1


def fmt_num(x):
    return f"{x:.2e}"


def main():
    parser = argparse.ArgumentParser(
        description="Run benchmark or evaluate benchmark against a past commit."
    )
    parser.add_argument(
        "--base",
        help="without this, run benchmark; otherwise, compare HEAD against BASE",
    )
    parser.add_argument(
        "--report",
        action="store_true",
        help=(
            "when `--base` is not specified, "
            "generate markdown report only without running benchmark"
        ),
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help=(
            "when `--base` is not specified, "
            f"write the report to ./{BENCH_REPORT} rather than printing to "
            "stdout"
        ),
    )
    args = parser.parse_args()

    if not args.base:
        if args.report:
            with open(
                Path(__file__).parent / BENCH_FILE, encoding="utf-8"
            ) as infile:
                head = json.load(infile)
        else:
            download_dataset()

            run_build("vim")
            vim_bench_zh = run_bench("vim", repeat_warmup=20)

            run_build("nvim")
            nvim_bench_zh = run_bench("nvim", repeat_warmup=20)

            head = {
                "version": VERSION,
                "vim": vim_bench_zh,
                "nvim": nvim_bench_zh,
            }

            with open(
                Path(__file__).parent / BENCH_FILE, "w", encoding="utf-8"
            ) as f:
                json.dump(head, f, indent=2)
                f.write("\n")

        ctx = {}
        for vim in ["vim", "nvim"]:
            ctx[f"{vim}_version"] = head[vim]["vim_version"]
            for field in ["mean", "std", "q90"]:
                ctx[f"{vim}_head_{field}"] = fmt_num(
                    head[vim]["builtin"][field] * 1000
                )
                ctx[f"{vim}_jieba_head_{field}"] = fmt_num(
                    head[vim]["jieba"]["warmed"][field] * 1000
                )
                ctx[f"{vim}_warmup_head_{field}"] = fmt_num(
                    head[vim]["jieba"]["warmup"][field]
                )

        report = """\
# Vim

> version: `{vim_version}`

## Builtin vs. Jieba (warmed)

| Metric | Builtin | Jieba (warmed) |
|--------|---------|----------------|
| Mean (ms) | {vim_head_mean} | {vim_jieba_head_mean} |
| Std (ms) | {vim_head_std} | {vim_jieba_head_std} |
| Q90 (ms) | {vim_head_q90} | {vim_jieba_head_q90} |

## Jieba (warmup)

| Metric | Jieba (warmup) |
|--------|----------------|
| Mean (s) | {vim_warmup_head_mean} |
| Std (s) | {vim_warmup_head_std} |
| Q90 (s) | {vim_warmup_head_q90} |

# Neovim

> version: `{nvim_version}`

## Builtin vs. Jieba (warmed)

| Metric | Builtin | Jieba (warmed) |
|--------|---------|----------------|
| Mean (ms) | {nvim_head_mean} | {nvim_jieba_head_mean} |
| Std (ms) | {nvim_head_std} | {nvim_jieba_head_std} |
| Q90 (ms) | N/A | {nvim_jieba_head_q90} |

## Jieba (warmup)

| Metric | Jieba (warmup) |
|--------|----------------|
| Mean (s) | {nvim_warmup_head_mean} |
| Std (s) | {nvim_warmup_head_std} |
| Q90 (s) | {nvim_warmup_head_q90} |
""".format(**ctx)

        if args.write:
            with open(
                Path(__file__).parent / BENCH_REPORT, "w", encoding="utf-8"
            ) as f:
                f.write(report)
        else:
            print(report, end="")

        return

    with open(Path(__file__).parent / BENCH_FILE, encoding="utf-8") as infile:
        head = json.load(infile)
    if head["version"] != VERSION:
        raise ValueError(f"HEAD version != {VERSION}")
    base = json.loads(
        subprocess.run(
            ["git", "show", f"{args.base}:docs/benchmark/{BENCH_FILE}"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    if base["version"] != VERSION:
        raise ValueError(f"BASE version != {VERSION}")
    ctx = {}
    for vim in ["vim", "nvim"]:
        for dname, dict_ in zip(["base", "head"], [base, head]):
            ctx[f"{vim}_{dname}_version"] = dict_[vim]["vim_version"]
        for field in ["mean", "std", "q90"]:
            for dname, dict_ in zip(["base", "head"], [base, head]):
                ctx[f"{vim}_{dname}_{field}"] = fmt_num(
                    dict_[vim]["builtin"][field] * 1000
                )
                ctx[f"{vim}_jieba_{dname}_{field}"] = fmt_num(
                    dict_[vim]["jieba"]["warmed"][field] * 1000
                )
                ctx[f"{vim}_warmup_{dname}_{field}"] = fmt_num(
                    dict_[vim]["jieba"]["warmup"][field]
                )
            ctx[f"{vim}_change_{field}"] = fmt_num(
                change_builtin(base[vim], head[vim], field) * 100
            )
            ctx[f"{vim}_jieba_change_{field}"] = fmt_num(
                builtin_adjusted_change_warmed(base[vim], head[vim], field)
                * 100
            )
            ctx[f"{vim}_warmup_change_{field}"] = fmt_num(
                change_warmup(base[vim], head[vim], field) * 100
            )

    print(
        """\
# Vim

> BASE version: `{vim_base_version}`

> HEAD version: `{vim_head_version}`

## Environment control

| Metric | BASE | HEAD | Change Δ(%) |
|--------|------|------|-------------|
| Builtin mean (ms) | {vim_base_mean} | {vim_head_mean} | {vim_change_mean} |
| Builtin std (ms) | {vim_base_std} | {vim_head_std} | {vim_change_std} |
| Builtin Q90 (ms) | {vim_base_q90} | {vim_head_q90} | {vim_change_q90} |

## Jieba (warmed)

| Metric | BASE | HEAD | Adjusted-change Δ(%) |
|--------|------|------|----------------------|
| Mean (ms) | {vim_jieba_base_mean} | {vim_jieba_head_mean} | {vim_jieba_change_mean} |
| Std (ms) | {vim_jieba_base_std} | {vim_jieba_head_std} | {vim_jieba_change_std} |
| Q90 (ms) | {vim_jieba_base_q90} | {vim_jieba_head_q90} | {vim_jieba_change_q90} |

## Jieba (warmup)

| Metric | BASE | HEAD | Change Δ(%) |
|--------|------|------|-------------|
| Mean (s) | {vim_warmup_base_mean} | {vim_warmup_head_mean} | {vim_warmup_change_mean} |
| Std (s) | {vim_warmup_base_std} | {vim_warmup_head_std} | {vim_warmup_change_std} |
| Q90 (s) | {vim_warmup_base_q90} | {vim_warmup_head_q90} | {vim_warmup_change_q90} |

# Neovim

> BASE version: `{nvim_base_version}`

> HEAD version: `{nvim_head_version}`

## Environment control

| Metric | BASE | HEAD | Change Δ(%) |
|--------|------|------|-------------|
| Builtin mean (ms) | {nvim_base_mean} | {nvim_head_mean} | {nvim_change_mean} |
| Builtin std (ms) | {nvim_base_std} | {nvim_head_std} | {nvim_change_std} |

## Jieba (warmed)

| Metric | BASE | HEAD | Adjusted-change Δ(%) |
|--------|------|------|----------------------|
| Mean (ms) | {nvim_jieba_base_mean} | {nvim_jieba_head_mean} | {nvim_jieba_change_mean} |
| Std (ms) | {nvim_jieba_base_std} | {nvim_jieba_head_std} | {nvim_jieba_change_std} |
| Q90 (ms) | {nvim_jieba_base_q90} | {nvim_jieba_head_q90} | {nvim_jieba_change_q90} |

## Jieba (warmup)

| Metric | BASE | HEAD | Change Δ(%) |
|--------|------|------|-------------|
| Mean (s) | {nvim_warmup_base_mean} | {nvim_warmup_head_mean} | {nvim_warmup_change_mean} |
| Std (s) | {nvim_warmup_base_std} | {nvim_warmup_head_std} | {nvim_warmup_change_std} |
| Q90 (s) | {nvim_warmup_base_q90} | {nvim_warmup_head_q90} | {nvim_warmup_change_q90} |
""".format(**ctx),
        end="",
    )


if __name__ == "__main__":
    main()

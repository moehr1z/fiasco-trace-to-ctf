#!/usr/bin/env python3

import re
import argparse
from collections import OrderedDict
import matplotlib.pyplot as plt
import pandas as pd
import matplotlib.ticker as ticker


def human_format(x, pos):
    if x >= 1e9:
        return f"{x * 1.0 / 1e9:.1f}G"
    elif x >= 1e6:
        return f"{x * 1.0 / 1e6:.1f}M"
    elif x >= 1e3:
        return f"{x * 1.0 / 1e3:.1f}K"
    else:
        return f"{x:.0f}"


def parse_benchmark_file(path, x_axis):
    run_map_rate = OrderedDict()
    run_map_cpus = OrderedDict()
    current = None
    ansi_escape = re.compile(r"\x1b\[[0-9;]*m")
    re_run_start = re.compile(r"^[^\w]*RUN\s+(\d+)")
    re_rate = re.compile(r"^[^\w]*RATE\s+(\d+)")
    re_rss = re.compile(r"^[^\w]*RSS=(\d+)\s+BYTES")
    re_throughput = re.compile(r"^[^\w]*THROUGHPUT:\s+([\d.]+)\s+\(EVENTS/SEC\)")
    re_rcv_throughput = re.compile(r"^[^\w]*RECEIVE THROUHGPUT:\s+([\d.]+)")
    re_dropped = re.compile(r"^[^\w]*EVENTS DROPPED:\s+(\d+)")
    re_n_cpus = re.compile(r"^[^\w]*NR CPUS:\s+(\d+)")
    re_avg_cpu = re.compile(r"^[^\w]*AVG CPU:\s*([\d.]+)%")
    re_run_end = re.compile(r"^[^\w]*END\s+(\d+)")
    re_events_total = re.compile(r"^[^\w]*EVENTS TOTAL:\s+(\d+)")

    with open(path, "r") as f:
        for line in f:
            line = ansi_escape.sub("", line.strip())
            m = re_run_start.match(line)
            if m:
                if current:
                    current["max_rss"] = (
                        max(current["all_rsses"]) if current["all_rsses"] else None
                    )
                    run_map_rate[(current["rate"], current["run_id"])] = current
                    run_map_cpus[(current["n_cpus"], current["run_id"])] = current
                run_id = int(m.group(1))
                current = {
                    "run_id": run_id,
                    "rate": None,
                    "all_rsses": [],
                    "avg_cpu": None,
                    "max_rss": None,
                    "dropped_events": None,
                    "throughput": None,
                    "rcv_throughput": None,
                    "n_cpus": None,
                    "events_total": None,
                }
                continue
            if not current:
                continue
            if re_rate.match(line):
                current["rate"] = int(re_rate.match(line).group(1))
                continue
            if re_rss.match(line):
                current["all_rsses"].append(int(re_rss.match(line).group(1)))
                continue
            if re_throughput.match(line):
                current["throughput"] = float(re_throughput.match(line).group(1))
                continue
            if re_rcv_throughput.match(line):
                current["rcv_throughput"] = float(
                    re_rcv_throughput.match(line).group(1)
                )
                continue
            if re_dropped.match(line):
                current["dropped_events"] = int(re_dropped.match(line).group(1))
                continue
            if re_n_cpus.match(line):
                current["n_cpus"] = int(re_n_cpus.match(line).group(1))
                continue
            if re_avg_cpu.match(line):
                current["avg_cpu"] = float(re_avg_cpu.match(line).group(1))
                continue
            if re_events_total.match(line):
                current["events_total"] = int(re_events_total.match(line).group(1))
                continue
            if re_run_end.match(line):
                eid = int(re_run_end.match(line).group(1))
                if current["run_id"] != eid:
                    print(f"Warning: mismatched END {eid} for run {current['run_id']}")
                current["max_rss"] = (
                    max(current["all_rsses"]) if current["all_rsses"] else None
                )
                run_map_rate[(current["rate"], current["run_id"])] = current
                run_map_cpus[(current["n_cpus"], current["run_id"])] = current
                current = None
    if current:
        current["max_rss"] = max(current["all_rsses"]) if current["all_rsses"] else None
        run_map_rate[(current["rate"], current["run_id"])] = current
        run_map_cpus[(current["n_cpus"], current["run_id"])] = current

    if x_axis == "cpus":
        return list(run_map_cpus.values())
    else:
        return list(run_map_rate.values())


def build_dataframe(runs):
    rows = []
    for r in runs:
        rows.append(
            {
                "run_id": r["run_id"],
                "rate": r["rate"],
                "avg_cpu": r["avg_cpu"],
                "max_rss": r["max_rss"],
                "dropped_events": r["dropped_events"],
                "throughput": r["throughput"],
                "rcv_throughput": r["rcv_throughput"],
                "n_cpus": r["n_cpus"],
                "events_total": r["events_total"],
            }
        )
    return pd.DataFrame(rows)


def plot_boxplot(
    df, group_col, metric_col, xlabel, ylabel, title, output_filename=None
):
    values = sorted(df[group_col].dropna().unique())
    data = [df[df[group_col] == v][metric_col].dropna().values for v in values]
    plt.figure(figsize=(8, 6))
    plt.boxplot(
        data,
        labels=[str(v) for v in values],
        patch_artist=True,
        boxprops=dict(facecolor="lightblue", linewidth=1),
    )
    plt.xlabel(xlabel)
    plt.ylabel(ylabel)
    plt.title(title)
    y_min = min([min(d) for d in data if len(d) > 0] + [0])
    y_max = max([max(d) for d in data if len(d) > 0] + [1])

    plt.grid(axis="y", linestyle="--", alpha=0.6)
    plt.gca().yaxis.set_major_formatter(ticker.FuncFormatter(human_format))
    if y_max == 1 and y_min == 0:
        plt.gca().yaxis.set_major_locator(ticker.FixedLocator([0, 1, 2]))
        plt.ylim(bottom=-0.1, top=2.0)
    else:
        plt.ylim(
            bottom=y_min
            - 0.05 * (max([max(d) for d in data if len(d) > 0] + [1]) - y_min),
            top=y_max + 0.05 * y_max,
        )

    if output_filename:
        plt.savefig(output_filename, bbox_inches="tight")
        print(f"Saved boxplot: {output_filename}")
    else:
        plt.show()


def main():
    parser = argparse.ArgumentParser(
        description="Parse benchmark output and build boxplots grouped by event rate or CPU count."
    )
    parser.add_argument(
        "-i", "--input", required=True, help="Benchmark output file path"
    )
    parser.add_argument(
        "-o", "--outdir", default=".", help="Output directory for plots"
    )
    parser.add_argument(
        "--x-axis",
        choices=["rate", "cpus"],
        default="rate",
        help="Group plots by 'rate' or 'cpus'",
    )
    args = parser.parse_args()

    runs = parse_benchmark_file(args.input, args.x_axis)
    if not runs:
        print("No valid runs found. Exiting.")
        return
    df = build_dataframe(runs)
    df["max_rss_mb"] = df["max_rss"] / (1024 * 1024)
    df["rcv_throughput_mb"] = df["rcv_throughput"] / (1024 * 1024)

    if args.x_axis == "rate":
        grp, xlabel, suf = "rate", "Event-Rate (Events/Sek.)", "by_rate"
        const_vals = df["n_cpus"].dropna().unique()
        const_label = "CPUs"
    else:
        grp, xlabel, suf = "n_cpus", "Anzahl CPUs", "by_cpus"
        const_vals = df["rate"].dropna().unique()
        const_label = "Event-Rate"

    if len(const_vals) == 1:
        const_str = str(const_vals[0])
    else:
        const_str = ",".join(str(v) for v in sorted(const_vals))

    metrics = [
        ("avg_cpu", "Durchschnittliche CPU-Auslastung (%)", "cpu_usage"),
        ("max_rss_mb", "Maximale RAM-Auslastung (MiB)", "ram_usage"),
        ("dropped_events", "Verlorene Events", "dropped_events"),
        ("throughput", "Konvertierungsdurchsatz (Events/Sek.)", "throughput"),
        ("rcv_throughput_mb", "Empfangsdurchsatz (MiB/Sek.)", "rcv_throughput"),
        ("events_total", "Anzahl Events", "events_total"),
    ]

    for metric, ylabel, prefix in metrics:
        if df[metric].dropna().empty:
            print(f"Skipping {metric}: no data")
            continue
        title = f"{ylabel} ({const_label}={const_str})"
        filename = f"{args.outdir}/{prefix}_{suf}.pdf"
        plot_boxplot(df, grp, metric, xlabel, ylabel, title, filename)

    print("All requested plots have been generated.")


if __name__ == "__main__":
    main()

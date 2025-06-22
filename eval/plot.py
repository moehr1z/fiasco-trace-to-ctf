#!/usr/bin/env python3

import re
import argparse
from collections import OrderedDict
import matplotlib.pyplot as plt
import pandas as pd


def parse_benchmark_file(path):
    # We’ll use an ordered dict mapping (rate, run_id) -> run_dict.
    # When a duplicate key appears, we simply overwrite with the newer block.
    run_map = OrderedDict()
    current = None

    # Regex patterns
    re_run_start = re.compile(r"^RUN\s+(\d+)")
    re_rate = re.compile(r"^RATE\s+(\d+)")
    re_rss = re.compile(r"^RSS=(\d+)\s+BYTES")
    re_throughput = re.compile(r"^THROUGHPUT:\s+([\d.]+)\s+\(EVENTS/SEC\)")
    re_rcv_throughput = re.compile(r"^RECEIVE THROUHGPUT:\s+([\d.]+)")
    re_dropped = re.compile(r"^EVENTS DROPPED:\s+(\d+)")
    re_n_cpus = re.compile(r"^NR CPUS:\s+(\d+)")
    re_avg_cpu = re.compile(r"^AVG CPU:\s*([\d.]+)%")
    re_run_end = re.compile(r"^END\s+(\d+)")

    with open(path, "r") as f:
        for line in f:
            line = line.strip()

            # Detect start of a new RUN
            m = re_run_start.match(line)
            if m:
                run_id = int(m.group(1))
                # If a previous run was still "open" (no END line) we finalize it now
                if current is not None:
                    # finalize previous run
                    if current["all_rsses"]:
                        current["max_rss"] = max(current["all_rsses"])
                    else:
                        current["max_rss"] = None

                    key = (current["rate"], current["run_id"])
                    run_map[key] = current

                # Begin a fresh run dict
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
                }
                continue

            # If we haven’t started any run yet, skip until first RUN
            if current is None:
                continue

            # RATE
            m = re_rate.match(line)
            if m:
                current["rate"] = int(m.group(1))
                continue

            # RSS
            m = re_rss.match(line)
            if m:
                rss_val = int(m.group(1))
                current["all_rsses"].append(rss_val)
                continue

            # THROUGHPUT
            m = re_throughput.match(line)
            if m:
                current["throughput"] = float(m.group(1))
                continue

            # EVENTS DROPPED
            m = re_dropped.match(line)
            if m:
                current["dropped_events"] = int(m.group(1))
                continue

            # RECEIVE THROUGHPUT
            m = re_rcv_throughput.match(line)
            if m:
                current["rcv_throughput"] = float(m.group(1))
                continue

            # NR CPUS
            m = re_n_cpus.match(line)
            if m:
                current["n_cpus"] = int(m.group(1))
                continue

            # AVG CPU
            m = re_avg_cpu.match(line)
            if m:
                current["avg_cpu"] = float(m.group(1))
                continue

            # END of run
            m = re_run_end.match(line)
            if m:
                end_id = int(m.group(1))
                if current["run_id"] != end_id:
                    print(
                        f"Warning: mismatched END {end_id} for run {current['run_id']}"
                    )
                # finalize current run
                if current["all_rsses"]:
                    current["max_rss"] = max(current["all_rsses"])
                else:
                    current["max_rss"] = None

                key = (current["rate"], current["run_id"])
                run_map[key] = current
                current = None
                continue

            # All other lines can be safely ignored

    # If file ended without an explicit END for the last run
    if current is not None:
        if current["all_rsses"]:
            current["max_rss"] = max(current["all_rsses"])
        else:
            current["max_rss"] = None
        key = (current["rate"], current["run_id"])
        run_map[key] = current

    # Return only the final dicts in their insertion (chronological) order
    return list(run_map.values())


def build_dataframe(runs):
    """
    Convert a list of run-dicts into a pandas DataFrame.
    """
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
            }
        )
    df = pd.DataFrame(rows)
    return df


def plot_boxplot(df, metric_col, ylabel, title, output_filename=None):
    """
    Create a boxplot of `metric_col` grouped by `rate`.
    - df: DataFrame with at least ['rate', metric_col]
    - ylabel: Y-axis label
    - title: plot title
    - output_filename: if given, save the figure there; otherwise plt.show()
    """
    rates = sorted(df["rate"].dropna().unique())
    data_for_box = [df[df["rate"] == r][metric_col].dropna().values for r in rates]

    plt.figure(figsize=(8, 6))
    plt.boxplot(
        data_for_box,
        labels=[str(r) for r in rates],
        patch_artist=True,
        boxprops=dict(facecolor="lightblue", linewidth=1),
    )
    plt.xlabel("Event Rate (events/sec)")
    plt.ylabel(ylabel)
    plt.title(title)
    plt.grid(axis="y", linestyle="--", alpha=0.6)

    if output_filename:
        plt.savefig(output_filename, bbox_inches="tight")
        print(f"Saved boxplot: {output_filename}")
    else:
        plt.show()


def main():
    parser = argparse.ArgumentParser(
        description="Parse benchmark output (dedup by RATE+RUN) and build boxplots grouped by rate."
    )
    parser.add_argument(
        "--input", "-i", required=True, help="Path to the benchmark output text file"
    )
    parser.add_argument(
        "--outdir",
        "-o",
        default=".",
        help="Directory where the plots will be saved (default: current directory)",
    )
    args = parser.parse_args()

    # Parse and deduplicate
    runs = parse_benchmark_file(args.input)
    if not runs:
        print("No valid runs found in the input file. Exiting.")
        return

    df = build_dataframe(runs)

    base_title = ""

    # 1) CPU usage boxplot
    plot_boxplot(
        df,
        metric_col="avg_cpu",
        ylabel="Average CPU Usage (%)",
        title=base_title + "CPU Usage by Event Rate",
        output_filename=f"{args.outdir}/cpu_usage_boxplot.pdf",
    )

    # 2) RAM usage (convert bytes to MiB)
    df["max_rss_mb"] = df["max_rss"] / (1024 * 1024)
    plot_boxplot(
        df,
        metric_col="max_rss_mb",
        ylabel="Max RAM Usage (MiB)",
        title=base_title + "Max RAM Usage by Event Rate",
        output_filename=f"{args.outdir}/ram_usage_boxplot.pdf",
    )

    # 3) Dropped events
    plot_boxplot(
        df,
        metric_col="dropped_events",
        ylabel="Dropped Events",
        title=base_title + "Dropped Events by Event Rate",
        output_filename=f"{args.outdir}/dropped_events_boxplot.pdf",
    )

    # 4) Throughput
    plot_boxplot(
        df,
        metric_col="throughput",
        ylabel="Throughput (events/sec)",
        title=base_title + "Throughput by Event Rate",
        output_filename=f"{args.outdir}/throughput_boxplot.pdf",
    )

    # 4) Receive Throughput
    df["rcv_throughput_mb"] = df["rcv_throughput"] / (1024 * 1024)
    plot_boxplot(
        df,
        metric_col="rcv_throughput_mb",
        ylabel="Receive Throughput (MB/sec)",
        title=base_title + "Receive Throughput by Event Rate",
        output_filename=f"{args.outdir}/rcv_throughput_boxplot.pdf",
    )

    print("All requested plots have been generated.")


if __name__ == "__main__":
    main()

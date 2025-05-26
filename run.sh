#!/usr/bin/env bash

OUTPUT="/dev/shm/ctf_trace"
PRINT_OUTPUT=""
BIN="./target/debug/l4re_tracestream"
RELEASE=false
LOG="warn"

while getopts "p:l:o:r" opt; do
  case $opt in
  p) PRINT_OUTPUT="$OPTARG" ;;
  l) LOG="$OPTARG" ;;
  o) OUTPUT="$OPTARG" ;;
  r) RELEASE=true ;;
  \?)
    echo "Invalid option: -$OPTARG" >&2
    exit 1
    ;;
  esac
done

if [ "$RELEASE" = true ]; then
  BIN="./target/release/l4re_tracestream"
fi

trash -f "$OUTPUT"
trash -f "$OUTPUT"_[0-9]*

CMD="RUST_LOG='$LOG' '$BIN' -o '$OUTPUT'"
if [ -z "$PRINT_OUTPUT" ]; then
  eval "$CMD"
else
  eval "$CMD" >>"$PRINT_OUTPUT"
fi

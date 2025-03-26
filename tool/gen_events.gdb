# gdb script to generate rust event struct from fiasco kernel

set language c++
set pagination off
python
import os
sys.path.insert(0, os.environ['L4RE_TRACEPARSE_TOOL_DIR'])
import gen_events
end

fiasco-gen-ktrace-events 
quit


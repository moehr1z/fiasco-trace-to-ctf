import os
import gdb
import re

INDENT_SIZE = 4


class Log_table(gdb.Command):
    def __init__(self, imgs):
        for f in imgs:
            self.query_one(f)

    db = {}

    def query_one(self, file):
        print("Processing %s" % (file))
        gdb.execute("file %s" % (file))
        log_table = gdb.execute("info address _jdb_log_table", False, True)
        log_table_end = gdb.execute("info address _jdb_log_table_end", False, True)

        # is there any more direct way of getting the addresses?
        regexp = re.compile(r" (is at|at address) (0x\w+)")
        m_start = regexp.search(log_table)
        m_end = regexp.search(log_table_end)
        if not m_start or not m_end:
            raise gdb.GdbError("Failed to get _log_table and/or _log_table_end")

        log_table = int(m_start.group(2), 0)
        log_table_end = int(m_end.group(2), 0)
        log_table_entry = gdb.lookup_type("Tb_log_table_entry")

        for e in range(log_table, log_table_end, log_table_entry.sizeof):
            fullname = gdb.parse_and_eval("((Tb_log_table_entry *)%d)->name" % e)
            # advance to next string go get the shortname
            tag = fullname
            while True:
                v = gdb.parse_and_eval("*(char *)%d" % tag)
                tag += 1
                if v == 0:
                    break

            tag_s = tag.string()
            fn_s = fullname.string()

            if hasattr(self.db, tag_s) and self.db[tag_s] != fn_s:
                raise gdb.GdbError(
                    "Mismatch, should not happen (%s vs %s for %s)"
                    % (self.db[tag_s], fn_s, tag_s)
                )

            self.db[fn_s] = tag_s

    def get_log_table(self):
        print(sorted(self.db))
        return sorted(self.db)


class Fiasco_tbuf(gdb.Command):
    base_block_size = 0
    tb_entry_size = 0
    events = []
    event_to_num = {}
    log_table = {}

    # events where the babeltrace impl macro should not be added because they require some custom handling
    no_bt_impl = ["Ipc", "IpcRes", "KeBin", "Nam"]

    # some shortnames from the event structs are different than the ones from the log table...
    dyn_shortnames = {
        "csw": "context_switch",
        "des": "destroy",
        "exr": "exregs",
        "unm": "unmap",
    }

    ktrace_shortnames = {
        "Context::Drq_log": "drq",
        "Context::Vcpu_log": "vcpu",
        "Factory::Log_entry": "factory",
        "Ipc_gate::Log_ipc_gate_invoke": "gate",
        "Irq_base::Irq_log": "irq",
        "Kobject::Log_destroy": "destroy",
        "Kobject::Log_name": "nam",
        "Mu_log::Map_log": "map",
        "Mu_log::Unmap_log": "unmap",
        "Rcu::Log_rcu": "rcu",
        "Task::Log_map_unmap": "tmap",
        "Tb_entry_bp": "bp",
        "Tb_entry_ctx_sw": "context_switch",
        "Tb_entry_ipc": "ipc",
        "Tb_entry_ipc_res": "ipc_res",
        "Tb_entry_ipc_trace": "ipc_trace",
        "Tb_entry_empty": "empty",
        "Tb_entry_ke": "ke",
        "Tb_entry_ke_bin": "ke_bin",
        "Tb_entry_ke_reg": "ke_reg",
        "Tb_entry_pf": "pf",
        "Tb_entry_sched": "sched",
        "Tb_entry_trap": "trap",
        "Tb_entry_union": "fullsize",
        "Thread::Log_exc_invalid": "ieh",
        "Thread::Log_pf_invalid": "ipfh",
        "Thread::Log_thread_exregs": "exregs",
        "Thread::Migration_log": "migration",
        "Timer_tick::Log": "timer",
        "Vm_svm::Log_vm_svm_exit": "svm",
    }

    # Non-simple types
    known_types_map = {
        "Cap_index": "u64",
        "Cpu_number": "u32",
        "Context::Drq_log::Type": "u32",
        "L4_msg_tag": "u64",
        "L4_obj_ref": "u64",
        "L4_timeout_pair": "u32",
        "L4_error": "u64",
        "cxx::Type_info": "u64",
    }

    # map for equivalents of C types in Rust
    c_to_rust_map = {
        "char": "i8",
        "unsigned char": "u8",
        "unsigned short": "u16",
        "int": "i32",
        "unsigned int": "u32",
        "long": "i64",
        "unsigned long": "u64",
        "unsigned long long": "u64",
        "void": "u64",
    }

    printlog_buf_current = 0
    printlog_buf = ["", "", ""]

    def __init__(self):
        super(Fiasco_tbuf, self).__init__("fiasco-gen-ktrace-events", gdb.COMMAND_DATA)

    def printlog(self, s):
        print(s, end=" ")
        self.printlog_buf[self.printlog_buf_current] += s

    def printlogi(self, indentlevel, s):
        ins = " " * (indentlevel * 1)
        print("%s%s" % (ins, s), end=" ")
        self.printlog_buf[self.printlog_buf_current] += ins + s

    def printlog_set_current_section(self, section):
        self.printlog_buf_current = section

    def printlog_write(self, file):
        dir = "../src/event"
        os.makedirs(dir, exist_ok=True)

        with open(dir + "/" + file, "w") as f:
            f.write(self.printlog_buf[0])
            f.write(self.printlog_buf[1])
            f.write(self.printlog_buf[2])

        self.printlog_buf = ["", "", ""]

    def handle_type_pointer(self, t):
        rt = str(t)

        if rt != "void" and rt != "char":
            return self.convert_c_type_to_rust("void")
        else:
            return self.convert_c_type_to_rust(rt)

    def handle_type(self, t):
        if t.name in self.known_types_map:
            return self.known_types_map[t.name]

        if t.name == "bool":
            return "u8"

        rtbasic = str(gdb.types.get_basic_type(t))

        if str(rtbasic) != t.name:
            return self.convert_c_type_to_rust(rtbasic)
        return self.convert_c_type_to_rust(t.name)

    def print_members(self, t, prepad, postpad=False, indent=INDENT_SIZE):
        behind_last_member = 0
        padidx = 1
        cur_size = 0
        for f in sorted(t.fields(), key=lambda x: getattr(x, "bitpos", -1)):
            if f.name == "Tb_entry":
                continue

            if f.name == "type":
                f.name = "type_"  # type is a keyword in rust

            if hasattr(f, "bitpos"):
                byteoff = f.bitpos // 8
                if byteoff * 8 != f.bitpos:
                    raise gdb.GdbError("Don't know how to handle bitfields, hack me")

                if prepad:
                    prepad = False
                    if self.base_block_size != 0 and byteoff != self.base_block_size:
                        # Add padding
                        padding = byteoff - self.base_block_size
                        self.printlogi(indent, "pub __pre_pad: [i8; %d],\n" % padding)
                elif cur_size < byteoff:
                    padding = byteoff - cur_size
                    self.printlogi(
                        indent, "pub __pad_%d: [i8; %d],\n" % (padidx, padding)
                    )
                    padidx += 1

                behind_last_member = byteoff + f.type.sizeof
                if f.type.code == gdb.TYPE_CODE_ARRAY:
                    tc = self.handle_type(f.type.target().unqualified())
                    c = "pub %s: [%s; %d]" % (
                        f.name.removeprefix("_"),
                        tc,
                        f.type.range()[1] + 1,
                    )
                    self.printlogi(indent, "%s, \n" % (c))
                elif f.type.code == gdb.TYPE_CODE_PTR:
                    tc = self.handle_type_pointer(f.type.target().unqualified())
                    self.printlogi(
                        indent,
                        "pub %s: %s,\n" % (f.name.removeprefix("_"), tc),
                    )

                # TODO
                elif (
                    f.type.code in [gdb.TYPE_CODE_UNION, gdb.TYPE_CODE_STRUCT]
                    and str(f.type.unqualified()) not in self.known_types_map
                ):
                    if f.type.code is gdb.TYPE_CODE_STRUCT:
                        self.printlogi(indent, "%s {\n" % (f.name.removeprefix("_")))
                        self.print_members(f.type, False, False, indent + 2)
                        self.printlogi(indent, "},\n")
                    else:
                        self.printlogi(
                            indent, "enum %s {\n" % (f.name.removeprefix("_"))
                        )
                        self.print_members(f.type, False, False, indent + 2)
                        self.printlogi(indent, "},\n")

                else:
                    tc = self.handle_type(f.type.unqualified())
                    name = f.name.removeprefix("_")
                    if name == "type":
                        name = "type_"
                    self.printlogi(
                        indent,
                        "pub %s: %s,\n" % (name, tc),
                    )

                cur_size = byteoff + f.type.sizeof

        if postpad:
            if behind_last_member > self.tb_entry_size:
                raise gdb.GdbError(
                    "Error: %s is too big (expected <= %d)"
                    % (t.name, self.tb_entry_size)
                )
            sz = self.tb_entry_size - behind_last_member
            self.printlogi(
                indent,
                "char __post_pad[%d],\n" % (sz),
            )

        return behind_last_member

    def print_single_struct(self, t, sname):
        # TODO ke and ke_reg are a bit complicated to implement, have to do that sometime
        if sname == "Ke" or sname == "KeReg":
            self.printlogi(0, "//TODO not yet implemented\n")
            self.print_derive_traits(sname)
            self.printlog("#[br(little)]\n")
            self.printlogi(0, "pub struct %sEvent {\n" % sname)
            self.printlogi(INDENT_SIZE, "pub common: EventCommon,\n\n")
            self.printlogi(0, "}\n")
            return

        self.print_derive_traits(sname)
        self.printlog("#[br(little)]\n")
        self.printlogi(0, "pub struct %sEvent {\n" % sname)
        self.printlogi(INDENT_SIZE, "pub common: EventCommon,\n\n")
        self.print_members(t, True, sname == "fullsize", INDENT_SIZE)
        self.printlogi(0, "}\n")

    def gen_ktrace_events(self, tbentry_types):
        # get event numbers (fixed and dyn)
        t = gdb.lookup_type("Tbuf_entry_fixed")
        if t.code is gdb.TYPE_CODE_ENUM:
            for f in t.fields():
                name = self.to_camel_case(f.name).removeprefix("Tbuf")
                if name == "Dynentries":
                    dyn_entry_offset = f.enumval
                    for idx, e in enumerate(self.log_table):
                        e = self.to_camel_case(self.dyn_shortnames.get(e, e))
                        self.event_to_num[e] = dyn_entry_offset + idx
                else:
                    if name != "Max":  # NOTE Max and Hidden have the same number
                        self.event_to_num[name] = f.enumval
        else:
            raise gdb.GdbError("Missing Tbuf_entry_fixed, old Fiasco?")

        # Unfortunately we are not able to extract Tb_entry::Tb_entry_size
        # so apply this guess:
        mword = gdb.lookup_type("Mword")
        self.tb_entry_size = 128 if mword.sizeof == 8 else 64

        print("Guessed Tb_entry size:", self.tb_entry_size)

        # Print struct for common event
        self.printlog("/* Note, automatically generated from Fiasco binary */\n")
        self.printlog("\n")
        self.printlog("use binrw::BinRead;\n\n")
        self.print_derive_traits()
        self.printlog("#[br(little)]\n")
        self.printlog("pub struct EventCommon {\n")
        self.base_block_size = self.print_members(gdb.lookup_type("Tb_entry"), False)
        self.printlog("}\n")
        self.printlog("\n")

        self.printlog_write("common.rs")

        # Print structs for individual event types
        for i in sorted(tbentry_types, key=lambda t: t.name):
            if i.name in self.ktrace_shortnames:
                self.printlog(
                    "/* Note, automatically generated from Fiasco binary */\n"
                )
                self.printlog("\n")

                name = self.ktrace_shortnames[i.name]
                self.events.append(name)

                self.printlog("#[allow(unused_imports)]\n")
                self.printlog("use ctf_macros::CtfEventClass;\n\n")
                self.printlog("use super::common::EventCommon;\n")
                self.printlog("use binrw::BinRead;\n")
                self.print_single_struct(i, self.to_camel_case(name))
                self.printlog("\n")
                self.printlog_write(name + ".rs")
            else:
                raise gdb.GdbError(
                    "Missing '%s' in internal knowledge base. Please add." % (i.name)
                )

        # print EventType enum
        self.gen_event_type()

    def get_tbentry_classes(self):
        print("Querying Tb_entry types. This might take a while.")
        # Is there any faster way of doing this?
        output = gdb.execute("info types", False, True)
        regexp = re.compile(r"^(?:\d+:\s+)?(\S+);$")  # should fetch all we need
        types = []
        for line in output.split("\n"):
            m = regexp.match(line)
            if m:
                try:
                    t = gdb.lookup_type(m.group(1))
                    if "Tb_entry" in t and t["Tb_entry"].is_base_class:
                        types.append(t)
                except gdb.error:
                    pass
        return types

    def convert_c_type_to_rust(self, s):
        return self.c_to_rust_map[s]

    def to_camel_case(self, s):
        s = "".join(t.title() for t in s.split())
        s = s.replace("-", "")
        s = s.replace("(", "")
        s = s.replace(")", "")
        s = s.replace("_", "")
        return s

    def print_derive_traits(self, name=""):
        if name != "" and name not in self.no_bt_impl:
            self.printlog(
                "#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, CtfEventClass)]\n"
            )
            self.printlog('#[event_name = "%s"]\n' % name.upper())
        else:
            self.printlog(
                "#[derive(BinRead, Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]\n"
            )

    def gen_event_type(self):
        self.printlog("/* Note, automatically generated from Fiasco binary */\n")
        self.printlog("\n")

        self.printlog("use core::fmt;\n")
        self.printlog("use num_enum::{IntoPrimitive, TryFromPrimitive};\n")
        self.printlog("\n")

        self.printlog(
            "#[derive(Copy, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive)]\n"
        )
        self.printlog("#[repr(u8)]\n")
        self.printlog("pub enum EventType {\n")
        for event, number in self.event_to_num.items():
            self.printlogi(INDENT_SIZE, "%s = %s,\n" % (event, str(number)))
        self.printlog("}\n\n")

        # Display trait EventType
        self.printlog("impl fmt::Display for EventType {\n")
        self.printlogi(
            1 * INDENT_SIZE, "fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n"
        )
        self.printlogi(2 * INDENT_SIZE, "use EventType::*;\n")
        self.printlogi(2 * INDENT_SIZE, "match self {\n")
        for event in self.event_to_num.keys():
            self.printlogi(
                3 * INDENT_SIZE,
                '%s => write!(f, "%s"), \n' % (event, event.upper()),
            )
        self.printlogi(2 * INDENT_SIZE, "}\n")
        self.printlogi(1 * INDENT_SIZE, "}\n")
        self.printlog("}\n\n")
        self.printlog_write("event_type.rs")

    def invoke(self, argument, from_tty):
        argv = gdb.string_to_argv(argument)
        if len(argv) < 1:
            print("Need 1st arg img file")
            exit(1)
        tbentry_types = self.get_tbentry_classes()
        self.log_table = Log_table([argv[0]]).get_log_table()
        self.gen_ktrace_events(tbentry_types)


Fiasco_tbuf()

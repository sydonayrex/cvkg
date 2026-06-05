import re
import os

with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    lines = f.readlines()

def write_file(filename, ranges, extra_imports=""):
    with open(f'../cvkg-render-gpu/src/{filename}', 'w') as f:
        f.write("#![allow(unused_imports, dead_code)]\n")
        f.write("use super::*;\n")
        f.write(extra_imports)
        f.write("\n")
        for (start, end) in ranges:
            # line numbers are 1-indexed
            f.writelines(lines[start-1:end])

# I need the line numbers for all these structures!

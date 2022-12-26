from __future__ import print_function
import sys

print("error!", file=sys.stderr)
sys.exit(3)

# args: -c 'print("test"); exec(open("examples/all_keywords.py").read())'
# expected exit status: 3
# expected stdout: test

# expected stderr: error!


#!/usr/bin/env python3
from cs6991 import test
import os
import sys

test.init(__file__, compile_commands=["./check_mark_request.sh #"])

def process_stages():
    for stage in ["01", "02", "03", "04", "05"]:
        stage_path = os.path.join(f"fixtures/staged/{stage}/")
        process_dir(stage, stage_path)

def process_dir(stage, dir_path):
    for file in os.listdir(dir_path):
        name = file.removesuffix(".yml")
        full_path = os.path.join(dir_path, file)
        if os.path.isdir(full_path):
            process_dir(full_path)
        elif file.endswith(".yml"):
            print(f"Generating test for {file}", file=sys.stderr)
            test.case('', test_name=f"stage{stage}_{name}", args=[full_path])

process_stages()

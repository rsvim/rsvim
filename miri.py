#!/usr/bin/env python3

# Formatted with ruff.

import os
import argparse
import logging
import subprocess

LOGGING_LEVEL = logging.INFO
TOTAL_JOBS = 10


def start(tests_list, job_index, package):
    job_index = int(job_index)
    tests_list = tests_list.split(",")
    tests_list = [t.strip() for t in tests_list if len(t.strip()) > 0]
    logging.debug(f"tests_list:{tests_list}")

    total_tests = len(tests_list)
    tests_per_job = total_tests // TOTAL_JOBS
    start_job_index = job_index * tests_per_job
    end_job_index = (job_index + 1) * tests_per_job
    is_last_job = job_index == TOTAL_JOBS - 1
    logging.info(
        f"total_tests:{total_tests}, tests_per_job:{tests_per_job}, is_last_job:{is_last_job} start_job_index:{start_job_index}, end_job_index:{end_job_index}"
    )
    if is_last_job:
        tests_list = [t for i, t in enumerate(tests_list) if i >= start_job_index]
    else:
        tests_list = [
            t
            for i, t in enumerate(tests_list)
            if i >= start_job_index and i < end_job_index
        ]
    tests_list = " ".join(tests_list)
    logging.debug(f"filtered tests_list:{tests_list}")
    command = f"cargo +nightly miri nextest run -F unicode_lines --no-default-features -p {package} {tests_list}"
    logging.info(command)
    os.system(command)


def generate():
    command_args = ["cargo", "+nightly", "nextest", "list", "--color=never"]
    logging.debug(command_args)
    tests_list_result = subprocess.run(command_args, capture_output=True, text=True)
    tests_list = tests_list_result.stdout.splitlines()
    tests_list = [
        t.strip() for i, t in enumerate(tests_list) if i > 0 and len(t.strip()) > 0
    ]
    tests_list = ",".join(tests_list)
    print(tests_list)


if __name__ == "__main__":
    logging.basicConfig(format="%(levelname)s: %(message)s", level=LOGGING_LEVEL)

    parser = argparse.ArgumentParser(
        description="help running miri tests in parallel groups"
    )
    parser.add_argument(
        "--generate",
        action="store_true",
        help="Generate cargo tests list",
    )
    parser.add_argument(
        "--job",
        help="Run cargo miri tests job in [0-9]",
    )
    parser.add_argument(
        "--package",
        help="Run cargo miri tests job with [PACKAGE] name",
    )
    parser.add_argument(
        "--tests",
        help="Run cargo miri tests job with [TESTS] list",
    )

    parser = parser.parse_args()
    # print(parser)

    if parser.generate:
        generate()
    elif parser.job:
        start(parser.tests, parser.job, parser.package)
    else:
        logging.error("Missing arguments, use -h/--help for more details.")

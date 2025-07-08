# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import subprocess


def head_commit(branch):
    if branch == "":
        raise ValueError("Expected non-empty branch name")

    git_cmd = f"git show --format=%H -s {branch}"
    bash_cmd = f"bash -l -c \'{git_cmd}\'"

    p = subprocess.Popen(bash_cmd, shell=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = p.communicate()
    stdout = stdout.replace("\n", "")

    if stdout == "":
        raise ValueError(f"Expected non-empty output for {git_cmd}")
    if stderr != "":
        raise ValueError(f"Expected empty error for {git_cmd}, got {stderr}")

    return stdout

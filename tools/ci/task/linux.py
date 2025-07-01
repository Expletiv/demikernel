# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

from ci.task.generic import BaseTask


class BaseLinuxTask(BaseTask):
    def __init__(self, host, cmd):
        ssh_cmd = f"ssh -C {host} \"bash -l -c \'{cmd}\'\""
        super().__init__(ssh_cmd)


class CheckoutOnLinux(BaseLinuxTask):
    def __init__(self, host, repository, branch):
        cmd = f"cd {repository} && git fetch origin && git checkout {branch} && git reset --hard {branch}"
        super().__init__(host, cmd)


class CompileOnLinux(BaseLinuxTask):
    def __init__(self, host, repository, target, is_debug):
        debug_flag = "DEBUG=yes" if is_debug else "DEBUG=no"
        profiler_flag = "PROFILER=yes" if not is_debug else "PROFILER=no"
        cmd = f"cd {repository} && make {profiler_flag} {debug_flag} {target}"
        super().__init__(host, cmd)


class RunOnLinux(BaseLinuxTask):
    def __init__(self, host, repository, target, is_debug, is_sudo, config_path):
        debug_flag = "DEBUG=yes" if is_debug else "DEBUG=no"
        sudo_cmd = "sudo -E" if is_sudo else ""
        profiler_flag = "PROFILER=yes" if not is_debug else "PROFILER=no"
        cmd = f"cd {repository} && {sudo_cmd} make -j 1 CONFIG_PATH={config_path} {profiler_flag} {debug_flag} {target} 2> out.stderr && cat out.stderr >&2 || ( cat out.stderr >&2 ; exit 1 )"
        super().__init__(host, cmd)


class CleanupOnLinux(BaseLinuxTask):
    def __init__(self, host, repository, is_sudo, branch):
        sudo_cmd = "sudo -E" if is_sudo else ""
        cmd = f"cd {repository} && {sudo_cmd} make clean && git checkout {branch} && git clean -fdx ; sudo -E rm -rf /dev/shm/demikernel* ; sudo pkill -f demikernel*"
        super().__init__(host, cmd)


class CloneOnLinux(BaseLinuxTask):
    def __init__(self, host, path, repository, branch):
        cmd = f"cd {path} && git clone {repository} --branch {branch}"
        super().__init__(host, cmd)

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

from ci.task.generic import BaseTask


class BaseWindowsTask(BaseTask):

    def __init__(self, host, cmd):
        ssh_cmd = f"ssh {host} \"{cmd}\""
        super().__init__(ssh_cmd)

    @staticmethod
    def _build_env_cmd():
        rust_path = "\\$RustPath = Join-Path \\$Env:HOME \\.cargo\\bin"
        git_path = "\\$GitPath = Join-Path \\$Env:ProgramFiles \\Git\\cmd"
        env_path_git = "\\$Env:Path += \\$GitPath + \';\'"
        env_path_rust = "\\$Env:Path += \\$RustPath + \';\'"
        vs_install_path = "\\$VsInstallPath = &(Join-Path \\${Env:ProgramFiles(x86)} '\\Microsoft Visual Studio\\Installer\\vswhere.exe') -latest -property installationPath"
        import_module = "Import-Module (Join-Path \\$VsInstallPath 'Common7\\Tools\\Microsoft.VisualStudio.DevShell.dll')"
        enter_vsdevshell = "Enter-VsDevShell -VsInstallPath \\$VsInstallPath -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'"

        env_cmd = " ; ".join([rust_path, git_path, env_path_git, env_path_rust, vs_install_path, import_module,
                              enter_vsdevshell])
        return env_cmd


class CheckoutOnWindows(BaseWindowsTask):
    def __init__(self, host, repository, branch):
        env_cmd = BaseWindowsTask._build_env_cmd()
        cmd = f"cd {repository} ; {env_cmd} ; git fetch origin ; git checkout {branch}; git reset --hard {branch}"
        super().__init__(host, cmd)


class CompileOnWindows(BaseWindowsTask):
    def __init__(self, host, repository, target, is_debug):
        env_cmd = BaseWindowsTask._build_env_cmd()
        debug_flag = "DEBUG=yes" if is_debug else "DEBUG=no"
        profiler_flag = "PROFILER=yes" if not is_debug else "PROFILER=no"
        cmd = f"cd {repository} ; {env_cmd} ; nmake {profiler_flag} {debug_flag} {target}"
        super().__init__(host, cmd)


class RunOnWindows(BaseWindowsTask):
    def __init__(self, host, repository, target, is_debug, _is_sudo, config_path):
        env_cmd = BaseWindowsTask._build_env_cmd()
        debug_flag = "DEBUG=yes" if is_debug else "DEBUG=no"
        cmd = f"cd {repository} ; {env_cmd} ; nmake CONFIG_PATH={config_path} {debug_flag} {target}"
        super().__init__(host, cmd)


class CleanupOnWindows(BaseWindowsTask):
    def __init__(self, host, repository, _is_sudo, branch):
        env_cmd = BaseWindowsTask._build_env_cmd()
        cmd = f"cd {repository} ; {env_cmd} ; nmake clean ; git checkout {branch}; git clean -fdx"
        super().__init__(host, cmd)

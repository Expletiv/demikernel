# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import time
import ci.task.linux as linux
from ci.task.linux import CheckoutOnLinux, CompileOnLinux, RunOnLinux, CleanupOnLinux
from ci.job.utils import wait_and_report
from ci.job.generic import BaseJob

class BaseLinuxJob(BaseJob):
    def __init__(self, config, name):
        super().__init__(config, name)

    def execute(self, serverTask, clientTask=None, no_wait=False):
        return super().execute(serverTask, clientTask, no_wait)


class CheckoutJobOnLinux(BaseLinuxJob):
    def __init__(self, config):
        super().__init__(config, "checkout")

    def execute(self):
        serverTask = CheckoutOnLinux(super().server(), super().repository(), super().branch())
        if not super().enable_nfs():
            clientTask = CheckoutOnLinux(super().client(), super().repository(), super().branch())
            return super().execute(serverTask, clientTask)
        return super().execute(serverTask)


class CompileJobOnLinux(BaseLinuxJob):
    def __init__(self, config):
        name = "compile-{}".format("debug" if config["is_debug"] else "release")
        super().__init__(config, name)

    def execute(self):
        cmd = f"all LIBOS={super().libos()}"
        serverTask = CompileOnLinux(super().server(), super().repository(), cmd, super().is_debug())
        if not super().enable_nfs():
            clientTask = CompileOnLinux(super().client(), super().repository(), cmd, super().is_debug())
            return super().execute(serverTask, clientTask)
        return super().execute(serverTask)


class CleanupJobOnLinux(BaseLinuxJob):
    def __init__(self, config):
        super().__init__(config, "cleanup")

    def execute(self):
        default_branch = "dev"
        serverTask = CleanupOnLinux(super().server(), super().repository(), super().is_sudo(), default_branch)
        if not super().enable_nfs():
            clientTask = CleanupOnLinux(super().client(), super().repository(), super().is_sudo(), default_branch)
            return super().execute(serverTask, clientTask)
        return super().execute(serverTask)


class UnitTestJobOnLinux(BaseLinuxJob):
    def __init__(self, config, name):
        super().__init__(config, name)

    def execute(self):
        server_cmd = f"{super().name()} LIBOS={super().libos()}"
        serverTask = RunOnLinux(super().server(), super().repository(), server_cmd, super().is_debug(),
                                super().is_sudo(), super().config_path())
        return super().execute(serverTask)


class UnitTestRustJobOnLinux(UnitTestJobOnLinux):
    def __init__(self, config):
        super().__init__(config, "test-unit-rust")

    def execute(self):
        return super().execute()


class UnitTestCJobOnLinux(UnitTestJobOnLinux):
    def __init__(self, config):
        super().__init__(config, "test-unit-c")

    def execute(self):
        return super().execute()


class EndToEndTestJobOnLinux(BaseLinuxJob):

    def __init__(self, config, job_name):
        self.job_name = job_name
        self.all_pass = config["all_pass"]
        super().__init__(config, job_name)

    def execute(self, server_cmd, client_cmd):
        serverTask = RunOnLinux(super().server(), super().repository(), server_cmd, super().is_debug(),
                                super().is_sudo(), super().config_path())
        jobs = {}
        jobs[self.job_name + "-server-" + super().server()] = serverTask.execute()
        time.sleep(super().delay())
        clientTask = RunOnLinux(super().client(), super().repository(), client_cmd, super().is_debug(),
                                super().is_sudo(), super().config_path())
        jobs[self.job_name + "-client-" + super().client()] = clientTask.execute()
        return wait_and_report(self.job_name, super().log_directory(), jobs, self.all_pass)


class SystemTestJobOnLinux(EndToEndTestJobOnLinux):
    def __init__(self, config):
        self.test_name = config["test_name"]
        self.server_args = config["server_args"]
        self.client_args = config["client_args"]
        super().__init__(config, f"system-test-{config['test_alias']}")

    def execute(self):
        server_cmd = f"test-system-rust LIBOS={super().libos()} TEST={self.test_name} ARGS=\\\"{self.server_args}\\\""
        client_cmd = f"test-system-rust LIBOS={super().libos()} TEST={self.test_name} ARGS=\\\"{self.client_args}\\\""
        return super().execute(server_cmd, client_cmd)


class TcpCloseTest(SystemTestJobOnLinux):
    def __init__(self, config, run_mode, who_closes, nclients):
        config["test_name"] = "tcp-close"
        config["test_alias"] = f"tcp-close-{run_mode}-{who_closes}-closes-sockets"
        config["all_pass"] = True
        config["server_args"] = f"--peer server --address {config['server_addr']}:12345 --nclients {nclients} --run-mode {run_mode} --whocloses {who_closes}"
        config["client_args"] = f"--peer client --address {config['server_addr']}:12345 --nclients {nclients} --run-mode {run_mode} --whocloses {who_closes}"
        super().__init__(config)


class TcpEchoTest(SystemTestJobOnLinux):
    def __init__(self, config, run_mode, nclients, bufsize, nrequests, nthreads):
        config["test_name"] = "tcp-echo"
        config["test_alias"] = f"tcp-echo-{run_mode}-{nclients}-{bufsize}-{nrequests}-{nthreads}"
        config["all_pass"] = True
        config["server_args"] = f"--peer server --address {config['server_addr']}:12345 --nthreads {nthreads}"
        config["client_args"] = f"--peer client --address {config['server_addr']}:12345 --nclients {nclients} --nrequests {nrequests} --bufsize {bufsize} --run-mode {run_mode}"
        super().__init__(config)


class TcpPingPongTest(SystemTestJobOnLinux):
    def __init__(self, config):
        config["test_name"] = "tcp-ping-pong"
        config["test_alias"] = "tcp-ping-pong"
        config["all_pass"] = True
        config["server_args"] = f"--server {config['server_addr']}:12345"
        config["client_args"] = f"--client {config['server_addr']}:12345"
        super().__init__(config)


class TcpPushPopTest(SystemTestJobOnLinux):
    def __init__(self, config):
        config["test_name"] = "tcp-push-pop"
        config["test_alias"] = "tcp-push-pop"
        config["all_pass"] = True
        config["server_args"] = f"--server {config['server_addr']}:12345"
        config["client_args"] = f"--client {config['server_addr']}:12345"
        super().__init__(config)


class TcpWaitTest(SystemTestJobOnLinux):
    def __init__(self, config, scenario, nclients):
        config["test_name"] = "tcp-wait"
        config["test_alias"] = f"tcp-wait-scenario-{scenario}"
        config["all_pass"] = True
        config["server_args"] = f"--peer server --address {config['server_addr']}:12345 --nclients {nclients} --scenario {scenario}"
        config["client_args"] = f"--peer client --address {config['server_addr']}:12345 --nclients {nclients} --scenario {scenario}"
        super().__init__(config)


class UdpPingPongTest(SystemTestJobOnLinux):
    def __init__(self, config):
        config["test_name"] = "udp-ping-pong"
        config["test_alias"] = "udp-ping-pong"
        config["all_pass"] = False
        config["server_args"] = f"--server {config['server_addr']}:12345 {config['client_addr']}:23456"
        config["client_args"] = f"--client {config['client_addr']}:23456 {config['server_addr']}:12345"
        super().__init__(config)


class UdpPushPopTest(SystemTestJobOnLinux):
    def __init__(self, config):
        config["test_name"] = "udp-push-pop"
        config["test_alias"] = "udp-push-pop"
        config["all_pass"] = True
        config["server_args"] = f"--server {config['server_addr']}:12345 {config['client_addr']}:23456"
        config["client_args"] = f"--client {config['client_addr']}:23456 {config['server_addr']}:12345"
        super().__init__(config)


class IntegrationTestJobOnLinux(BaseLinuxJob):
    def __init__(self, config, test_name):
        config["all_pass"] = True
        super().__init__(config, f"integration-test ({test_name})")
        self.server_args = f"--local-address {super().server_addr()}:12345 --remote-address {super().client_addr()}:23456"
        self.test_name = test_name

    def execute(self):
        server_cmd = f"test-integration-rust TEST_INTEGRATION={self.test_name} LIBOS={super().libos()} ARGS=\\\"{self.server_args}\\\""
        serverTask = RunOnLinux(super().server(), super().repository(), server_cmd, super().is_debug(),
                                super().is_sudo(), super().config_path())
        return super().execute(serverTask)


def job_test_system_rust(test_alias, test_name, repo, libos, is_debug, server, client, server_args, client_args,
                         is_sudo, all_pass, delay, config_path, log_directory):
    config = {
        "test_alias": test_alias,
        "test_name": test_name,
        "repository": repo,
        "libos": libos,
        "is_debug": is_debug,
        "server": server,
        "client": client,
        "server_args": server_args,
        "client_args": client_args,
        "is_sudo": is_sudo,
        "all_pass": all_pass,
        "delay": delay,
        "config_path": config_path,
        "log_directory": log_directory
    }
    job = SystemTestJobOnLinux(config)
    return job.execute()


class InstallJobOnLinux(BaseLinuxJob):
    def __init__(self, config):
        name = "install-{}".format("debug" if config["is_debug"] else "release")
        super().__init__(config, name)

    def execute(self):
        cmd = f"LIBOS={super().libos()} INSTALL_PREFIX={super().install_prefix()} install"
        serverTask = linux.CompileOnLinux(super().server(), super().repository(), cmd, super().is_debug())
        if not super().enable_nfs():
            clientTask = linux.CompileOnLinux(super().client(), super().repository(), cmd, super().is_debug())
            return super().execute(serverTask, clientTask)
        return super().execute(serverTask)

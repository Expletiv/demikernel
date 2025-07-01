# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import time
from ci.task.windows import CheckoutOnWindows, CompileOnWindows, RunOnWindows, CleanupOnWindows
from ci.job.utils import wait_and_report
from ci.job.generic import BaseJob


class BaseWindowsJob(BaseJob):
    def __init__(self, config, name):
        super().__init__(config, name)

    def execute(self, serverTask, clientTask=None):
        return super().execute(serverTask, clientTask)


class CheckoutJobOnWindows(BaseWindowsJob):
    def __init__(self, config):
        super().__init__(config, "checkout")

    def execute(self):
        serverTask = CheckoutOnWindows(super().server(), super().repository(), super().branch())
        if not super().enable_nfs():
            clientTask = CheckoutOnWindows(super().client(), super().repository(), super().branch())
            return super().execute(serverTask, clientTask)
        return super().execute(serverTask)


class CompileJobOnWindows(BaseWindowsJob):
    def __init__(self, config):
        name = "compile-{}".format("debug" if config["is_debug"] else "release")
        super().__init__(config, name)

    def execute(self):
        cmd = f"all LIBOS={super().libos()}"
        server_task = CompileOnWindows(super().server(), super().repository(), cmd, super().is_debug())
        if not super().enable_nfs():
            client_task = CompileOnWindows(super().client(), super().repository(), cmd, super().is_debug())
            return super().execute(server_task, client_task)
        return super().execute(server_task)


class CleanupJobOnWindows(BaseWindowsJob):
    def __init__(self, config):
        super().__init__(config, "cleanup")

    def execute(self):
        default_branch = "dev"
        server_task = CleanupOnWindows(super().server(), super().repository(), super().is_sudo(), default_branch)
        if not super().enable_nfs():
            client_task = CleanupOnWindows(super().client(), super().repository(), super().is_sudo(), default_branch)
            return super().execute(server_task, client_task)
        return super().execute(server_task)


class UnitTestJobOnWindows(BaseWindowsJob):
    def __init__(self, config, name):
        super().__init__(config, name)

    def execute(self):
        server_cmd = f"{super().name()} LIBOS={super().libos()}"
        server_task = RunOnWindows(super().server(), super().repository(), server_cmd, super().is_debug(),
                                   super().is_sudo(), super().config_path())
        return super().execute(server_task)


class UnitTestRustJobOnWindows(UnitTestJobOnWindows):
    def __init__(self, config):
        super().__init__(config, "test-unit-rust")

    def execute(self):
        return super().execute()


class UnitTestCJobOnWindows(UnitTestJobOnWindows):
    def __init__(self, config):
        super().__init__(config, "test-unit-c")

    def execute(self):
        return super().execute()


class EndToEndTestJobOnWindows(BaseWindowsJob):
    def __init__(self, config, job_name):
        self.job_name = job_name
        self.all_pass = config["all_pass"]
        super().__init__(config, job_name)

    def execute(self, server_cmd, client_cmd):
        serverTask = RunOnWindows(super().server(), super().repository(), server_cmd, super().is_debug(),
                                  super().is_sudo(), super().config_path())
        jobs = {}
        jobs[self.job_name + "-server-" + super().server()] = serverTask.execute()
        time.sleep(super().delay())
        clientTask = RunOnWindows(super().client(), super().repository(), client_cmd, super().is_debug(),
                                  super().is_sudo(), super().config_path())
        jobs[self.job_name + "-client-" + super().client()] = clientTask.execute()
        return wait_and_report(self.job_name, super().log_directory(), jobs, self.all_pass)


class SystemTestJobOnWindows(EndToEndTestJobOnWindows):
    def __init__(self, config):
        self.test_name = config["test_name"]
        self.server_args = config["server_args"]
        self.client_args = config["client_args"]
        super().__init__(config, f"system-test-{config['test_alias']}")

    def execute(self):
        server_cmd = f"test-system-rust LIBOS={super().libos()} TEST={self.test_name} ARGS=\'{self.server_args}\'"
        client_cmd = f"test-system-rust LIBOS={super().libos()} TEST={self.test_name} ARGS=\'{self.client_args}\'"
        return super().execute(server_cmd, client_cmd)


class TcpCloseTest(SystemTestJobOnWindows):
    def __init__(self, config, run_mode, who_closes, nclients):
        config["test_name"] = "tcp-close"
        config["test_alias"] = f"tcp-close-{run_mode}-{who_closes}-closes-sockets"
        config["all_pass"] = True
        config["server_args"] = f"--peer server --address {config['server_addr']}:12345 --nclients {nclients} --run-mode {run_mode} --whocloses {who_closes}"
        config["client_args"] = f"--peer client --address {config['server_addr']}:12345 --nclients {nclients} --run-mode {run_mode} --whocloses {who_closes}"
        super().__init__(config)


class TcpEchoTest(SystemTestJobOnWindows):
    def __init__(self, config, run_mode, nclients, bufsize, nrequests, nthreads):
        config["test_name"] = "tcp-echo"
        config["test_alias"] = f"tcp-echo-{run_mode}-{nclients}-{bufsize}-{nrequests}-{nthreads}"
        config["all_pass"] = True
        config["server_args"] = f"--peer server --address {config['server_addr']}:12345 --nthreads {nthreads}"
        config["client_args"] = f"--peer client --address {config['server_addr']}:12345 --nclients {nclients} --nrequests {nrequests} --bufsize {bufsize} --run-mode {run_mode}"
        super().__init__(config)


class TcpPingPongTest(SystemTestJobOnWindows):
    def __init__(self, config):
        config["test_name"] = "tcp-ping-pong"
        config["test_alias"] = "tcp-ping-pong"
        config["all_pass"] = True
        config["server_args"] = f"--server {config['server_addr']}:12345"
        config["client_args"] = f"--client {config['server_addr']}:12345"
        super().__init__(config)


class TcpPushPopTest(SystemTestJobOnWindows):
    def __init__(self, config):
        config["test_name"] = "tcp-push-pop"
        config["test_alias"] = "tcp-push-pop"
        config["all_pass"] = True
        config["server_args"] = f"--server {config['server_addr']}:12345"
        config["client_args"] = f"--client {config['server_addr']}:12345"
        super().__init__(config)


class TcpWaitTest(SystemTestJobOnWindows):
    def __init__(self, config, scenario, nclients):
        config["test_name"] = "tcp-wait"
        config["test_alias"] = f"tcp-wait-scenario-{scenario}"
        config["all_pass"] = True
        config["server_args"] = f"--peer server --address {config['server_addr']}:12345 --nclients {nclients} --scenario {scenario}"
        config["client_args"] = f"--peer client --address {config['server_addr']}:12345 --nclients {nclients} --scenario {scenario}"
        super().__init__(config)


class UdpPingPongTest(SystemTestJobOnWindows):
    def __init__(self, config):
        config["test_name"] = "udp-ping-pong"
        config["test_alias"] = "udp-ping-pong"
        config["all_pass"] = False
        config["server_args"] = f"--server {config['server_addr']}:12345 {config['client_addr']}:23456"
        config["client_args"] = f"--client {config['client_addr']}:23456 {config['server_addr']}:12345"
        super().__init__(config)


class UdpPushPopTest(SystemTestJobOnWindows):
    def __init__(self, config):
        config["test_name"] = "udp-push-pop"
        config["test_alias"] = "udp-push-pop"
        config["all_pass"] = True
        config["server_args"] = f"--server {config['server_addr']}:12345 {config['client_addr']}:23456"
        config["client_args"] = f"--client {config['client_addr']}:23456 {config['server_addr']}:12345"
        super().__init__(config)


class IntegrationTestJobOnWindows(BaseWindowsJob):
    def __init__(self, config, test_name):
        config["all_pass"] = True
        super().__init__(config, f"integration-test ({test_name})")
        self.server_args = f"--local-address {super().server_addr()}:12345 --remote-address {super().client_addr()}:23456"
        self.test_name = test_name

    def execute(self):
        server_cmd = f"test-integration-rust TEST_INTEGRATION={self.test_name} LIBOS={super().libos()} ARGS=\'{self.server_args}\'"
        serverTask = RunOnWindows(super().server(), super().repository(), server_cmd, super().is_debug(),
                                  super().is_sudo(), super().config_path())
        return super().execute(serverTask)

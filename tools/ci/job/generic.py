# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

from ci.job.utils import wait_and_report

class BaseJob:
    def __init__(self, config, name):
        self.name = name
        self.config = config

    def execute(self, serverTask, clientTask, no_wait=False):
        jobs = {}
        jobs[self.name + "-server-" + self.server()] = serverTask.execute()
        if clientTask is not None:
            jobs[self.name + "-client-" + self.client()] = clientTask.execute()
        return wait_and_report(self.name, self.log_directory(), jobs, no_wait)

    def name(self):
        return self.name

    def branch(self):
        return self.config["branch"]

    def server(self):
        return self.config["server"]

    def client(self):
        return self.config["client"]

    def repository(self):
        return self.config["repository"]

    def enable_nfs(self):
        return self.config["enable_nfs"]

    def is_sudo(self):
        return self.config["is_sudo"]

    def config_path(self):
        return self.config["config_path"]

    def is_debug(self):
        return self.config["is_debug"]

    def libos(self):
        return self.config["libos"]

    def log_directory(self):
        return self.config["log_directory"]

    def delay(self):
        return self.config["delay"]

    def server_addr(self):
        return self.config["server_addr"]

    def client_addr(self):
        return self.config["client_addr"]

    def install_prefix(self):
        return self.config["install_prefix"]

    def ld_library_path(self):
        return self.config["ld_library_path"]

    def path(self):
        return self.config["path"]

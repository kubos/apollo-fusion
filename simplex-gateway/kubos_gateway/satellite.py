# Copyright 2019 Kubos Corporation
# Licensed under the Apache License, Version 2.0
# See LICENSE file for details.

"""
System configuration module

Connects all running services to MT
"""

import asyncio
import logging

logger = logging.getLogger(__name__)


class Satellite:
    def __init__(self, major_tom, host, system_name):
        self.major_tom = major_tom
        self.host = host
        self.system_name = system_name
        self.registry = []

    def register_service(self, *services):
        for service in services:
            self.registry.append(service)
            service.satellite = self

    async def send_metrics_to_mt(self, metrics):
        # {'parameter': 'voltage', 'subsystem': 'eps',
        #  'timestamp': 1531412196.211, 'value': '0.15'}
        for metric in metrics:
            if type(metric['value']) is not float:
                if metric['value'] in ['true', 'True']:
                    metric['value'] = 1.0
                elif metric['value'] in ['false', 'False']:
                    metric['value'] = 0.0
                try:
                    metric['value'] = float(metric['value'])
                except ValueError as e:
                    logger.warning("parameter: {}, subsystem: {}".format(
                            metric['parameter'],
                            metric['subsystem']
                        )+" has invalid string value " +
                        ": {} : Converting to 0".format(metric['value']))
                    metric['value'] = 0
        await self.major_tom.transmit_metrics([
            {
                "system": self.system_name,
                "subsystem": metric['subsystem'],
                "metric": metric['parameter'],

                "value": metric['value'],

                # Timestamp from KubOS is expected to be fractional seconds since unix epoch.
                # Convert to milliseconds for Major Tom
                "timestamp": int(metric['timestamp'] * 1000)
            } for metric in metrics
        ])

    async def start_services(self):
        await asyncio.gather(*[service.connect() for service in self.registry])

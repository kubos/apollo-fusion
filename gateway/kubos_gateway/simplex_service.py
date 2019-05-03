# Copyright 2019 Kubos Corporation
# Licensed under the Apache License, Version 2.0
# See LICENSE file for details.

"""
Module for continuously fetching, parsing, and sending Simplex data
"""

import asyncio
import logging
from datetime import datetime
from kubos_gateway.nsl_simplex_webapi import NSLWeb
import time
from kubos_gateway.satellite import Satellite

LOGGER = logging.getLogger(__name__)

class SimplexService():
    """ NSL Simplex Interface"""
    def __init__(self, satellite):
        self.satellite = satellite

    async def get_message(self):
        """ Get new simplex records and forward them on to MT """

        nsl = NSLWeb()
        records = nsl.simplex_download()

        # do simplex web api call
        print("Records: {}".format(records))

        metrics = [{
            'subsystem': 'test-sys',
            'parameter': 'test-param',
            'value': 'test-val',
            'timestamp': 1531412196211.0
            }]

        # TODO: Not everything can be respresented by a float, which is required for MT telemetry
        # values
        await self.satellite.send_metrics_to_mt(metrics)

    async def start_listener(self):
        """ Listen for new simplex records and forward them on to MT """
        # Log in to NSL web API

        # Set up a listener loop
        while True:
            await self.get_message()

            await asyncio.sleep(20)

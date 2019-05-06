# Copyright 2019 Kubos Corporation
# Licensed under the Apache License, Version 2.0
# See LICENSE file for details.

"""
High-level gateway module

This is what ties everything together
"""

import asyncio
import logging

from kubos_gateway.major_tom import MajorTom
from kubos_gateway.satellite import Satellite
from kubos_gateway.simplex_service import SimplexService

class Gateway(object):
    @staticmethod
    def run_forever(config):
        logging.info("Starting up!")
        loop = asyncio.get_event_loop()

        # Setup MajorTom
        major_tom = MajorTom(config)

        # Setup Satellite
        satellite = Satellite(
            host=config['sat-ip'],
            major_tom=major_tom,
            system_name=config['system-name'])
        major_tom.satellite = satellite

        # Connect to Major Tom
        asyncio.ensure_future(major_tom.connect_with_retries())

        # Start simplex listener
        simplex = SimplexService(satellite)
        asyncio.ensure_future(simplex.start_listener())

        loop.run_forever()
        loop.close()

    @staticmethod
    def set_log_level(log_level=logging.INFO, very_verbose=False):
        logging.basicConfig(
            level=log_level,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')

# Copyright 2019 Kubos Corporation
# Licensed under the Apache License, Version 2.0
# See LICENSE file for details.

"""
Module for continuously fetching, parsing, and sending Simplex data
"""

import asyncio
import binascii
import json
import logging
from kubos_gateway.nsl_simplex_webapi import NSLWeb
import struct
from kubos_gateway.satellite import Satellite

LOGGER = logging.getLogger(__name__)

# Parsing tables for the H&S beacons
TEMPERATURE = {"parsing": "20b",
                "names": ["eps_mb_temp","eps_db_temp","eps_bcr2a_temp","eps_bcr2b_temp","eps_bcr8a_temp","eps_bcr8b_temp","eps_bcr9a_temp","eps_bcr9b_temp","mai_gyro_temp","mai_motor_temp","bim_temp0","bim_temp1","bim_temp2","bim_temp3","bim_temp4","bim_temp5","bm2_temp","bm2_ts1_temp","bm2_ts2_temp","bm2_temp_range"]}
ADCS1 = {"parsing": "<LHHHBBBBf",
         "names": ["gps_time","good_cmd_count","bad_cmd_count","bad_checksum_count","last_command","acs_mode","attdet_mode","eclipse","angle_to_go"]}
ADCS2 = {"parsing": "<3f3h3h4h",
         "names": ["body_rate_x","body_rate_y","body_rate_z","wheel_speed_x","wheel_speed_y","wheel_speed_z","wheel_bias_x","wheel_bias_y","wheel_bias_z","qbo_0","qbo_1","qbo_2","qbo_3"]}
APP_ERRORS = {"parsing": "<HH8s22s",
           "names": ["app_errors_count", "app_last_timestamp", "app_last_source", "app_last_msg"]}
SERVICE_ERRORS = {"parsing": "<HH8s22s",
           "names": ["service_errors_count", "service_last_timestamp", "service_last_source", "service_last_msg"]}
GPS_POSITION = {"parsing": "<BH3d",
           "names": ["position_status","position_type","position_x","position_y","position_z"]}
GPS_VELOCITY = {"parsing": "<BH3d",
           "names": ["velocity_status","velocity_type","velocity_x","velocity_y","velocity_z"]}
GPS_MISC = {"parsing": "<BHLLHBfBHL",
           "names": ["time_status","time_week","time_ms","system_status","gps_status","power_status","power_3v_usb","power","lock_time_week","lock_time_ms"]}
OBC = {"parsing": "<BBB",
           "names": ["ram_available","disk_in_use","deployed"]}
GENERAL_POWER = {"parsing": "<HhH3B3BHHHHhhhhhh",
           "names": ["voltage","current","pf_status","mb_reset_bo","mb_reset_wdt","mb_reset_sw","db_reset_bo","db_reset_wdt","db_reset_sw","remaining_cap","full_cap","charge_voltage","charge_current","voltage_12v","current_12v","voltage_5v","current_5v","voltage_3v","current_3v"]}
BATTERY_MB_POWER = {"parsing": "<4H3h3h",
           "names": ["voltage_cell1","voltage_cell2","voltage_cell3","voltage_cell4","voltage_bcr1","current_bcr1a","current_bcr1b","voltage_bcr2","current_bcr2a","current_bcr2b"]}
DB_POWER = {"parsing": "<12h",
           "names": ["voltage_bcr6","current_bcr6a","current_bcr6b","voltage_bcr7","current_bcr7a","current_bcr7b","voltage_bcr8","current_bcr8a","current_bcr8b","voltage_bcr9","current_bcr9a","current_bcr9b"]}
RADIO = {"parsing": "x"}
SUPMCU = {"parsing": "<BHBHBHBHBHBH",
           "names": ["aim2_uptime","aim2_reset","bim_uptime","bim_reset","pim_uptime","pim_reset","sim_uptime","sim_reset","rhm_uptime","rhm_reset","bm2_uptime","bm2_reset"]}

# Temporary dummy data until we can actually send real data over the simplex
DUMMY_DATA = '''[
{"PayloadID":"747956","Payload":"213320B7FF00002A2B002A2B00A81BA800D0201000D22E0B009E135901FB0C7400","DT_NSLReceived":"2017-12-18 22:16:22"},
{"PayloadID":"747957","Payload":"223410001000000000F65359003F00122114000C00","DT_NSLReceived":"2017-12-18 22:20:44"},
{"PayloadID":"747958","Payload":"238000FEFFF8FF6300FEFFFFFFF0FFFAFFFEFF5400FFFFF8FF","DT_NSLReceived":"2017-12-18 22:21:03"},
{"PayloadID":"747959","Payload":"3811105757DEDEDFDF0B19E5E5E5E5E5E516272704","DT_NSLReceived":"2017-12-18 22:24:10"},
{"PayloadID":"748221","Payload":"092C00F766626561636F6E2D614661696C656420746F2073656E6420626561636F6E20","DT_NSLReceived":"2017-12-19 06:52:13"},
{"PayloadID":"748347","Payload":"0A5200845C6E6F766174656C2D726571756573745F6572726F72732028736572766963","DT_NSLReceived":"2017-12-19 13:19:39"},
{"PayloadID":"748348","Payload":"18590300","DT_NSLReceived":"2017-12-19 13:20:04"},
{"PayloadID":"775329","Payload":"30F18300FC830078300098300108300EE83000","DT_NSLReceived":"2018-02-02 17:48:58"},
{"PayloadID":"775330","Payload":"11010000000000000000000000000000000000000000000000000000","DT_NSLReceived":"2018-02-02 17:49:14"},
{"PayloadID":"775331","Payload":"12010000000000000000000000000000000000000000000000000000","DT_NSLReceived":"2018-02-02 17:49:30"},
{"PayloadID":"775332","Payload":"1314000000000000FF6FFD6100090B02440C3FFF000000000000","DT_NSLReceived":"2018-02-02 17:49:46"},
{"PayloadID":"775333","Payload":"0105CC7A1E0000000000000001000100000000","DT_NSLReceived":"2018-02-02 18:09:47"},
{"PayloadID":"775334","Payload":"02000000000000000000000000000000000000C800C8009CFF000000000000FF7F","DT_NSLReceived":"2018-02-02 18:10:27"}]
'''

class SimplexService():
    """ NSL Simplex Interface"""
    def __init__(self, satellite):
        self.satellite = satellite

    async def get_message(self):
        """ Get new simplex records and forward them on to MT """

        nsl = NSLWeb()
        #raw = nsl.simplex_download()
        raw = json.loads(DUMMY_DATA)

        records = []

        for entry in raw:
            records.append(process_record(entry))

        # Dummy data
        metrics = [{
            'subsystem': 'test-sys',
            'parameter': 'test-param',
            'value': 'test-val',
            'timestamp': 1531412196211.0
            }]

        # TODO: Not everything can be respresented by a float, which is required for MT telemetry
        # values
        # await self.satellite.send_metrics_to_mt(records)
        await self.satellite.send_metrics_to_mt(metrics)

    async def start_listener(self):
        """ Listen for new simplex records and forward them on to MT """
        # Log in to NSL web API

        # Set up a listener loop
        while True:
            await self.get_message()

            await asyncio.sleep(20)

def process_record(data):
    """
    Take a simplex record
    Get the `Payload` field
        - (Verify the first 3 bytes are 0x505050. Note: Not doing that atm. Might not actually need to.)
        - Read the next byte (it's the message header/type)
        - Based on the message type, parse the remaining bytes into the appropriate fields
    Maybe do something with the `PayloadID` field?
    Return a list of metrics? that can be fed into MT
        - Subsystem can be determined by the message header
        - Timestamp should be derived from the `DT_NSLReceived` field
    """
    payload = data['Payload']

    header = int(payload[0:2], 16)

    packet = binascii.unhexlify(payload[2:])

    (subsystem, input_dict) = {
        0x01: ("MAI-400", ADCS1),
        0x02: ("MAI-400", ADCS2),
        0x09: ("Errors", APP_ERRORS),
        0x0A: ("Errors", SERVICE_ERRORS),
        0x11: ("OEM7", GPS_POSITION),
        0x12: ("OEM7", GPS_VELOCITY),
        0x13: ("OEM7", GPS_MISC),
        0x18: ("OBC", OBC),
        0x21: ("Power", GENERAL_POWER),
        0x22: ("Power", BATTERY_MB_POWER),
        0x23: ("Power", DB_POWER),
        0x28: ("Duplex", RADIO),
        0x30: ("SupMCU", SUPMCU),
        0x38: ("Temperature", TEMPERATURE)
        }.get(header)
        # TODO: What happens if there's a bad/unknown packet?

    # Convert the record into a set of key/value pairs
    output_dict = read_telemetry_items(input_dict, packet)
    
    LOGGER.debug("Subsystem: {}, Data: {}".format(subsystem, output_dict))
    
    # TODO: Transform into MT-ready structure
    
    return "TODO"

def read_telemetry_items(input_dict, data):
    """
    Creates the output_dict, reads the data, inputs it into parsing mehods,
    then inserts and formats it in the output_dict.
    """
    # Create empty dictionary
    output_dict = {}

    # Parse the data
    parsed_data = unpack(
        parsing=input_dict['parsing'],
        data=data)
    output_dict.update(
        format_data(
            telem_field="temp",
            input_dict=input_dict,
            read_data=data,
            parsed_data=parsed_data))

    return output_dict

def format_data(telem_field, input_dict, read_data, parsed_data):
    """
    Takes in the read data, parsed data, and the input dictionary and outputs
    a formatted dictionary in the form of:
    {
        'fieldname': parsed_data,
        etc...
    }
    """
    output_dict = {}
    if "names" in input_dict:
        if len(parsed_data) == 1:
            raise KeyError(
                "Only one item parsed but subfields are listed: " +
                telem_field)
    if len(parsed_data) > 1:
        # Multiple items parsed
        if "names" not in input_dict:
            raise KeyError(
                "Must be a names field when multiple items are parsed: " +
                telem_field)
        if len(input_dict['names']) != len(parsed_data):
            raise KeyError(
                "Number of field names doesn't match parsing strings: " +
                telem_field)
        for ind, field in enumerate(input_dict['names']):
            output_dict.update(
                {field: parsed_data[ind]})
    
    else:
        # Single item parsed - pull in dict then update with parsed data.
        # Must be done in this order otherwise it generates a keyerror.
        output_dict[telem_field] = read_data
        output_dict[telem_field]['data'] = parsed_data[0]
    return output_dict

def unpack(parsing, data):
    """
    Basically just an abstraction of struct.unpack() to allow for types that
    are not standard in the method.

    Input data read over I2C from a Pumpkin module and parsing string that
    indicates a special parsing method or is a valid format string for the
    python struct.unpack() method.

    Outputs a tuple where each field is an item parsed.
    """
    if type(parsing) not in [str, bytes]:
        # Check that parsing is a valid type
        raise TypeError(
            'Parsing field must be a valid struct parsing string. Input: '
            + str(type(parsing)))
        
    if type(data) is str:
        data = data.encode()

    if parsing == "str":
        # Search for the null terminator,
        # return the leading string in a tuple
        str_data = data.split(b'\0')[0]
        return (str_data.decode(),)
    elif parsing == "hex":
        # Store as a hex string. This is so we can return binary data.
        # Return as a single field in a tuple
        return (binascii.hexlify(data).decode(),)

    # All others parse directly with the parsing string.
    return struct.unpack(parsing, data)
#!/usr/bin/env python

import argparse
import app_api
import sys
import time
import logging

SERVICES = app_api.Services()
WHEEL_SPEED_THRESHOLD = 5000 # RPM
WHEEL_SPEED_THRESHOLD_TIMEOUT = 20 # Seconds
ANGLE_TO_GO_THRESHOLD = 2 # degrees
ANGLE_TO_GO_THRESHOLD_TIMEOUT = 3*60 # 3 Minutes
RATE_THRESHOLD = 6 # deg/s
RATE_THRESHOLD_TIMEOUT = 10 # Seconds
ADCS_SETUP_TIMEOUT = 24*60*60 # 24 hours
HS_LOOP_TIME = 1*60 # 5 minutes
NOOP_RETRY = 3

def on_boot(logger):

    logger.info("OnBoot logic")

    # Launch ADCS Setup
    trigger_adcs_setup(logger=logger)

    start_time = time.time()
    previous_timestamp = 0
    while True:
        # try:
        # Check timestamp
        logger.debug("Checking for timestamp change")
        previous_timestamp = check_timestamp(logger=logger,previous_timestamp=previous_timestamp)

        # Check for reboot
        logger.debug("Checking for reboot")
        check_reboot(logger=logger)

        # Check for thresholds
        logger.debug("Checking wheel speeds")
        check_speed(logger=logger)

        logger.debug("Checking Angle To Go")
        check_angle(logger=logger)

        logger.debug("Checking Spin")
        check_spin(logger=logger)

        # except Exception as e:
        #     logger.error("Something went wrong: " + str(e) + "\r\n")

        if (start_time + ADCS_SETUP_TIMEOUT) < time.time():
            logger.info("launch ADCS setup: " + startApp)

        time.sleep(HS_LOOP_TIME)

def trigger_adcs_setup(logger):
    startApp = '''
    mutation {
        startApp(name: adcs-hs, runLevel: OnCommand): {
            success
            errors
            pid
        }
    }
    '''
    logger.info("launch ADCS setup: " + startApp)

def check_timestamp(logger, previous_timestamp):
    """
    Check that telemetry timestamp has increased
        If not, send no-op command
        If it responds, issue a warning log
        If it doesn’t, issue an error log and reboot
    """
    timestamp = query_tlmdb(logger=logger,tlm_key="gpsTime")['timestamp']
    if timestamp > previous_timestamp:
        previous_timestamp = timestamp
    else:
        logger.warning("ADCS tlm not updating, sending no-op")
        success = noop_cmd(logger)
        if success == False:
            reboot_mai(logger,reason="ADCS not responding to NOOP CMD")

    return previous_timestamp

def check_reboot(logger):
    """
    Check for reboot
        If it has: restart the ADCS setup logic from the deployment sequence
    """
    rebooted = True
    if rebooted:
        logger.warning("MAI rebooted, rerunning setup")
        trigger_adcs_setup(logger=logger)

def check_speed(logger):
    wheel_speed_x = "rwsSpeedTach_0"
    x_speed = query_tlmdb(logger=logger,tlm_key=wheel_speed_x)['value']
    wheel_speed_y = "rwsSpeedTach_1"
    y_speed = query_tlmdb(logger=logger,tlm_key=wheel_speed_y)['value']
    wheel_speed_z = "rwsSpeedTach_2"
    z_speed = query_tlmdb(logger=logger,tlm_key=wheel_speed_z)['value']
    speed = [x_speed,y_speed,z_speed]
    mai_wheel_speed_key = "rwsSpeedTach"

    counter = 0
    while any(val > WHEEL_SPEED_THRESHOLD for val in speed):
        time.sleep(1)
        counter+=1
        if counter >= WHEEL_SPEED_THRESHOLD_TIMEOUT:
            reboot_mai(logger=logger,reason="Wheel speed over {} for {} seconds. Speeds: {}".format(WHEEL_SPEED_THRESHOLD,WHEEL_SPEED_THRESHOLD_TIMEOUT,speed))
        speed = query_mai(logger=logger,tlm_key=mai_wheel_speed_key)

def check_angle(logger):
    angle_to_go_key = "angleToGo"
    angle = query_tlmdb(logger=logger,tlm_key=angle_to_go_key)['value']

    counter = 0
    while angle > ANGLE_TO_GO_THRESHOLD:
        time.sleep(1)
        counter+=1
        if counter >= ANGLE_TO_GO_THRESHOLD_TIMEOUT:
            reboot_mai(logger=logger,reason="Angle to Go over {} for {} seconds. Angle: {}".format(ANGLE_TO_GO_THRESHOLD,ANGLE_TO_GO_THRESHOLD_TIMEOUT,angle))
        angle = query_mai(logger=logger,tlm_key=angle_to_go_key)

def check_spin(logger):
    logger.debug("Checking Body Rate")
    x_key = "omegaB_0"
    y_key = "omegaB_1"
    z_key = "omegaB_2"
    x = query_tlmdb(logger=logger,tlm_key=x_key)['value']
    y = query_tlmdb(logger=logger,tlm_key=y_key)['value']
    z = query_tlmdb(logger=logger,tlm_key=z_key)['value']
    # rms_rate = np.sqrt(np.mean(np.square([x,y,z])))
    rms_rate = rms([x,y,z])
    mai_rate_key = "omegaB"

    counter = 0
    while rms_rate > RATE_THRESHOLD:
        time.sleep(1)
        counter+=1
        if counter >= RATE_THRESHOLD_TIMEOUT:
            reboot_mai(logger=logger,reason="rms of spin over {} for {} seconds. Rate: {}".format(RATE_THRESHOLD,RATE_THRESHOLD_TIMEOUT,rms))
        rate = query_mai(logger=logger,tlm_key=mai_rate_key)
        # rms_rate = np.sqrt(np.mean(np.square(rate)))
        rms_rate = rms(rate)

def noop_cmd(logger):
    logger.info("Sending NOOP cmd")
    try:
        for i in range(NOOP_RETRY):
            mutation = """mutation {
                noop {
                    errors: String,
                    success: Boolean
               }
            }"""
            result = SERVICES.query(service="mai400-service",query=mutation)
            if result['noop']['success']:
                return True
    except Exception:
        logger.error("MAI service not responding")
        return False

    return False

def reboot_mai(logger,reason):
    logger.error(reason)
    ## TODO: actually reset the mai ##

def query_tlmdb(logger,tlm_key,subsystem = "MAI400"):

    query = """{
        telemetry(subsystem: "%s", parameter: "%s", limit: 1) {
            value
            timestamp
        }
    }""" % (subsystem,tlm_key)
    logger.debug("Querying telemetry database for {}".format(tlm_key))
    result = SERVICES.query(service="telemetry-service",query=query)
    logger.debug(result)

    # telemetry db returns strings
    result['telemety']['value'] = float(result['telemety']['value'])

    return result['telemetry']

def query_mai(logger,tlm_key):

    query = """{
        telemetry{
            nominal{
                %s
            }
        }
    }""" % tlm_key
    logger.debug("Querying MAI400 for {}".format(tlm_key))
    logger.debug(query)
    result = SERVICES.query(service="mai400-service",query=query)
    logger.debug(result)

    return result['telemetry']['nominal'][tlm_key]

def on_command(logger):
    # Sets up ADCS
    logger.info("ADCS Setup Triggered.")

def rms(array):
    sum = 0
    for val in array:
        sum += val**2
    mean = sum/len(array)
    rms_val = mean**0.5
    return rms_val

# def logging_setup(app_name, level = logging.DEBUG):
#
#     import logging
#     """Set up the logger for the program
#     All log messages will be sent to rsyslog using the User facility.
#     Additionally, they will also be echoed to ``stdout``
#
#     Args:
#
#         - app_name (:obj:`str`): The application name which should be used for all log messages
#         - level (:obj:`logging.level`): The minimum logging level which should be recorded.
#           Default: `logging.DEBUG`
#
#     Returns:
#         An initialized Logger object
#     """
#     # Create a new logger
#     logger = logging.getLogger(app_name)
#     # We'll log everything of Debug level or higher
#     logger.setLevel(level)
#     # Set the log message template
#     formatter = logging.Formatter(app_name + ' %(message)s')
#
#     # Set up a handler for logging to stdout
#     stdout = logging.StreamHandler(stream=sys.stdout)
#     stdout.setFormatter(formatter)
#
#     # Finally, add our handlers to our logger
#     logger.addHandler(stdout)
#
#     return logger

def main():

    ### UNCOMMENT THIS WHEN ACTUALLY RUNNING ###
    logger = app_api.logging_setup("adcs-hs")
    # logger = logging_setup("adcs-hs")

    parser = argparse.ArgumentParser()

    parser.add_argument('--run', '-r')

    args = parser.parse_args()

    if args.run == 'OnBoot':
        on_boot(logger)
    elif args.run == 'OnCommand':
        on_command(logger)
    else:
        logger.error("Unknown run level specified")
        sys.exit(1)



if __name__ == "__main__":
    main()

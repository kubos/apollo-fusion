#!/usr/bin/env python

import argparse
import app_api
import sys
import time
import logging

SERVICES = app_api.Services()

## Housekeeping
WHEEL_SPEED_THRESHOLD = 5000 # RPM
WHEEL_SPEED_THRESHOLD_TIMEOUT = 20 # Seconds
ANGLE_TO_GO_THRESHOLD = 2 # degrees
ANGLE_TO_GO_THRESHOLD_TIMEOUT = 3*60 # 3 Minutes
RATE_THRESHOLD = 6 # deg/s
RATE_THRESHOLD_TIMEOUT = 10 # Seconds
ADCS_SETUP_TIMEOUT = 24*60*60 # 24 hours
HS_LOOP_TIME = 1*60 # 5 minutes
NOOP_RETRY = 3
REBOOT_COUNT = 0

## Setup
DETUMBLE_RATE_THRESHOLD = 0.5 # deg/s
DETUMBLE_RATE_THRESHOLD_TIMEOUT = 12*60 # Loops
DETUMBLE_RATE_LOOP_TIME = 60 # Seconds
POINTING_ANGLE_THRESHOLD = 1 # Degree
POINTING_ANGLE_TIMEOUT = 5*60 # 5 Minutes
GPS_LOCK_TIMEOUT = 12*60 # Loops (1 hour)
GPS_LOOP_TIME = 5 # Seconds
GPS_RETRIES = 10 # Tries


def on_boot(logger):

    logger.info("OnBoot logic")

    # Make sure ADCS is on
    power_on(logger=logger)

    # Launch ADCS Setup
    trigger_adcs_setup(logger=logger)

    time.sleep(5) # Sleep to wait for MAI to come online

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
            """
            Launch ADCS setup every 24 hours
            """
            trigger_adcs_setup(logger=logger)

        time.sleep(HS_LOOP_TIME)

def trigger_adcs_setup(logger):
    startApp = '''
    mutation {
        startApp(name: "adcs-hs", runLevel: OnCommand): {
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
        previous_timestamp = 0
        success = noop_cmd(logger)
        if success == False:
            reboot_mai(logger,reason="ADCS not responding to NOOP CMD")

    return previous_timestamp

def check_speed(logger):
    """
    Any of the 3 wheel speeds > WHEEL_SPEED_THRESHOLD
    for WHEEL_SPEED_THRESHOLD_TIMEOUT seconds
    """
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

def check_angle(logger,reboot = True, threshold=ANGLE_TO_GO_THRESHOLD,timeout=ANGLE_TO_GO_THRESHOLD_TIMEOUT):
    """
    Angle to Go > ANGLE_TO_GO_THRESHOLD degrees
    for ANGLE_TO_GO_THRESHOLD_TIMEOUT seconds
    """
    angle_to_go_key = "angleToGo"
    angle = query_tlmdb(logger=logger,tlm_key=angle_to_go_key)['value']

    counter = 0
    while angle > ANGLE_TO_GO_THRESHOLD:
        time.sleep(1)
        counter+=1
        if counter >= ANGLE_TO_GO_THRESHOLD_TIMEOUT:
            if reboot = True:
                reboot_mai(logger=logger,reason="Angle to Go over {} for {} seconds. Angle: {}".format(ANGLE_TO_GO_THRESHOLD,ANGLE_TO_GO_THRESHOLD_TIMEOUT,angle))
            return False
        angle = query_mai(logger=logger,tlm_key=angle_to_go_key)
    return True

def check_spin(logger,reboot = True,threshold = RATE_THRESHOLD,timeout = RATE_THRESHOLD_TIMEOUT,loop_time = 1):
    """
    RMS of Body Rate > RATE_THRESHOLD deg/s
    for RATE_THRESHOLD_TIMEOUT seconds
    """
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
    while rms_rate > threshold:
        time.sleep(loop_time)
        counter+=1
        if counter >= timeout:
            if reboot = True:
                reboot_mai(logger=logger,reason="rms of spin over {} for {} seconds. Rate: {}".format(RATE_THRESHOLD,RATE_THRESHOLD_TIMEOUT,rms))
            return False
        rate = query_mai(logger=logger,tlm_key=mai_rate_key)
        rms_rate = rms(rate)
    return True

def noop_cmd(logger):
    logger.info("Sending NOOP cmd")
    for i in range(NOOP_RETRY):
        mutation = """mutation {
            noop {
                errors: String,
                success: Boolean
           }
        }"""
        try:
            result = SERVICES.query(service="mai400-service",query=mutation)
            if result['noop']['success']:
                return True
        except Exception:
            logger.warning("MAI did not respond, retrying.")

    return False

def reboot_mai(logger,reason):
    REBOOT_COUNT += 1
    logger.error(f"Rebooting MAI400. Reboot Count: {REBOOT_COUNT} Reason: {reason} ")
    ## TODO: actually reboot the mai ##

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
    result['telemetry'][0]['value'] = float(result['telemetry'][0]['value'])

    return result['telemetry'][0]

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

def rms(array):
    # replacement for: np.sqrt(np.mean(np.square(array)))
    sum = 0
    for val in array:
        sum += val**2
    mean = sum/len(array)
    rms_val = mean**0.5
    return rms_val

def on_command(logger):
    # Sets up ADCS
    logger.info("ADCS Setup Started. Powering on GPS")
    power_gps(logger=logger)

    logger.info("Waiting for Detumble")
    wait_for_detumble(logger=logger)

    logger.info("Waiting for GPS lock")
    update_gps_info(logger=logger)

    logger.info("Going to Nadir pointing")
    set_attitude_determination_mode(logger=logger,mode=SUNMAG)
    set_attitude_control_mode(logger=logger,mode=NADIR)

    logger.info("Waiting for Angle to converge")
    wait_for_angle(logger=logger)
    set_attitude_determination_mode(logger=logger,mode=EHS)

    logger.info("ADCS Setup Complete")

    """
    On Board Setup (performed on every boot up):

    Turn on MAI
    Wait until detumbling is finished
    Check Body Rate every 1 minute
    If it’s < 0.5 deg/s, check every second for 30 seconds
    12 hours has passed
    (B dot can be used and can be verified with body rate, but body rate is less reliable for detumbling. Body rate should be roughly .1 degree)
    Turn on GPS
    Wait until GPS Lock is current (Position and velocity are finesteering, and time is less than 5 minutes old)
    Check every 1 minute
    If it waits over 1 hour:
    abort setup
    issue error
    Go to Safe Mode (acquisition)
    Feed BestXYZ GPS data directly into AODCS (Attitude and Orbit Determination and Control System)
    Sets GPS Time
    Sets RV
    Set Attitude determination mode to Mode 0: Sun/Mag Mode (CSS or sun/mag)
    Set ACS mode to Mode 3: Normal Mode (Nadir)
    Wait to get to Nadir/Quaternion/desired orientation
    Angle to Go is close to zero < 1 degree for 30 seconds
    Check every second
    Abort and go to acquisition mode if it doesn’t converge in 5 minutes
    Set Att Det Mode to Mode 2: EHS/Mag. (EHS - ONLY FOR NADIR MODE)

    """

def power_on(logger):
    logger.info("Powering on MAI400")
    pass

def wait_for_detumble(logger):
    detumbled = check_spin(logger=logger,
                           reboot=False,
                           threshold=DETUMBLE_RATE_THRESHOLD,
                           timeout=DETUMBLE_RATE_THRESHOLD_TIMEOUT,
                           loop_time=DETUMBLE_RATE_LOOP_TIME)
    if detumbled == False:
        reboot_mai(logger=logger,reason="Did not detumble. Rebooting into acquisition mode.")
        raise

def update_gps_info(logger):
    counter = 0
    while True:
        counter += 1
        power_gps(logger=logger)
        lock = wait_for_lock(logger=logger)
        if lock != False:
            success = submit_lock_data(logger=logger,lock=lock)
            if success:
                logger.info("GPS Time and Ephemeris successfully configured.")
                power_gps(logger=logger,state='OFF')
                return
        logger.warning('Updating Lock was unsuccessful. Retrying.')
        if counter > GPS_RETRIES:
            power_gps(logger=logger,state='OFF')
            raise


def power_gps(logger,state='ON'):
    logger.info(f'Power GPS: {state}')
    pass

def wait_for_lock(logger):
    # Wait until GPS Lock is current (Position and velocity are finesteering, and time is less than 5 minutes old)
    logger.info('Waiting for GPS Lock')
    time_status = None
    pos_status = None
    vel_status = None
    LOCKED = "FINESTEERING"
    counter = 0
    while counter >= GPS_LOCK_TIMEOUT:
        if time_status is not LOCKED:
            logger.debug(f'Waiting for time convergence: {time_status}')
        if pos_status is not LOCKED:
            logger.debug(f'Waiting for position convergence: {pos_status}')
        if vel_status is not LOCKED:
            logger.debug(f'Waiting for velocity convergence: {vel_status}')

        if all(status = LOCKED for status in [time_status,pos_status,vel_status]):
            logger.info("Lock Achieved!")
            lock = True
            return lock

        counter += 1
        time.sleep(GPS_LOOP_TIME)

    logger.error(f"GPS Timed out waiting for lock")

def submit_lock_data(logger,lock):
    # Feed BestXYZ GPS data directly into AODCS (Attitude and Orbit Determination and Control System)
    logger.debug("Setting System Time")
    # Set Processor Time
    logger.debug("Setting GPS Time on MAI")
    # Set GPS Time
    logger.debug("Setting Ephemeris on MAI")
    # Set GpsObservation2
    success = True
    if success:
        return True

    return False



def wait_for_angle(logger):
    angle = check_angle(logger=logger,
                        reboot=False,
                        threshold=POINTING_ANGLE_THRESHOLD,
                        timeout=POINTING_ANGLE_TIMEOUT)
    if angle == False:
        reboot_mai(logger=logger,reason="Angle did not converge. Rebooting into acquisition mode.")
        raise

def main():

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

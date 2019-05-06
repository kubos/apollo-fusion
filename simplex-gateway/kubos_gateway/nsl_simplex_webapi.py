#!/usr/bin/python3.6
#------------------------------------------------------------
# NearSpace Launch (NSL) Web API access demo
# Permission is granted to NSL's clients to use and modify this code
# as needed for NSL-related purposes.
# Aug 2017, modified Apr 2018 -sb
#------------------------------------------------------------
# Heavily modified by Kubos Corporation to fetch simplex data for later use

"""
NSL web interface module
"""

import requests            # See http://docs.python-requests.org/en/master/
import copy
import logging

USER_NAME = 'kubos'
PASSWORD = 'Denton2Space!'

# Apollo Fusion Simplex
#MISSION_ID = '949'
#SIMPLEX_ESN = '0-3216405'

# Other simplex that we have access to.
# Remove when AF simplex has transmitted data
MISSION_ID = '912'
SIMPLEX_ESN = '0-2316106'

BASE_URL = 'https://data2.nsldata.com/~gsdata/webAPIv1.0/'
LOGIN_URL = BASE_URL + 'login.php?UserName=' + USER_NAME + '&Password=' + PASSWORD
SIMPLEX_URL = BASE_URL + 'simplex.php?MissionID=' + MISSION_ID +'&ESN=' + SIMPLEX_ESN
LOGOUT_URL = BASE_URL + 'logout.php'

LOGGER = logging.getLogger(__name__)

class NSLWeb():
    """ NSL Web API """
    def simplex_download(self):
        """ Download all available simplex records """
        try:
            cookies = dict()

            #------------------
            # Do login
            #------------------
            r = requests.get(LOGIN_URL, cookies=cookies)
            response = r.json()
            if response['requestResult'] is False:
                raise Exception('Failed to login to NSL server: {}'.format(response))

            #PHPSESSID contains the critical session cookie
            #cookies = copy.deepcopy(r.cookies)
            cookies = r.cookies
            LOGGER.debug('Cookie = {}'.format(cookies['PHPSESSID']))

            #------------------
            # Download simplex data
            #------------------
            simplex_records = []

            r = requests.get(SIMPLEX_URL, cookies=cookies)
            simplex_records = r.json()
            if simplex_records['requestResult']:
                LOGGER.debug('Response = {}'.format(simplex_records['results']))
            else:
                raise Exception('Failed to fetch simplex records: {}'.format(simplex_records))

            #------------------
            # Do logout
            #------------------
            r = requests.get(LOGOUT_URL, cookies=cookies)
            response = r.json()
            if response['requestResult'] is False:
                raise Exception('Failed to logout of NSL server: {}'.format(response))

            return simplex_records["results"]

        except Exception as e:
            LOGGER.error('Error: {}'.format(e.args))

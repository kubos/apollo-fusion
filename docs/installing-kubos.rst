Installing KubOS onto the Apollo Fusion Stack
=============================================

This document covers the process to do a from-scratch installation of KubOS

1. :doc:`Build the OS images <building-kubos>`
2. Flash the SD card
3. Boot into the SD card and flash the eMMC
4. Flash the SD card with the Aux SD image
5. Log into the stack and CHANGE THE DATE
6. Install the Python, etc files

Build the OS
------------

Follow the instructions in :doc:`building-kubos` in order to build the OS images

Install the OS
--------------

Refer to the main `Kubos Linux installation <http://docs.kubos.co/latest/os-docs/kubos-linux-on-mbm2.html>`__
instructions and install the `kubos-linux.img` and `aux-sd.img` images.

Connect to the Stack
--------------------

Follow the :doc:`communicating-with-the-stack` instructions to get connected to the stack via the debug UART
connection.

Change the IP Address
---------------------

We recommend doing the remaining file transfers via the Ethernet connection, since it is capable of
greater transfer speeds.

Log into the stack and then edit ``/etc/network/interfaces``. Update the IP address field to be an
address of your choosing.

Once updated, run the following commands in order to make the board use the new address::
    
    $ ifdown eth0; ifup eth0
    
The address can be verified by running the ``ipaddr`` command

Change the System Date
----------------------

In order to install the remaining packages, the system date of the stack needs to be updated.
Connect to the stack and run the following command::

    $ date -s 2018-01-01
    
The output should look like this::

    $ date -s 2018-01-01
    Mon Jan  1 00:00:00 UTC 2018
    
If you would like, you may use today's date instead.

Install the Remaining KubOS Packages
------------------------------------

Run ``tools/package-install.sh`` in order to install the remaining required packages.
The script expects a single argument, which is the IP address of the stack.
For example::

    ./tools/package-install.sh 168.168.2.20

Installing KubOS onto the Apollo Fusion Stack
=============================================

This document covers the process to do a from-scratch installation of KubOS

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

Install the Remaining KubOS Packages
------------------------------------

(Install ``sshpass``... ``sudo apt-get install sshpass``. TODO: Add to vagrant?)

Run ``tools/package-install.sh`` in order to install the remaining required packages.
The script expects a single argument, which is the IP address of the stack.
For example::

    ./tools/package-install.sh 168.168.2.20

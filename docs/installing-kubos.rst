Installing KubOS onto the Apollo Fusion Stack
=============================================

This document covers the process to do a from-scratch installation of KubOS

Build the OS
------------

Follow the instructions in :doc:`building-kubos` in order to build the OS images

Install the OS
--------------

Refer to the main `Kubos Linux installation <http://docs.kubos.co/latest/installation-docs/installing-linux-mbm2.html>`__
instructions and install the `kubos-linux.img` and `aux-sd.img` images.

The short version:

    - Install `Etcher <https://etcher.io/>`__ onto your host machine
    - Flash an SD card with `kubos-linux.img`
    - Boot into Kubos Linux **on the SD card**
    - Unmount everything
    - Flash the eMMC with the contents of the SD card via the ``dd`` command
    - Shutdown the board and remove the SD card
    - Flash the SD card with `aux-sd.img`
    - Put SD back into board and boot normally

Connect to the Stack
--------------------

We recommend doing the remaining file transfers via the Ethernet connection, since it is capable of
greater transfer speeds. As a result, please follow the :ref:`ethernet` instructions to get connected to the stack.

Install the Remaining KubOS Packages
------------------------------------

`SSH into your Kubos SDK box <http://docs.kubos.co/latest/installation-docs/sdk-installing.html#start-the-vagrant-box>`__

Install ``sshpass`` in your SDK with ``sudo apt-get install sshpass``.

Verify that you can SSH into the stack.

Navigate to your copy of this repo and run ``tools/package-install.sh`` in order to install the
remaining required packages.

The script expects a single argument, which is the IP address of the stack.
For example::

    ./tools/package-install.sh 168.168.2.20
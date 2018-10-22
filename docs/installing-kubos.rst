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

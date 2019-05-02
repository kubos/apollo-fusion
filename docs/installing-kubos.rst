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
    - Run ``install-os`` to install the OS into the eMMC. Will take 20min or so
    - Shutdown the board and remove the SD card
    - Flash the SD card with `aux-sd.img`
    - Put SD back into board and boot normally

Build the Apps
--------------

Navigate to the `tools` directory in this repo.

Run ``bundle-apps.sh``. This will build all Rust-based apps and then bundle all apps in the `apps`
directory into a tar file, ``apps-{date}.tar.gz``.

Install the Apps
----------------

Transfer the ``apps-{date}.tar.gz`` file into the ``/home/kubos`` directory on the OBC.

For example::

    $ scp apps-2019.05.02.tar.gz kubos@192.168.2.20:/home/kubos
    
Log into the OBC, untar the file, and then run the ``install-apps.sh`` script::

    $ ssh kubos@192.168.2.20
    kubos@168.168.2.20's password: Kubos123
    /home/kubos # tar -xzf apps-2019.05.02.tar.gz
    /home/kubos # ./install-apps.sh
    
    

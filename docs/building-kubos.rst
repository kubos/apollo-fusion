Building KubOS for Apollo Fusion
================================

Overview
--------

The goal of this document is to create the KubOS images required for the Apollo Fusion stack.

After building the OS files, refer to the :doc:`installation page <installing-kubos>` for installation instructions.

Reference Documents
-------------------

Pumpkin Documentation
~~~~~~~~~~~~~~~~~~~~~

The :title:`CubeSat Kit Motherboard Module (MBM) 2` reference document
is available from Pumpkin and is a useful document for learning what 
each of the hardware components are and how they are connected.

Kubos Documentation
~~~~~~~~~~~~~~~~~~~

-  :doc:`installing-kubos` - Steps to install KubOS
-  :doc:`communicating-with-the-stack` - General guide for interacting with KubOS

Kubos Linux Build Process
-------------------------

.. warning::

    The OS files cannot be built using a `synced folder <https://www.vagrantup.com/docs/synced-folders/>`__ in a Vagrant box (or regular VM).
    VirtualBox does not support hard links in shared folders, which are crucial in order to complete
    the build.
    
`Build and SSH into a Kubos SDK box <http://docs.kubos.co/latest/installation-docs/sdk-installing.html>`__
    
In order to build KubOS, three components are needed:

- The `kubos-linux-build repo <https://github.com/kubos/kubos-linux-build>`__ - This is the main Kubos repo used to build Kubos Linux
- The `apollo-fusion repo <https://github.com/kubos/apollo-fusion>`__ - This repo contains the proprietary components of KubOS for Apollo Fusion
- `BuildRoot <https://buildroot.org/>`__ - The actual build system

These components should be setup as children of the same parent directory. 
There are several commands and variables in the build process which use relative file paths to navigate between the components.

After the environment has been set up, all build commands will be run from the BuildRoot directory unless otherwise stated.

To set up a build environment and build Kubos Linux:

Create a new parent folder to contain the build environment

::

    $ mkdir kubos-linux

Enter the new folder

::

    $ cd kubos-linux

Download BuildRoot-2017.02 (more current versions of BuildRoot may work as well,
but all testing has been done against 2017.02)

.. note:: All Kubos documentation will refer to v2017.02.8, which is the latest version of the LTS release at the time of this writing.

::

    $ wget https://buildroot.org/downloads/buildroot-2017.02.8.tar.gz && tar xzf buildroot-2017.02.8.tar.gz && rm buildroot-2017.02.8.tar.gz

Pull the kubos-linux-build repo

::

    $ git clone http://github.com/kubos/kubos-linux-build
    
Pull the apollo-fusion repo (this is a private repo, so you will be required to log in in order to successfully clone it)

::

    $ git clone http://github.com/kubos/apollo-fusion

Move into the buildroot directory

::

    $ cd buildroot-2017.02.8

Point BuildRoot to the external folders and tell it to build using the proprietary iOBC
configuration

.. note::

    You will need to build with ``sudo``, since the cross-compile toolchain resides in
    a protected directory

::

    $ sudo make BR2_EXTERNAL=../kubos-linux-build:../apollo-fusion apollo-fusion_defconfig

Build everything

::

    $ sudo make

The full build process will take a while. Running on a Linux VM, it took about
an hour. Running in native Linux, it took about ten minutes. Once this build
process has completed once, you can run other BuildRoot commands to rebuild
only certain sections and it will go much more quickly (<5 min).

BuildRoot documentation can be found
`**here** <https://buildroot.org/docs.html>`__

The generated image will be located in ``buildroot-2017.02.8/output/images/kubos-linux.img``.
TODO: Move the image somewhere useful for flashing

Create auxilliary SD Card Image
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

TODO: Automate creating the Aux SD image

By default, the build process will create a bootable SD card image. This will be flashed
onto the eMMC. In order to create a full Kubos Linux setup, you'll want to also create
an auxiliary image for the microSD card containing the upgrade partition and an additional
user data partition.

TODO: copy the instructions here
Follow the :ref:`upgrade-creation` instructions in order to create a base Kubos Package file
(`kpack-base.itb`) to be used for recovery.

Then, from the `kubos-linux-build/tools` folder, run the ``format-aux.sh`` script. 
This will create a new SD card image, `aux-sd.img`, with two partitions:

- An upgrade partition containing `kpack-base.itb`
- A user data partition

The image's disk signature will be 0x41555820 ("AUX ").

There are two parameters which may be specified:

-  -s : Sets the size of the aux-sd.img file, specified in MB. The default is 3800 (3.8GB)
-  -i : Specifies the name and location of the kpack-\*.itb file to use as kpack-base.itb

For example:

::

    $ ./format-aux.sh -i ../kpack-2017.07.21.itb


Reset the Global Links
~~~~~~~~~~~~~~~~~~~~~~

If you run a full build, the links to all the Kubos SDK modules will be changed to
point at modules within the buildroot directory. As a result, you will be unable
to build any future Kubos SDK projects as a non-privileged user.

To fix this, run these commands:

::

    $ cd $HOME/.kubos/kubos/tools
    $ ./kubos_link.py
    

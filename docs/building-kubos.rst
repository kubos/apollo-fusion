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

Move into the ``apollo-fusion/tools`` directory and run the ``build-os.sh`` script

::

    $ cd apollo-fusion/tools
    $ ./build-os.sh

The full build process will take a while. Running on a Linux VM, it took about
an hour. Running in native Linux, it took about ten minutes. Once this build
process has completed once, you can run other BuildRoot commands to rebuild
only certain sections and it will go much more quickly (<5 min).

TODO: final image output directory

Reset the Global Links
~~~~~~~~~~~~~~~~~~~~~~

If you run a full build, the links to all the Kubos SDK modules will be changed to
point at modules within the buildroot directory. As a result, you will be unable
to build any future Kubos SDK projects as a non-privileged user.

To fix this, run these commands:

::

    $ cd $HOME/.kubos/kubos/tools
    $ ./kubos_link.py
    

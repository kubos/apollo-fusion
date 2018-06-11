Communicating with the Apollo Fusion Stack
==========================================

This doc covers how to setup communication with the Apollo Fusion stack and then how to manually communicate with the services running on it.

Communicating via Serial Debug
------------------------------

The serial debug port is the default available communication port on the stack.

Setup
~~~~~

Connect a Pumpkin USB debug adapter to the UART0 port on the Pumpkin MBM2 board and then connect the USB to your computer. 

.. todo: Get photo of UART0 port on MBM2 board

From the SDK
~~~~~~~~~~~~

From an instance of the `Kubos SDK <http://docs.kubos.co/latest/installation-docs/sdk-installing.html>`__,
run ``minicom kubos``. This will automatically create a session with the debug UART.

You can then log in. The default user account and password is Kubos/Kubos123.

Fully logged in, the console should look like this:

.. code-block:: console

    Welcome to Kubos Linux
    Kubos login: kubos
    Password:
    /home/kubos #
        
From a Host Machine
~~~~~~~~~~~~~~~~~~~

.. warning: All instances of the SDK must be shutdown in order to connect to the serial port directly from a host machine

Identify which COM port the USB is being presented as and then create a serial connection with your tool of choice
with the following settings:

+-----------+--------+
| Setting   | Value  |
+===========+========+
| Baudrate  | 115200 |
+-----------+--------+
| Bits      | 8      |
+-----------+--------+
| Parity    | N      |
+-----------+--------+
| Stop Bits | 1      |
+-----------+--------+
    
You can then log in. The default user account and password is Kubos/Kubos123.

Fully logged in, the console should look like this:

.. code-block:: console

    Welcome to Kubos Linux
    Kubos login: kubos
    Password:
    /home/kubos #

.. _ethernet:

Communicating via Ethernet
--------------------------

Setup
~~~~~

Connect an ethernet cable from the stack to either your computer or an open network port.

Log into the stack and then edit ``/etc/network/interfaces``. Update the IP address field to be an
address of your choosing.

Once updated, run the following commands in order to make the board use the new address::
    
    $ ifdown eth0; ifup eth0
    
The address can be verified by running the ``ipaddr`` command

SSH
~~~

Once the stack has been given a valid IP address, you can create an SSH connection to it.

This can be done from either the SDK or your host machine.

To connect from the command line, run ``ssh kubos@{ip-address}``.
You will be prompted for the `kubos` account password.

You can also use a tool, like PuTTY, to create an SSH connection.
    
File Transfer
~~~~~~~~~~~~~

Once the IP address has been set, you can also transfer files to and from the stack using the ``scp`` command.
Again, this command can be run from either the SDK or your host machine.

For example, if I wanted to send a file on my host machine, `test.txt`, to reside in the `kubos` account's home directory,
given a stack IP of ``10.50.1.10``, I would enter::

    $ scp test.txt kubos@10.50.1.10:/home/kubos

Communicating with Services
---------------------------

The Apollo Fusion stack has a collection of Kubos services which are automatically started at boot time and are used
to control the system. Ordinarily, all communication with these services will be taken care of by the mission application.

If desired, you may manually communicate with any of the services to collect data or run small operations.

Configuration
~~~~~~~~~~~~~

The stack's `config.toml <https://github.com/kubos/apollo-fusion/blob/master/common/overlay/home/system/etc/config.toml>`__ file
contains the configuration information about all of the services. The file is located on the stack in ``/home/system/etc/config.toml``.

Using this file, you can look up which services are available on the system and which UDP port is used to communicate with each of them.

GraphQL
~~~~~~~

All of the services communicate by receiving `GraphQL <https://graphql.org/learn/>`__ requests and returning JSON responses.

GraphQL requests are broken into two types: queries and mutations. Queries are requests for data, while mutations request that commands
be run against the underlying system which the service controls.

A basic query for the power status might look like this::

    {
        power {
            state,
            uptime
        }
    }
    
And the service might give a response like this::

    {
        "power": {
            "state": "ON",
            "uptime": 1000
        }
    }
        
Alternatively, a basic mutation request might look like this (note the prefixing "mutation")::

    mutation {
        power(state: RESET) {
            success,
            errors,
            state
        }
    }

And return the following response::

    {
        "power": {
            "success": True,
            "errors": "",
            "state": "RESET"
        }
    }
    
Available Requests
~~~~~~~~~~~~~~~~~~

The GraphQL requests for each of the services can be found in the main starting file of each of the service's source directories in the
`kubos repo <https://github.com/kubos/kubos/tree/master/services>`__.

For example, the requests for the telemetry service can be found in `services/telemetry-service/src/main.rs` in the top comments section.

Sending a Request
~~~~~~~~~~~~~~~~~

There are several ways to send a request to a Kubos service:

    - From the command line directly on the stack
    - From the command line in your host machine or SDK
    - By running the UDP client program

Native Command Line
^^^^^^^^^^^^^^^^^^^

If you connect to the stack, you can use the ``nc`` command to send requests directly to services.

For example:

.. code-block:: console

    /home/kubos # echo "{ping}" | nc -uw1 0.0.0.0 8002
    {"errs":"","msg":{"ping":"pong"}}
    /home/kubos #
    
In this case, the IP address of the stack doesn't need to be known (or set up); only the port of the service needs to be specified.

Remote Command Line
^^^^^^^^^^^^^^^^^^^

If not directly connected to the stack, the ``nc`` command can still be used to send requests to services, 
but the IP address must also be known:

.. code-block:: console

    vagrant@vagrant:~$ echo "mutation { noop { success, errors}}" | nc -uw1 168.168.2.20 8003
    {"errs":"","msg":{"noop":{"errors":"Failed to get command response","success":false}}}
    vagrant@vagrant:~$

In this example, we requested that the service run the no-op command against its underlying hardware and to report its success status and
any failures. The underlying hardware was turned off, so the request failed and we were informed that we were unable to get a response from
the device. 

UDP Client Program
^^^^^^^^^^^^^^^^^^

The UDP client program included in the `main Kubos repo <https://github.com/kubos/kubos/tree/master/examples/udp-service-client>`__
can be used when sending larger queries or mutations.

To use the program:

- Connect to an instance of the SDK
- Run ``cd ~/.kubos/kubos/examples/udp-service-client``
- Edit the ``config.toml`` file in the folder to specify your host machine's IP address and port to use (the port simply must be unused),
  and then the stack's IP address and the port of the service you are attempting to communicate with
- Edit the ``query.txt`` file to contain your desired query or mutation request. *Note: Only one query or mutation request may be specified in
  the file, though multiple operations may be specified within that.*
  
  For example::
  
    {
        errors,
        power {
            state,
            uptime
        },
        mode
    }
    
- Run the program::

    $ cargo run config.toml
    
- This will build the test program and then execute the GraphQL request. The returned JSON will be output:

.. code-block:: console

    Bound socket to 168.168.2.10:8080
    Sent 73 bytes
    Received 85 bytes from 168.168.2.20:8002
    {
      "errs": "",
      "msg": {
        "errors": [],
        "mode": "TEST_MODE",
        "power": {
          "state": "ON",
          "uptime": 349
        }
      }
    }
    
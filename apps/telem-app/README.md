# Mission Application for Gathering Telemetry

This project loops through each of the subsystems present and gathers all available
telemetry.
This telemetry is then sent to the telemetry service to be stored via the direct UDP
port.

After each round of gathering, the app sleeps for one minute.
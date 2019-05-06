# KubOS Python Gateway for Major Tom

*Note:* The gateway is currently in Beta, and is not well documented.


# First time setup

1. Setup a virtualenv
```bash
pip3 install virtualenv
virtualenv virtualenv -p `which python3`
source virtualenv/bin/activate
pip3 install --upgrade -r requirements.txt
```

Every time:

```bash
source virtualenv/bin/activate
pip3 install --upgrade -r requirements.txt
python run.py
```

See `python run.py --help`

Deactivate virtualenv:

```bash
deactivate
```

# Usage

Specifying a config file to use:

```bash
python run.py -c ../configs/duplex-gateway.json
```

Specifying a mode:

```bash
python run.py -c ../configs/duplex-gateway.json gateway
python run.py -c ../configs/duplex-gateway.json stream_tlm
```

Available modes:

* gateway
* stream\_tlm

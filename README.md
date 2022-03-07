# Overview

`dkms-resolver` provides a discovery mechanism for DKMS Identifiers (AID's), where other participants are able to discover Identifiers KEL's. It is based upon Kademlia DHT concept with the aim to be the ambient discovery infrastructure.

# Usage

## Options

```txt
USAGE:
    keri_resolver [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --api-port <api-port>                        API listen port [env: API_PORT=]  [default: 9599]
        --api-public-host <api-public-host>
            API public host name. Announced in DHT together with API port [env: API_PUBLIC_HOST=]  [default: localhost]

        --dht-bootstrap-addr <dht-bootstrap-addr>    DHT bootstrap IP address [env: DHT_BOOTSTRAP_ADDR=]
        --dht-port <dht-port>                        DHT listen port [env: DHT_PORT=]  [default: 9145]

```

## API

- Get issuer's Key Event State (called by TDA)

  ```http
  GET /key_states/{issuer_id} HTTP/1.1
  ```

- Get issuer's Key Event Log (called by TDA)

  ```http
  GET /key_logs/{issuer_id} HTTP/1.1
  ```

- Create key event (called by witness)

  ```http
  POST /messages/{issuer_id} HTTP/1.1
  Content-Type: application/octet-stream

  // Signed event data
  {
    "v": "KERI10JSON000120_",
    "t": "icp",
    "d": "Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8",
    "i": "Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8",
    "s": "0",
    "kt":"1",
    "k": ["DqI2cOZ06RwGNwCovYUWExmdKU983IasmUKMmZflvWdQ"],
    "n": "E7FuL3Z_KBgt_QAwuZi1lUFNC69wvyHSxnMFUsKjZHss",
    "bt": "0",
    "b": [],
    "c": [],
    "a": []
  }-AABAAJEloPu7b4z8v1455StEJ1b7dMIz-P0tKJ_GBBCxQA8JEg0gm8qbS4TWGiHikLoZ2GtLA58l9dzIa2x_otJhoDA
  ```

- Get witness' IP address (called by TDA)

  ```http
  GET /witness_ips/{witness_id} HTTP/1.1
  ```

- Set witness' IP address (called by witness)

  ```http
  PUT /witness_ips/{witness_id} HTTP/1.1
  Content-Type: application/json

  {
      "ip": "127.0.0.1:1234"
  }
  ```

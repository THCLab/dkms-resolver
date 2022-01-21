# `keri_resolver`

## Options

- `--api-port <api-port>` - API listen port [default: 9599]
- `--dht-port <dht-port>` - DHT listen port [default: 9145]
- `--bootstrap-addr <bootstrap-addr>` - DHT bootstrap IP address

## API

- Get issuer's key state (called by TDA)

  ```http
  GET /key_states/{issuer_id} HTTP/1.1
  ```

- Set issuer's key state (called by witness)

  ```http
  PUT /key_states/{issuer_id} HTTP/1.1
  Content-Type: application/json

  {
      // Key state JSON
  }
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

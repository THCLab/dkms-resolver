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

- Get issuer's key state (called by TDA)

  ```http
  GET /key_states/{issuer_id} HTTP/1.1
  ```

- Set issuer's key state (called by witness)

  ```http
  PUT /key_states/{issuer_id} HTTP/1.1
  Content-Type: text/plain

  // signed message
  {
    "v":"KERI10JSON000292_",
    "t":"rpy",
    "d":"E_v_Syz2Bhh1WCKx9GBSpU4g9FqqxtSNPI_M2KgMC1yI",
    "dt":"2021-01-01T00:00:00.000000+00:00",
    "r":"/ksn/Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8",
    "a":{
      "v":"KERI10JSON0001d7_",
      "i":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8",
      "s":"3",
      "p":"EYhzp9WCvSNFT2dVryQpVFiTzuWGbFNhVHNKCqAqBI8A",
      "d":"EsL4LnyvTGBqdYC_Ute3ag4XYbu8PdCj70un885pMYpA",
      "f":"3",
      "dt":"2021-01-01T00:00:00.000000+00:00",
      "et":"rot",
      "kt":"1",
      "k":["DrcAz_gmDTuWIHn_mOQDeSK_aJIRiw5IMzPD7igzEDb0"],
      "n":"E_Y2NMHE0nqrTQLe57VPcM0razmxdxRVbljRCSetdjjI",
      "bt":"0",
      "b":[],
      "c":[],
      "ee":{"s":"3","d":"EsL4LnyvTGBqdYC_Ute3ag4XYbu8PdCj70un885pMYpA","br":[],"ba":[]}
    }
  }-FABEt78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV80AAAAAAAAAAAAAAAAAAAAAAwEsL4LnyvTGBqdYC_Ute3ag4XYbu8PdCj70un885pMYpA-AABAAycUrU33S2856nVTuKNbxmGzDwkR9XYY5cXGnpyz4NZsrvt8AdOxfQfYcRCr_URFU9UrEsLFIFJEPoiUEuTbcCg
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

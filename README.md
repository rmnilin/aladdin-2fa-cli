# Aladdin 2FA CLI

Unofficial CLI tool similar to [Aladdin 2FA Desktop](https://www.aladdin-rd.ru/catalog/aladdin-2fa/#download) to:
- Register JaCarta Authentication Server tickets
- Save tickets to JSON files
- Retrieve authentication requests based on push tokens from registered ticket JSON files
- Accept authentication requests

## Docker Usage Tips

```shell
$ touch ticket.json

$ docker run \
    --volume ./ca.crt:/ca.crt \
    --volume ./ticket.json:/ticket.json \
    ghcr.io/rmnilin/aladdin-2fa-cli \
        --ca-cert-path ca.crt \
        --verbose \
        register \
            --file-path ticket.json \
            'jasticket://7921ab1cfe67bd9c42116c7e379ba821?address=https://mtcloud-a2fa.a-rd.ru&key=RL7RKKH3JI5ZVS5HF6RYVMIKQ4&pinLen=0'

2025-04-11T05:20:19.386648Z INFO Ticket registered and saved to ticket.json

$ docker run \
    --volume ./ca.crt:/ca.crt \
    --volume ./ticket.json:/ticket.json \
    ghcr.io/rmnilin/aladdin-2fa-cli \
        --ca-cert-path ca.crt \
        --verbose \
        accept \
            --file-path ticket.json \
            --daemon

2025-04-11T05:21:39.924450Z DEBUG No authentication requests received
2025-04-11T05:21:40.976704Z DEBUG No authentication requests received
2025-04-11T05:21:41.578688Z DEBUG Authentication request with session ID ae8fe8fc80a77af1 received
2025-04-11T05:21:41.593017Z INFO Authentication request with session ID ae8fe8fc80a77af1 accepted
2025-04-11T05:21:42.641137Z DEBUG No authentication requests received
```

### Docker Compose

```yaml
services:
  aladdin-2fa-cli:
    image: ghcr.io/rmnilin/aladdin-2fa-cli
    volumes:
      - ./ca.crt:/ca.crt
      - ./ticket.json:/ticket.json
    command:
      - --ca-cert-path
      - ca.crt
      - --verbose
      - accept
      - --file-path
      - ticket.json
      - --daemon
```

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
    --volume ./ca.crt:/ca.crt:ro \
    --volume ./ticket.json:/ticket.json \
    ghcr.io/rmnilin/aladdin-2fa-cli \
        --ca-cert-path ca.crt \
        register \
            --file-path ticket.json \
            'jasticket://7921ab1cfe67bd9c42116c7e379ba821?address=https://mtcloud-a2fa.a-rd.ru&key=RL7RKKH3JI5ZVS5HF6RYVMIKQ4&pinLen=0'

Ticket registered successfully and saved to ticket.json

$ docker run \
    --volume ./ca.crt:/ca.crt:ro \
    --volume ./ticket.json:/ticket.json \
    ghcr.io/rmnilin/aladdin-2fa-cli \
        --ca-cert-path ca.crt \
        accept \
            --file-path ticket.json \
            --daemon \
            -v

Accepting all incoming authentication requests
No requests
No requests
Accepted request 758c91635c1f9e71
No requests
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
      - accept
      - --file-path
      - ticket.json
      - --daemon
      - -v
```

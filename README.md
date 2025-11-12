# wusp

tcp over websocket proxy written in rust

## usage

> **NOTE**: currently wusp only supports one user and one proxy target, i made this to proxy ssh requests to my
> forgejo server so i dont really plan on adding more, but i might
> also i made this to learn rust so this is probably trash.
> if you want better functionality consider using [jpillora/chisel](https://github.com/jpillora/chisel)

### running the server

server is really simple, authentication is passed via `Authorization` header and websocket connection is established.
then any further websocket messaging is relayed to tcp and vice versa.

a new tcp connection is opened for every successful websocket connection.

```sh
wusp server <address> <target> --auth <auth>
```

example:

```sh
# here wusp listens on port 8080 (on http) and proxies requests to ssh on 22, and only allows authorized clients
wusp server 127.0.0.1:8080 127.0.0.1:22 --auth super_secure_password
```

not passing the auth string will allow any client to connect.

### running the client

the client is also really simple and binds stdin/stdout to remote tcp server.

```sh
wusp client <host> --auth <auth>
```

example:

```sh
wusp client ws://127.0.0.1:8080 --auth super_secure_password
```

---

for client, host can be passed via `WUSP_HOST`.
for server, binding address can be passed via `WUSP_ADDRESS`, target can be passed via `WUSP_TARGET`.
for both commands `auth` is optional and can also be passed via `WUSP_AUTH` env var.

```sh
WUSP_AUTH=super_secure_password wusp client ws://127.0.0.1:8080
```

im using clap so i believe cli arg would take precedence over env var

## examples

1. **connecting to an ssh server over https**

   for me its cloudflare tunnels or other tunneling provider (cloudflare supports ssh but you need the big bad cloudflare client so its meh)

   ```sh
   ssh -o ProxyCommand="wusp client wss://myserver.com --auth super_secure_password" myserver.com
   ```

   or you can have it in config (~/.ssh/config)

   ```plaintext
   Host myserver.com
       ProxyCommand wusp client wss://myserver.com --auth super_secure_password
   ```

   now you can simply do

   ```sh
   ssh myserver.com
   ```

## todo

- [ ] option to bind client to a tcp port rather than stdin/stdout
- [ ] multiple targets/users

## safety

i dont know im new to rust but i made it pretty simple so read the [source code](./src/main.rs)

## contribution

go wild with it, keep it simple tho. not that anyone will ever contribute :(

## license

under [MIT](./LICENSE) because im lazy to read through the others

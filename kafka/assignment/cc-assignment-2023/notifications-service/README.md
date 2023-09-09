# Execution

```bash
cargo run -p notifications-service -- --secret-key <SECRET_KEY> --external-ip <EXTERNAL_IP>
```

- `<SECRET_KEY>`: 32 character string that must match the key being passed to
  the notifications-service, e.g., "QJUHsPhnA0eiqHuJqsPgzhDozYO4f1zh".
- `<EXTERNAL_IP>`: is the IP or DNS you will use in the browser to connect to
  the api documentation.

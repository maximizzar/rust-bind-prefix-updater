# rust-bind-prefix-updater
bind9 prefix updater (ddns for ipv6)

It updates bind AAAA dns records. It just needs a config like this.

```json
{
    "hosts": [
        "hosta",
        "hostb",
        "hostc"
    ],
    "prefix_size": 64,
    "record_db_path": "/path/to/db/file"
}
```

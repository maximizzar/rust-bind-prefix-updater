# rust-bind-prefix-updater
bind9 prefix updater (ddns for ipv6)

Provide via cli args a hostname and a bind9 zones file (contains your domains records). The tool then checks if the Prefix from the hostname inside your config, matches the current prefix. If not, the prefix gets updated.

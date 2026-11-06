# Automatic hibernation exceptions

**Settings → Performance & Privacy → Hibernation exceptions** keeps listed domains and their subdomains out of automatic hibernation, including the resident-tab limit. Entries are hostnames without a scheme, path or port. The browser trims whitespace and a final dot, normalizes case and internationalized names, and matches on DNS label boundaries, so `example.com` covers `shop.example.com` but not `notexample.com`.

The list does not change explicit hibernation commands. A user can still hibernate an exempt tab manually; regular protections such as the active-tab and edited-form checks continue to apply. Removing a domain makes its tabs eligible for a later automatic hibernation pass.

The list is stored with local preferences and included in backups. Older backups without the list restore an empty list.

neo
===

`neo` is a simple command line client for [Neocities](http://neocities.org).

Status
------

All basic functionality is available; `info`, `list`, `upload`, and `delete`
commands work as you would expect. More advanced features are planned if this
gets users.

There is some lack of documentation at the moment, but `neo help`,
`neo help <command>`, and the follwoing section should tell you pretty much
everything you need to know.

All feedback on the current interface is more than welcome, please open an
issue if you have any comments.

Getting Started
---------------

The usual way of using `neo` is to setup a `Neo.toml` at the root directory of
your site. In this file you can put your site name and auth credentials, either
your username and password or a site API key (see your site settings).

Example `Neo.toml`:

```toml
default_site = "neo-cli"

[sites]
neo-cli = { key = "<key>" }
my-site = { password = "<password>" }
```

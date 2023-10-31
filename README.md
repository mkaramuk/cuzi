# Cuzi

Cuzi (pronounced _ghu-zii_, which means _little_ in Turkish), is an educational project that aimed at make a lightweight reverse proxy alternative using the popular programming language Rust.

## Build

You can use the `cargo run` command to run the project. It is currently under development, and you can edit the `config.json` file at the root of the repository according to your preferences.

## Configuration

Let's take a look at the configuration.

A very simple configuration would look like this:

```json
{
	"port": 2001,
	"proxies": [
		{
			"path": "/",
			"target": "example.com",
			"target_port": 443,
			"use_tls": true
		}
	]
}
```

The port field defines the port on which Cuzi operates. The proxies field defines the routes that will be redirected, and each proxy has its own configuration.

### Proxy configuration

The `path` field defines the route to be redirected. In the example, it is set to `/`. If Cuzi is running on a machine with the IP address `192.168.1.43`, you can trigger this route by typing `https://192.168.1.43:2001/`, which will route to /.

The `target` field defines where this route leads. In the example above, it is set to `example.com`, which is an example domain registered by IANA.

The `target_port` field specifies the port of the target domain to which the connection will be made.

The `use_tls` field determines whether the socket communication will be over TLS or not.

You can add as many proxies as you need.

## Contribute

Cuzi is more than just an educational project; I intend to develop it further. If you wish to contribute to the project, learn, and collaborate together, please feel free to open a pull request or reach out to me via e-mail/discord, etc.

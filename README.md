# About

Pushtime is a small binary that integrates with the [Pushover][pushover] service to send push notifications with execution information to your mobile device.

It is particularly handy when you want to get notified upon completion of long running tasks.

Pushtime is inspired by the GNU utility `time` that is used to measure execution time of a command.

# Usage

Ensure your configuration is available at `$HOME/.pushtime`. Sample configuration is included in this repository as `.pushtime.sample`, it should look like the following:

```
PUSHOVER_USER=USER-TOKEN
PUSHOVER_TOKEN=APPLICATION-TOKEN
```

You can fetch these tokens from your [Pushover][pushover] dashboard.

## Basic Commands

Run your command with `pushtime` as a prefix:

```shell
pushtime ls
```

You will see the output of the command, followed by a push notification identifying the total runtime, and exit code, along with the full command line used.

## Commands with arguments

Nothing special required. Anything you put after `pushtime` will be considered a command and its arguments.

```shell
pushtime ls -la
```


[pushover]: https://pushover.net
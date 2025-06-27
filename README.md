# IDEA SSH Fix

Watch for JetBrains IDEA `.desktop` file changes and make sure `Exec=idea %u` is always set.

Then you can put the following script in your `~/.local/bin/idea` and run `chmod +x ~/.local/bin/idea`:

```shell
#!/bin/sh
export SSH_AUTH_SOCK=$(gpgconf --list-dirs agent-ssh-socket)
exec /home/pplaczek/.local/share/JetBrains/Toolbox/scripts/idea "$@"
```

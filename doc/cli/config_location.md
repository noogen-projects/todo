# Config location

## Config location resolution

1. If environment variable `TODO_ROOT_CONFIG` is set, then this path locates the root config file.

2. Otherwise, the root config file is located in the current user home directory, at `<HOME>/.todo/todo.toml`.

3. If an additional config file path is specified by `--config-file` argument, then:
    1.1. If the config file path is absolute, then this path locates the additional config directly.
    1.2. If the config file path is relative, then this path locates the additional config relative to the current working directory.

4. Otherwise, the additional config file will be searched in the current working directory, at `./todo.toml` and in its parent directories.

5. If additional config file are found, then the additional config are merged into the root config wich overrides the root configurations.

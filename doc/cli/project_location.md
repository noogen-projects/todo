# Project location

## Project location resolution

1. If a project path is specified (provided by `--project-path` or `--project` argument), then:
  1.1. If the project path is absolute, then this path locates the project directly.
  1.2. If the project path is relative, then this path locates the project relative to the current working directory.

2. Otherwise, if a project ID is specified (provided by `--project-id` or `--project` argument), then:
  2.1. If the working mode is `Local`:
    2.1.1. Attempt to locate the project with the given ID in the current working directory.
    2.1.2. If not found, search for the project with the given ID in the parent directories of the current working directory.
  2.2. If the working mode is `Global`:
    2.2.1. Attempt to locate the project with the given ID in the global project list (as defined in the configuration).
    2.2.2. If not found, search for the project with the given ID in the global project search roots (as defined in the configuration).

3. Otherwise, if a project name is specified (provided by `--project-name` or `--project` argument), attempt to locate the project by name using the same logic as in section 2.

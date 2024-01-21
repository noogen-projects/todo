# Todo

A universal, locally-oriented task manager.

## Simple example for the file-system tracker

List projects and their subprojects:

```sh
$ todo project list
project A
  ├─ project B
  └─ project C
project D
```

Show info about projects and issues:

```sh
$ todo project info
project A
  ├─ project B
  │    - To complete task 1
  │    - Some other task 2
  │    # Milestone A
  │    ..3
  └─ project C
project D
```

List of project issues stored in the `TODO.md` file:

```sh
$ todo list -p "project B"
- To complete task 1
  - Subtask 1
  - Subtask 2
- Some other task 2

# Milestone A

- Some other task 3
  - Subtask 3
- Some other task 4
```

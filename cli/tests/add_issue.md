# Add issue to project

## Prepare projects

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ todo new --with-manifest "test B"
    Creating `test B` project
```

## Add issue inside project dir

```sh
$ todo add "task 1"
Error: could not find `Project.toml` or `*.manifest.md` in `${current_dir_path}` or any parent directory
```

```sh
$ cd "test A"
$ todo add "task 1"
    Adding `task 1` issue to `test A` project
```

```sh
$ ls "test A"
Project.toml TODO.md
```

```sh
$ cat "test A/TODO.md"
- task 1
```

```sh
$ cd "test A"
$ todo add "task 2"
    Adding `task 2` issue to `test A` project
```

```sh
$ cat "test A/TODO.md"
- task 2
- task 1
```

```sh
$ cd "test A"
$ todo add "task 1"
    Adding `task 1` issue to `test A` project
Error: issue `task 1` in `${current_dir_path}` already exists
```

```sh
$ cd "test A"
$ todo add "task 2"
    Adding `task 2` issue to `test A` project
Error: issue `task 2` in `${current_dir_path}` already exists
```

```sh
$ ls "test A"
Project.toml TODO.md
```

```sh
$ cat "test A/TODO.md"
- task 2
- task 1
```

```sh
$ cd "test A"
$ todo add --last "task 3"
    Adding `task 3` issue to `test A` project
```

```sh
$ cat "test A/TODO.md"
- task 2
- task 1
- task 3
```

## Add issue to specifyed project

```sh
$ todo add "task 1" --project "test A"
    Adding `task 1` issue to `test A` project
Error: issue `task 1` in `${current_dir_path}/test A` already exists
```

```sh
$ rm "test A/TODO.md"
$ ls "test A"
Project.toml
```

```sh
$ todo add "task 1" --project "test A"
    Adding `task 1` issue to `test A` project
```

```sh
$ ls "test A"
Project.toml TODO.md
```

```sh
$ cat "test A/TODO.md"
- task 1
```

```sh
$ todo add "task 2" --project "test A"
    Adding `task 2` issue to `test A` project
```

```sh
$ cat "test A/TODO.md"
- task 2
- task 1
```

```sh
$ todo add --last "task 3" --project "test A"
    Adding `task 3` issue to `test A` project
```

```sh
$ cat "test A/TODO.md"
- task 2
- task 1
- task 3
```

```sh
$ todo add "task 1" --project "test B"
    Adding `task 1` issue to `test B` project
```

```sh
$ ls "test B"
test B.manifest.md
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
```md todo
- task 1
```
````

```sh
$ todo add "task 1" --project "test B"
    Adding `task 1` issue to `test B` project
Error: issue `task 1` in `${current_dir_path}/test B` already exists
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
```md todo
- task 1
```
````

```sh
$ todo add "task 2" --project "test B"
    Adding `task 2` issue to `test B` project
```

```sh
$ ls "test B"
test B.manifest.md
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
```md todo
- task 2
- task 1
```
````

```sh
$ todo add --last --project "test B" "task 3"
    Adding `task 3` issue to `test B` project
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
```md todo
- task 2
- task 1
- task 3
```
````

# Add issue to subproject

## Prepare subprojects

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ todo new "test A/test C"
    Creating `test C` project under `${current_dir_path}/test A`
```

```sh
$ todo new --with-manifest "test A/test D"
    Creating `test D` project under `${current_dir_path}/test A`
```


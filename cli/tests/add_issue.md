# Add issue

## Prepare projects

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ todo new "test A/test B"
    Creating `test B` project under `${current_dir_path}/test A`
```

```sh
$ todo new --with-manifest "test A/test C"
    Creating `test C` project under `${current_dir_path}/test A`
```

```sh
$ todo new --with-manifest "test D"
    Creating `test D` project
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
Project.toml TODO.md test C test B
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
Project.toml TODO.md test C test B
```

```sh
$ cat "test A/TODO.md"
- task 2
- task 1
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
Project.toml test C test B
```

```sh
$ todo add "task 1" --project "test A"
    Adding `task 1` issue to `test A` project
```

```sh
$ ls "test A"
Project.toml TODO.md test C test B
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
$ todo add "task 1" --project "test D"
    Adding `task 1` issue to `test D` project
```

```sh
$ ls "test D"
test D.manifest.md
```

````sh
$ cat "test D/test D.manifest.md"
# test D

```toml project
id = "test D"
name = "test D"
```
```md todo
- task 1
```
````

```sh
$ todo add "task 1" --project "test D"
    Adding `task 1` issue to `test D` project
Error: issue `task 1` in `${current_dir_path}/test D` already exists
```

````sh
$ cat "test D/test D.manifest.md"
# test D

```toml project
id = "test D"
name = "test D"
```
```md todo
- task 1
```
````

```sh
$ todo add "task 2" --project "test D"
    Adding `task 2` issue to `test D` project
```

```sh
$ ls "test D"
test D.manifest.md
```

````sh
$ cat "test D/test D.manifest.md"
# test D

```toml project
id = "test D"
name = "test D"
```
```md todo
- task 2
- task 1
```
````

## Add issue to subproject


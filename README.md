# MARKDOWN executor

A simple application that parses markdown files and executes the _shell_ code
blocks. Consider a markdown file, named `README.md`, with the following content:

![README.md](./assets/images/README.md.png)

By using the MARKDOWN executor application, you can execute all _shell_ code
blocks encapsulated by ` ```shell ` and ` ``` ` markers. To run the commands
from the `README.md` file in the current directory, execute:

```shell
$ me
```

This command will output the following.

```
Hello, world!
```

In this example we only have one _shell_ code block that simply prints
`Hello, world!`.

For additional options and usage instructions, you can access the help
documentation with:

```shell
$ me --help
```

## Development Guidelines

1. Build and test the application

   ```shell
   $ cargo fmt
   $ cargo clippy
   $ cargo build --release
   $ cargo test
   ```

2. Continuous delivery

   The MARKDOWN executor employs GitHub Actions for automated building and
   testing. Every push to the `main` branch triggers these actions, ensuring
   that the application remains stable and functional with each update.

   You can find all official releases
   [here](https://github.com/albertattard/me/releases). There is no need for
   special tags; simply push your changes to `main`, and GitHub Actions will
   handle the build and testing processes.

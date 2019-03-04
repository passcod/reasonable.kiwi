# Reasonable Kiwi

## Develop

### Getting started

Most of Reasonable will fail to build without the presence of the `ARCH_KEYS`
variable in the **build environment**. Those keys should be generated with the
`arch_keygen` binary, thus:

```
$ cargo run --bin arch_keygen >> .env
```

The arch public key will be printed out. Copy that onto the next command:

```
$ cargo run --bin master_keygen $ARCH_PUBLIC_KEY >> .env
```

Now you can run usual cargo commands by prefixing them with `dotenv`, which you
can install with cargo. You'll also need the `diesel` tool. Install both as
needed with:

```
$ cargo install dotenv --features cli
$ cargo install diesel_cli
```

Then start postgres, create a database, and add it to the `.env`, e.g.:

```
DATABASE_URL=postgres://localhost/reasonable
```

### Checking out

While the above setup need only be done the once, some steps should be done
whenever new code is pulled:

 - Run migrations:

   ```
   $ diesel migration run
   ```

- TBC

## Etc

 - Copyright © [Félix Saparelli](https://passcod.name).
 - Licensed under the [Artistic License 2.0](./LICENSE).

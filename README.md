# Reasonable Kiwi

## Workings

This is more of a brain-dump than a good and proper document.

So, Reasonable is about building a third-party service that people can use to
privately store reasons or comments about actions they take or took on Twitter.
Notably, blocks and mutes, but also follows, so may as well be generic. The
private part is very important, because those reasons could be highly
sensitive, contain personal information, and generally may be used for more
harassment than not. That's ironic, but also terrible, so this is important.

So, okay, just stick the reasons in a DB, make it visible only to the user when
they login using Twitter OAuth 3-party flow, done! Right?

Not quite. The threat model is not just "the public", it's also people hacking
the service and dumping the DB, or reading data through exploits, by which
point if the only protection is a login token then everyone is thouroughly
boned. The design outlined below protects against that. It also protects in a
limited fashion against the "attacker is myself" scenario, whereupon I turn
evil and start reading everyone's stuff.

There's two other vectors which are much harder to guard against, but I
should mention them here for completeness:

1. People hacking the service and staying in there while e.g. tracing
   processes or something. There's a limited amount of protection against
   this, but this is pretty much a nightmare scenario tbh.

2. People hacking the Twitter account and logging into Reasonable via those
   credentials. There's basically no protection against this.

One way to protect against these would be to make Reasonable a desktop or
mobile app instead, that keeps the data entirely local. That adds other
problems, though, foremost about processing power, storage space, backup, sync,
etc. Another option would be to provide another key to Reasonable and doing
end-to-end encryption, such that the key never leaves the browser or app. There
would still be some risk to data not encrypted with the personal key (see
below). However, that would also add a significant barrier to using the
service, and may even lure into bad security if the user's end-to-end
passphrase is weak! Nonetheless, either or both of these could be an option way
down the line, maybe, or someone else could do it (for the separate app thing).
For now let's just acknowledge these last two threats as fairly unmitigable.

So, the key insight here is that it's possible for an app to store an extremely
small amount of data (~100 bytes) on a Twitter user's account in such a way
that it's completely hidden from every other party except:

1. The user themself, and
2. Any other app authorised to read the account.

That way is: **private lists**.

That is, an app can create a private list with no members, and set the
name/title and the description of the list to arbitrary strings. Then the app
can retrieve the list of lists from the user, and the description, and retrieve
the stored data that way.

There's a very strict limit of characters on both the title and the description
of a list. Also, we probably want to have some kind of recognisable text there
so the user doesn't assume it's garbage and delete the list (which could e.g.
lock them out). So that limits things even more. What I ended up doing, after a
bit of experimenting, was:

 - **Title**: `Reasonable key (pls keep)`
 - **Description**: `Don't change! ➡️ https://reasonable.kiwi/help/list-key $ENCODED_KEY`

The encoded key is some random bytes encoded with [qntm's Base65536][b65536].
Now, the readme for that project explains that for modern tweets, base2048
should be used instead, because of how Twitter does its character counting. But
it turns out that list descriptions still use Twitter's **old** character
counting, so base65535 is the correct choice!

So now we have a crypto key that only the user and any authorised apps can
read. That's good if we're the only app there, but not so good in the much more
common case where that's not the case. It's also no good if the user decides to
screenshot the key or their list of lists and post it somewhere.

So to protect against that, we encrypt the key with Reasonable's key. Now only
Reasonable, in co-operation with the user, can read the key.

After that, "all" that's needed is to chain and layer crypto both ways (down
from the user's key, and up from Reasonable's key), and we're in business!

Some of the perhaps overkill things I've done / thought of:

 - Reasonable would be built and deployed as a single binary. There would be an
   "arch" assymetric keypair and an additional "check" public key built inside
   the binary itself. Then at runtime, a "master" symmetric key would be
   provided via the environment, signed by the private "check" key and
   encrypted by the "arch" keys. On application boot, the master key would be
   decrypted then its signature checked. That is likely the most overkill,
   unnecessary, and fiddly part, but the idea was to prevent someone obtaining
   hold of the binary or the source and getting the keys to the kingdom.

 - Primary keys in the database are UUIDs to avoid lookup-ability and any sense
   of natural ordering.

 - A user's twitter ID needs to be stored fairly accessibly to lookup a user
   while login and for most operations. But there's no need to store the actual
   user ID in plain text. Instead, we can concat the twitter ID and the master
   key, then hash the lot, and that gives us something we can match and index!

 - We also need to store a user's Twitter access key and secret for the API
   interactions. Again, no need to store in plain text, just encrypt a small
   structure with the master key, and open it only on demand when the keys are
   needed.

 - At that point we have access to their personal key and can open their actual
   reasons, write new ones, modify existing ones, etc. To do any kind of
   searching requires fairly opaque indexes, with more hashing to both enable
   the indexing while guaranteeing privacy.

 - For now I figured it would be enough, esp for a proof of concept, but in the
   future it may be wise to consider a key rotation scheme per user that
   doesn't involve re-encrypting every object.

Crypto is done with sodium.

[b65536]: https://github.com/qntm/base65536

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

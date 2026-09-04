# Encrypted files

An `encrypt` rule stores the file as ciphertext in the repository. The
plaintext only exists at the path the file is placed on.

```lua
ld.crypt.lock({
  recipients = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p",
  identity = "~/.keys/age.txt",
})

ld.rules({
  { match = ".ssh/id_*", encrypt = true },
  { match = ".netrc", encrypt = true },
  { match = ".config/wireguard/**", encrypt = true, mode = "0640" },
})
```

Encryption runs through the `age` or `gpg` binary on your `PATH`; luadot ships
no cryptography of its own. `age` is the default, `ld.crypt.backend("gpg")`
switches. The stored file keeps its path and gains the backend's extension:
`add ~/.netrc` lands in `.netrc.age`. The extension is what marks a file
as encrypted from then on, whatever the rules say later.

- `add` encrypts into the repository and never writes the plaintext there.
- `take` re-encrypts the system copy into the repository, for the recipients
  set now.
- `apply` decrypts to the system as a plain copy, whatever `link` says, with
  mode `600` unless a `mode` rule says otherwise, and the `owner` a rule
  names.
- `status` compares the decrypted content against the system copy; a file it
  cannot decrypt is `unreadable`.
- `diff` compares the decrypted content too, and stages both sides in the
  private mirror it builds for every diff, so no plaintext outlives the
  command. The report names the file as you use it, `.netrc` and not
  `.netrc.age`.
- `edit` decrypts to a private temporary directory (`0700`, under
  `$XDG_RUNTIME_DIR` when it exists) and writes the plaintext `600` there,
  opens the editor, re-encrypts and removes it, even when the editor exits
  badly. An unchanged file is left alone.
- `rm` deletes the ciphertext; when the system copy is missing it decrypts one
  last time to leave the plaintext behind.
- Conflict policies compare the decrypted content against the system.

Encrypting is done to the `recipients` (age public keys, or key ids for gpg).
Decrypting with age needs the `identity`, the private key file; gpg ignores it
and uses its keyring both ways. A failed decryption stops `apply` with the
tool's own error rather than skipping the file; `diff` warns and leaves that
file out of the report.

## The identity

The `identity` also takes a command, for a key living in a password manager.
The command's output is the private key: it runs once per command, never per
file, and the key is written to a `600` file in the same private temporary
directory `edit` uses, removed when the command ends.

A written identity is read as a command when it carries a space, as a path
when it does not. A `type` sets it explicitly, and several words are a program
and its arguments, run without a shell:

```lua
ld.crypt.lock({ recipients = "age1ql3z…", identity = "~/.keys/age.txt" })
ld.crypt.lock({ recipients = "age1ql3z…", identity = "pass show age/key" })
ld.crypt.lock({ identity = { type = "file", "/mnt/my key.txt" } })
ld.crypt.lock({ identity = { type = "command", "unlock-key" } })
ld.crypt.lock({ identity = { "op", "read", "op://vault/age/key" } })
```

`type` is `"file"` or `"command"`, one per lock, never both.

age plugins work as for age itself: point `identity` at the plugin identity
and use the plugin's recipients. luadot only checks that the plugin binary a
key names is on your `PATH` (`AGE-PLUGIN-YUBIKEY-1…` and `age1yubikey1…` both
need `age-plugin-yubikey`) and reports the missing one.

## Locking with a passphrase

`ld.crypt.lock("passphrase")` encrypts to a passphrase instead of keys: `age
--passphrase` or `gpg --symmetric`, with the tool doing the asking, so nothing
about the passphrase passes through luadot. **It is weaker than keys**: one
passphrase opens every secret, everyone sharing the repository shares it, and
changing it means re-encrypting everything. Every command touching a secret
says so once; `ld.opt.passphrase_warn(false)` silences that line. age asks per
file, and only gpg's agent caches the answer, so expect one prompt per secret
with age.

The call takes one form or the other, the string `"passphrase"` or a table of
keys. A configuration that computes it picks between the two at the call:

```lua
ld.crypt.lock(ld.class.get("profile") == "personal" and "passphrase" or {
  recipients = "age1example",
})
```

`ld.crypt.backend` is independent and combines with either lock.

## rekey

Changing the recipients does not reach the files already stored. `luadot
rekey` decrypts each secret and encrypts it again for the recipients set now,
in place, one staging file at a time so a failure never leaves a half-written
secret. `-n` reports what it would touch. Switching `ld.crypt.backend` and
running it moves each secret to the other extension (`.netrc.age` becomes
`.netrc.gpg`). `rekey` commits nothing.

## The calls

| Call | Arguments | Effect |
| --- | --- | --- |
| `ld.crypt.backend(name)` | `"age"`, `"gpg"` | Tool used to encrypt and decrypt managed files. Defaults to `"age"`. |
| `ld.crypt.lock(lock)` | `"passphrase"`, or a table of `recipients` and `identity` | How secrets are locked: the word locks with a passphrase, the table with keys. Defaults to keys with none set. |
| `ld.crypt.lock`'s `recipients` | a key or a list of them | Public keys or key ids the files are encrypted to. |
| `ld.crypt.lock`'s `identity` | a path, a command line, or a table carrying `type` and its value | Private key used to decrypt with age; gpg uses its keyring. A path resolves `~` and a relative path against your home directory; a command prints the key instead. |
| `ld.opt.passphrase_warn(enabled)` | `true`, `false` | Whether passphrase mode says it is weaker than keys. Defaults to `true`. |
| `ld.crypt(options)` | a table of options | Sets several options at once; only the keys it carries. |

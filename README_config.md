Config files
============

We have two config files for Yarrosco: `yarrsecrets.toml` and `yarrosco.toml`

Remember that Yarrosco expects to find them on the current folder from where
it is running.

## yarrsecrets.toml

The config `yarrsecrets.toml` is only useful if you want to hide the 
credentials for your services. It **does not** add security. If someone gets
access to your drive files they can easily decipher the secrets. It's only 
useful if you want to stream yourself developing Yarrosco's code, as it would
prevent accidental leak of keys.

If you don't need this, you can just leave the file empty with just:

    [secrets]

Also you can also delete the file and the result is the same.

**If you're not interested on setting up secrets, just move to the next section**

This file sets placeholders and their replacements.

For example:

    [secrets.twitch]
    placeholder = '%%TWITCH_OAUTH_TOKEN%%'
    secret = 'EF_bbakdBkR9JZPZBUlfOg|IIjPbB-yCiGI45CLxpEk28F6llvJeXztZ56MFZXE07F53QObvz_tz10P5IQNx6cEgwmp71Sm_Lc7Eqalj0tKQAU'
    version = 1

This defines a secret called "twitch". The name is one of your choice. Another sample:

    [secrets.twitch_for_myusername]

The placeholder is a string that it will be replaced in the main config. It has to be complex enough so nothing else matches.

Bad: **(DON'T DO THIS)** 

    placeholder = 'token'

By doing that, every time the word "token" appears in the config, or as part of another word (i.e. `/token/` or `detokenizer`) will be replaced inside, breaking your config and potentially exposing your secrets.

Therefore, prefer having very specific strings, and they also **MUST be unique**.

Good examples:

    placeholder = '$$TWITCH_TOKEN_USERNAME$$'
    placeholder = '{{TWITCH_TOKEN_USERNAME}}'
    placeholder = '{@twitch.username.token@}'

Version is always 1. This is reserved in case we need to change the scheme for 
how the secret is encoded.

The secret is obtained by running `yarrpass`. Before this, it is recommended to 
have a password set in your `.bashrc`, so maybe append this to the end of the file:

    export YARROSCO_PASSPHRASE="set a big password or pass phrase here"

And log off and log in from your desktop for this to apply. Alternatively, run
also the command in your console.

You can also get a pretty good password by executing:

    $ head -n20000 /dev/random | sha256sum
    d414e08ced933923d2d3b14fe713d3ebd06d44050bc4b8a29484ea1430f0580e  -

Be aware that if you lose the password (move to a different computer), you will
need to create the secrets again. They can't be recovered without the password.

Once this is done, launch `yarrpass`:

    $ cargo run --bin yarrpass
    $ cargo run --bin yarrpass
    Finished dev [unoptimized + debuginfo] target(s) in 1.01s
    Running `target/debug/yarrpass`
    **** ENCODE ****
    Input Secret Message:   (input is always hidden)
    Token: qj3sZZEdX3ZphKa7LO3zQw|pIBGgR91ZbadgAGBFrD0GN5EhogHIei2KC9mS75-mSO7lYM8j0u5VXOpwTuvkZ9oxg

When it asks for the secret message, paste the token or password you need, then
hit intro.

Yarrpass will return the "token", which is the secret encoded using the password.

Then you just have to paste this token in the config:

    secret = 'qj3sZZEdX3ZphKa7LO3zQw|pIBGgR91ZbadgAGBFrD0GN5EhogHIei2KC9mS75-mSO7lYM8j0u5VXOpwTuvkZ9oxg'

Once this is done, Yarrosco will replace the entries in the config of `%%TWITCH_OAUTH_TOKEN%%` with the decoded message.

Repeat this for as many secrets do you need.

You can use this for anywhere in your config, but only the passwords and oauth_tokens are protected. Other fields might be printed in debug messages.

(.. TBD ..)

## yarrosco.toml

The config `yarrosco.toml` contains all configuration options possible. Tune
them to your liking.

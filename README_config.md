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

This file 

## yarrosco.toml

The config `yarrosco.toml` contains all configuration options possible. Tune
them to your liking.

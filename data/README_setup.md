Configuring the HTML App
=========================

This folder contains an HTML application using a single Typescript file that 
will query using AJAX the files `yarrdb_data.jsonl` and `yarrdb_log.jsonl`.

The result will be parsed and held in an in-memory database, then the chat is
rendered from this memory database.

> **NOTE:** This app attempts to read from the "current folder", such as 
> doing `url="./yarrdb_data.jsonl"`. The database files must be accessible this way.
> Later on it will be explained how to configure this.

> **NOTE:** This has only been tested by using a local web server, Firefox ESR and OBS
> capturing the Firefox window. Other methods might work, but haven't been tested.
> (As I don't have OBS Browser plugin, I can't test that)

# Prerequisites

* Local Web server running. Any port is okay. We will assume the standard port 80.
  If you have yours running in a different port, just use `http://localhost:9999/`
  where 9999 is your port number.
* A folder that the web server is actually serving, that is also writable by the
  user that runs Yarrosco.

# Short version

1. Mount a folder in your web server for localhost. In our case, this folder is
   just `~/www` (`/home/youruser/www`).

2. Ensure that this folder has the same owner and group as the ones Yarrosco is
   running as.

3. Create a folder called "yarrosco": `~/www/yarrosco/`

4. Link the following files to `~/www/yarrosco/`:
    * data/script.js
    * data/styles_base.css
    * data/styles_yarr1.css
    * data/yarrosco_chat.html
    * data/yarrosco.ico
    > **TIP:** You can do this by executing: 
    > ```bash
    > ln -s data/*.js data/*.css data/*.html data/*.ico  ~/www/yarrosco/
    > ```

5. Ensure these files can be accessed when navigating to http://localhost/yarrosco
    > **NOTE:** Most web servers by default would reject following soft-links. 
    > Ensure to disable this behavior.

6. Link the database files to `~/www/yarrosco/`:
    * yarrdb_log.jsonl    
    * yarrdb_data.jsonl
    > **TIP:** You can do this by executing: 
    > ```bash
    > ln -s yarrdb*.jsonl  ~/www/yarrosco/
    > ```

7. Navigate to http://localhost/yarrosco/yarrosco_chat.html and test if it works as expected.
    If something doesn't seem to work, open the Javascript console and inspect the logs.

## If you need to install a server

I recommend using Nginx. In Debian/Ubuntu, it can be installed with just:

    $ sudo apt install nginx

This will create files under `/etc/nginx` and by default it serves in the port 80.

> **NOTE:** Port 80 is the default http port, and it works when you access http://localhost. 
> If you choose anther port, for example 8080, you'll need to use http://localhost:8080 instead.

You should have now a file called `/etc/nginxsites-enabled/nginx-test-site`.

We can change the contents of this file to match our needs:
```
server {
        listen 80 default_server;
        listen [::]:80 default_server;

        root /home/user/www;   # <--- Change THIS!

        index index.html;

        server_name test.localhost;

        location / {
        autoindex on;
        try_files $uri $uri/ =404;
        }
}
```

This is not the best way to do it, but it does work and is simple enough.

After the change, just execute:

    $ sudo /etc/init.d/nginx restart

And now go to http://localhost and test your changes.

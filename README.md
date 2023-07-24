# core

## USES UTC EPOCH TIME

<!--
    TODO:
        - add an expiry, might need to restructure how it works, (use numbers)
        - make the get more efficient as it is somewhat of a big string, it should just be the value or empty string if non-existent key
        - send errors back
        - remove all the unwraps
        - create a 'batch' command, define a schema for it to take one dict with a bunch of key and value pairs (does all the pairs have the same expiry? or separate expiries?) (DO BOTH)
        - add metrics like pg_stats?
        - better error messages
        - separate code in different files and folders i.e. models in models dir
	- delete command
	- create a cli tool like redis
	- create a frontend like akms with docs and a hosted server(http only)
	- dockerize this and publish to a image repository
	- need another server for public use (i.e. each hash is contained within a user)
-->

<!-- TODO: protocol buffers:
    - this would need to be another server
    - another repo
    - maybe a flag on the client library to select which one to use (tcp or pb)
    - possibly a way to have it on the same server i.e. this main.rs file?
-->

<!-- TODO: search up redis features -->
<!-- TODO: a leaderboard feature -->

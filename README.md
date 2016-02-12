JSONCDC
=======

JSONCDC provides change data control for Postgres, translating the Postgres
write ahead log to JSON.

It is written in Rust and, being short, is a good skeleton project for other
would be plugin authors who'd like to use Rust instead of C when writing
Postgres extensions.

Our library Requires rust stable 1.1 or greater.  You can bug `posix4e` (or
`posix4e_`) on Freenode if you want to get involved.

Tasks to work on should be available on:
[![HuBoard
badge](http://img.shields.io/badge/Hu-Board-7965cc.svg)](https://huboard.com/posix4e/jsoncdc)

[![Linux
Status](https://travis-ci.org/posix4e/jsoncdc.svg?branch=master)](https://travis-ci.org/posix4e/jsoncdc)


Copyright and License
---------------------

Copyright (c) 2016 Alex Newman, Jason Dusek


Status
------

JSONCDC is presently installable with `pgxn`, from the unstable channel:
`pgxn install jsoncdc --unstable`.


Usage
-----

    SELECT * FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');
    --- Wait for some transactions, and then:
    SELECT * FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);


Formats
-------

- [x] JSON output
- [ ] Protobufs output
- [ ] Avro output

Destinations
------------

- [ ] File
- [ ] HTTP
- [ ] Kafka
- [ ] Kinesis

Features
--------

- [ ] Monitored by Rust metrics

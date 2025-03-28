# Contributing to the LTZF Project

## First and Foremost
Thanks!

## Second, TLDR
The project consists of three parts. To read more about them, I recommend [docs/README.md](docs/README.md). 
Make yourself familiar with the structure!

A quick setup instruction for development can be found in [SETUP.md](SETUP.md).

## State of the Project and the most Pressing Construction Sites
The project is currently in a very very unstable state. 
This primarily means the API is changing in subtle ways every now and then, (although the pace of change is slowing).
The reason for this is that we are still figuring out how to  represent seventeen independent legislative processes, which is an ongoing venture.

If this is fine, then the most pressing concerns would be
1. if you know python
    1. give the collector subproject a little overhaul, solidifying error handling, logging and general execution structure
    2. give the document handling in collectors a little overhaul! I am sure there is a better was to do it than what happens now
    3. give the caching a little overhaul! I would love a local caching option for example, without redis
    4. write tests for the collector sub-project as well as any scraper you might find there or write yourself.
    5. (after some meddling with collectors) give us input on how to improve the http api from the collection side! There is always room for improvement and better consistency
2. if you know how to do websites
    1. give us one, please. The one currently residing in webserver/ is a placeholder and more or less a way of making sure the correct data comes out
    2. give your input on how to improve the http api from the GET-side of things! Suggestions are very much appreciated
3. if you know rust
    1. write tests for the database/ subproject. Scenario tests as well as unit tests in rust
    2. (if you also know sql) review the more complex queries. The filtering, the merge candidate finding and the merging itself are not well tested at this point
4. if you have a system design background
    1. give us feedback on the whole structure!
5. if you are just here for shits and giggles
    1. write documentation for the existing system
    2. do some research into government websites and where which information is accessible, documenting it as well as possible to improve the lives of future scraper writers
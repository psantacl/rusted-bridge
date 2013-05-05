# Rusted Bridge

## What is it?
  Rusted bridge is essentialy my version of the [nailgun](http://www.martiansoftware.com/nailgun/background.html) project which provides an optimization strategy for jvm load times so that clojure code can effectively be used for commandline apps.  The first time rusted bridge is invoked for a specific app, it  will spin up the jvm and daemonize it in the background.  All subsequent interaction with that specific clojure app are proxied over a non-blocking socket to the already running jvm and thus are significantly faster. Standard in, standard out, and standard error are redirected over the bridge from the daemonized jvm to the Rust client running in your terminal.

## Why did you build this if nailgun already had it covered?
  I wanted to learn Rust and it seemed like a fun first project.

## Should I make use of rusted bridge?
  Probably not.  Rusted bridge was an educational exercise for me and is not maintained.  That said, if you would like to see examples of file i/o, json, and non-blocking sockets in Rust, you may find the code to be an interesting read. :)





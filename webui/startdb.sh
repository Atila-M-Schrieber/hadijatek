#!/bin/bash
#

~/.surrealdb/surreal start --log trace --user root --pass root --allow-funcs --bind 0.0.0.0:8080 file:test.db 

# Jitterish
Authors:

    MkB (primary)
    SingleChar

Categories:

    Pwn
    Potentially Rev
    Misc
    ?

## Overview

Jitterish is the best cloud storage your company could possibly rent!

It is a storage service with a web frontend. You can store data and retrieve it again. Data access is via JIT-compiled queries.

Example:
```
func is_in_team(entry, teamname) { return entry["team"] == teamname; }
func get_points(entry) { return entry["points"]; }
func sum(data, acc) { return data + (acc ?: 0); }

query all_entries on data1;
query team_points on data1 filter is_in_team map get_points reduce sum; 
```

### Web frontend
- User management
- Selects target database
- Users can submit queries
### Database engine
- Receives queries from users
- talks to the JIT to compile the query language
- database engine can write data
### JIT Compiler
- Produces executable files from user code
- Compiled files can only read data (maybe limited to some users)


## Flag Stores
Flag stores are not fully determined yet and subject to change.

In general, we envision most flag stores as fields in a user profile. There are private and public data tables, and there are various different vulnerabilities in each part that allow access to different fields of the profile/users.

Rough idea is to have at least one flag stores per component (so at least 3 in total, frontend, db engine, jit)

### Flag Store 1

The first flag store is an account with a private and a public table. Attackers can run JIT code against the public table but not the private table.
Bypassing the JIT limits (or RCE) give access to the private table.

### Flag Store 2
tbd...

## Vulnerabilities
### Web Frontend
tbd.
Maybe a typical misc vulnerability, or some vulnerable queries in our own jitted language.

    Difficulty: tbd, maybe medium
    Discoverability: ?
    Patchability: ?

### Database Engine
tbd.

    Difficulty: ?
    Discoverability: ?
    Patchability: ?
    
### JIT Compiler, vuln1 (RCE)
There'll be at least one vuln that gives you RCE when running the jitted query.
Current WIP approach, might change: 
```
func jit_rt_constant(){}
func jit_entrypoint() { system("sh"); }
```

    Difficulty: medium to hard
    Discoverability: medium (obvious if you know how linkers work, not so obvious otherwise)
    Patchability: medium
    
### JIT Compiler, vuln2 (symbol override)
You can override permission checks in the runtime library, by re-defining their internal symbols. WIP approach:
```
func _ZN11DataStorage8isPublicEv(){ return true; }
```

    Difficulty: medium
    Discoverability: hard
    Patchability: easy

## Patches
TBD.

## Current status
- JIT prototype exists

## Work Packages
### WP 1, JIT
- 90% prototype is there, zero checks at the moment, lets see how many we put
### WP 2, Database Engine
### WP 3, Frontend
### WP 4, Checker


## Timeline

(All points are until End of August. Thanks.)

Plan is to develop JIT, db engine and frontend somewhat in parallel because of dependencies. Rough goal for a first usable thing is roughly end of June.

After that, we will finalize checkers and sample exploits.

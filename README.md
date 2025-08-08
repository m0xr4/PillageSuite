# Pillage Suite

[![License: AGPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)

## About
Pillage Suite is a powerful graphical interface, inspired by Bloodhound, for exploring Network Shares in an Active Directory environment.
It provides tools for the indexing and querying of files, folders, shares as well as identities from Active Directory. 

Read more about the capabilities, usage aswell as motivation to create the tool on the release blog:

**NOTE**: This application is built to run on Windows, several currently used components rely on the Windows API.

### TL;DR
In almost every network pentest i ever did, we run into the same weakness: misconfigured SMB shares. Over-permissive ACLs, inherited rights nobody noticed, “temporary” Everyone:Read, and sensitive data parked on paths that were never meant to hold such data. Threat Actors don’t need to escalate their permissions if they can simply browse to the crown jewels with low privileged accounts after an initial access is established.

Pillage Suite enumerates and indexes effective permissions on content within SMB shares across your Active Directory. Through easy to create queries you are able to uncover data exposure before attackers exploit it.

**Highlights**
- Enumerate and index network shares to collect metadata on all contained files and directories, including applied ACLs
- Collect information of Active Directory to get a complete view on ACLs, including permissions from nested groups
- Predefined and user specified queries allow to create tables and graphs of indexed data through queries
- Search for Credentials in selected files


## Build Instructions
Compiled releases can be found here: https://github.com/m0xr4/PillageSuite/releases

### Install dependencies
  ```
npm install
cargo install tauri-cli
```
### Build the release .EXE
```
cd src-tauri
cargo tauri build
```
### Start dev server for debugging
```
npm run tauri dev
```

## Usage
For documentation about the tool, please consult the wiki: https://github.com/m0xr4/PillageSuite/wiki

You need a neo4j Instace as DB, look here for a ready to deploy docker config: https://github.com/m0xr4/PillageSuite/wiki/Data-Imports#neo4j-backend


## Disclaimer

I’m **not a professional developer**. This project is a learning/side effort, mainly created during free time. The code may contain bugs, incomplete features, or shortcuts that **do not follow best practices**—especially on the frontend. Use at your own risk.

This software is provided **“as is”**, without warranty of any kind. Under the GPL-3.0 license, there is **no warranty** and the author shall **not be liable** for any damages arising from the use of this software.

### Responsible Use
- **Only use this tool on networks, systems, and data you own or have explicit permission to test/use.**
- The author is **not responsible** for any misuse, unauthorized access, or violations of applicable laws or policies by users of this tool.

By using this software, you agree to use it responsibly and in compliance with all applicable laws, regulations, and terms of service.

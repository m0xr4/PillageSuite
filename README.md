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
- Easy to use features for data collection and imports 


## Build Instructions
The current standalone .exe release can be found here: https://github.com/m0xr4/PillageSuite/releases

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

## Data Ingestion
To explore data we first need to enumerate and index metadata from shares and AD. The tool comes with bundled actions to enumerate and import data. In situations where you cannot or dont want to run the enumeration directly from the GUI you are free to use standalone (prebuilt or self developed) enumeration tools.

### Bundled ingestions
The tool allows to enumerate shares and the AD through its "Active Index" menu.
<img width="1490" height="728" alt="index" src="https://github.com/user-attachments/assets/0a743338-7bff-4e7e-980e-3cf9961d8bde" />

To import data into the database, use the "Import" menu
<img width="1205" height="747" alt="import" src="https://github.com/user-attachments/assets/f02cca70-3812-4b7e-a63b-f517d60aa380" />


### Standalone ingestions
The bundled application uses rust components to conduct the enumerations. These components where first built as standalone tools and are available here:

If you would like to create your own enumeration tool, you're free to do so, just make sure the output is compliant with the expected formats.

## Neo4j Backend
Neo4j is used as a Database to hold all the enumerated data. below you can find a docker file to quickly spin up a neo4j Database. Usage of docker inside WSL is recommended.
If you do not use this configuration below, make sure your neo4j instance has the apoc plugin installed.
Username and password for the access is controlled with this line: ```NEO4J_AUTH=<username>/<your_password>```
```
version: "3.8"

services:
  neo4j-apoc:
    image: neo4j:2025.02.0
    container_name: neo4j-apoc
    ports:
      - "7474:7474"
      - "7687:7687"
    environment:
      - NEO4J_apoc_export_file_enabled=true
      - NEO4J_apoc_import_file_enabled=true
      - NEO4J_apoc_import_file_use__neo4j__config=true
      - NEO4J_PLUGINS=["apoc"]
      - NEO4J_AUTH=neo4j/your_password
      # Set initial heap and maximum heap
      - NEO4J_server_memory_heap_initial__size=4G
      - NEO4J_server_memory_heap_max__size=4G
      - NEO4J_dbms_memory_transaction_total_max=4G
```

## Disclaimer

I’m **not a professional developer**. This project is a learning/side effort, mainly created during free time. The code may contain bugs, incomplete features, or shortcuts that **do not follow best practices**—especially on the frontend. Use at your own risk.

This software is provided **“as is”**, without warranty of any kind. Under the GPL-3.0 license, there is **no warranty** and the author shall **not be liable** for any damages arising from the use of this software.

### Responsible Use
- **Only use this tool on networks, systems, and data you own or have explicit permission to test/use.**
- The author is **not responsible** for any misuse, unauthorized access, or violations of applicable laws or policies by users of this tool.

By using this software, you agree to use it responsibly and in compliance with all applicable laws, regulations, and terms of service.

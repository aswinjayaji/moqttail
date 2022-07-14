# DTLS CLI

Command Line Interface to DOS attack and test DTLS servers.

## Usage

`cargo run -- <path_to_config_file>`

## Example

`cargo run -- sample_config.txt`

### Note

Server should be running on the target socket otherwise you will get an OS Error.

## Config file format

```
<target>
<flight>
<times>
<flight>
<times>
[if flight is 3] <cookie> (hex string where each byte is represented by two characters)
<flight>
<times>
```
